use base64;
use jsonwebtoken::{decode, errors::ErrorKind, Algorithm, DecodingKey, Validation};
use lazy_static::lazy_static;
use log::{debug, error, info, warn};
use poem::Request;
use poem_openapi::auth::Bearer;
use poem_openapi::SecurityScheme;
use reqwest::Error as ReqwestError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::RwLock;

pub const USERNAME_PLACEHOLDER: &str = "ANONYMOUS-USER-PLACEHOLDER";
pub const EMAIL_PLACEHOLDER: &str = "anonymous@example.com";

lazy_static! {
    static ref PUBLIC_KEYS: RwLock<Vec<String>> = RwLock::new(vec![]);
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub username: String,
    pub email: String,
    pub organizations: Vec<i32>,
    pub projects: Vec<i32>,
}

impl User {
    fn new(username: &str, email: &str) -> Self {
        Self {
            username: username.to_string(),
            email: email.to_string(),
            // Be compatible with the old version, the token might not contain the organizations field.
            organizations: vec![-1],
            projects: vec![-1],
        }
    }

    fn add_organizations(&mut self, organizations: Vec<i32>) {
        self.organizations = organizations;
    }

    fn add_projects(&mut self, projects: Vec<i32>) {
        self.projects = projects;
    }
}

fn get_username_from_claims(claims: &Claims) -> Option<String> {
    if !claims.name.is_empty() {
        Some(claims.name.clone())
    } else if !claims.email.is_empty() {
        Some(claims.email.clone())
    } else if !claims.nickname.is_empty() {
        Some(claims.nickname.clone())
    } else {
        None
    }
}

#[derive(Debug, Deserialize)]
pub struct Jwks {
    keys: Vec<Jwk>,
}

#[derive(Debug, Deserialize)]
struct Jwk {
    kty: String,
    kid: String,
    n: String,
    e: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    nickname: String,
    name: String,
    picture: String,
    email: String,
    email_verified: bool,
    locale: Option<String>,
    updated_at: String,
    iss: String,
    aud: String,
    iat: i64,
    exp: i64,
    sub: String,
    nonce: String,
}

pub async fn fetch_and_store_jwks(url: &str) -> Result<Jwks, ReqwestError> {
    let response = reqwest::get(url).await?;
    let jwks = response.json::<Jwks>().await?;

    let mut keys = PUBLIC_KEYS.write().unwrap(); // Assuming PUBLIC_KEYS is a globally accessible RwLock
    *keys = jwks.keys.iter().map(|j| j.n.clone()).collect();

    Ok(jwks)
}

fn get_jwks_from_cache(kid: &str) -> Option<Jwks> {
    let keys = PUBLIC_KEYS.read().unwrap(); // Assuming PUBLIC_KEYS is a globally accessible RwLock
    if keys.is_empty() {
        return None;
    }

    info!("keys: {:?}", keys);
    let jwks = Jwks {
        keys: keys
            .iter()
            .map(|key| Jwk {
                kty: "RSA".to_string(),
                kid: kid.to_string(),
                n: key.to_string(),
                e: "AQAB".to_string(),
            })
            .collect(),
    };

    Some(jwks)
}

fn find_decoding_key(jwks: &Jwks, kid: &str) -> Option<DecodingKey> {
    jwks.keys
        .iter()
        .find(|j| j.kid == kid)
        .map(|jwk| DecodingKey::from_rsa_components(&jwk.n, &jwk.e).unwrap())
}

async fn validate_token_with_rs256(
    client_id: &str,
    token: &str,
    jwks: &Jwks,
    kid: &str,
) -> Result<Claims, String> {
    let decoding_key = find_decoding_key(jwks, kid).unwrap();
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[client_id]);
    let token_data =
        decode::<Claims>(token, &decoding_key, &validation).map_err(|e| e.to_string())?;

    Ok(token_data.claims)
}

// For simple scenarios, we can use HS256 to verify the token. Such as integrating with the label studio.
fn validate_token_with_hs256(token: &str, secret_key: &str) -> Result<User, String> {
    let token_data = decode::<User>(
        token,
        &DecodingKey::from_secret(secret_key.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .map_err(|e| e.to_string())?;

    Ok(token_data.claims)
}

fn detect_algrithom(token: &str) -> Result<Algorithm, ErrorKind> {
    if !token.contains('.') {
        return Err(ErrorKind::InvalidToken);
    }
    let parts: Vec<&str> = token.split('.').collect();
    let header = parts[0];
    let header_json = match base64::decode(header) {
        Ok(header_json) => header_json,
        Err(e) => {
            error!("Error: {}", e);
            return Err(ErrorKind::InvalidToken);
        }
    };
    let header_json_str = match String::from_utf8(header_json) {
        Ok(header_json_str) => header_json_str,
        Err(e) => {
            error!("Error: {}", e);
            return Err(ErrorKind::InvalidToken);
        }
    };
    let header_json_value: Value = match serde_json::from_str(&header_json_str) {
        Ok(header_json_value) => header_json_value,
        Err(e) => {
            error!("Error: {}", e);
            return Err(ErrorKind::InvalidToken);
        }
    };
    let alg = match header_json_value["alg"].as_str() {
        Some(alg) => alg,
        None => {
            error!("Error: invalid alg.");
            return Err(ErrorKind::InvalidToken);
        }
    };

    match alg {
        "HS256" => Ok(Algorithm::HS256),
        "RS256" => Ok(Algorithm::RS256),
        _ => Err(ErrorKind::InvalidAlgorithm),
    }
}

fn detect_kid(token_str: &str) -> Option<String> {
    if !token_str.contains('.') {
        return None;
    }

    let parts: Vec<&str> = token_str.split('.').collect();
    let header = parts[0];
    let header_json = base64::decode(header).unwrap();
    let header_json_str = String::from_utf8(header_json).unwrap();
    let header_json_value: Value = serde_json::from_str(&header_json_str).unwrap();
    let kid = header_json_value["kid"].as_str();
    match kid {
        Some(kid) => Some(kid.to_string()),
        None => None,
    }
}

#[derive(SecurityScheme)]
#[oai(type = "bearer", checker = "jwt_token_checker")]
pub struct CustomSecurityScheme(pub User);

async fn jwt_token_checker(_: &Request, bearer: Bearer) -> Option<User> {
    // Get jwt_secret_key from environment variable
    let default_user = Some(User::new(USERNAME_PLACEHOLDER, EMAIL_PLACEHOLDER));

    let jwt_secret_key = match std::env::var("JWT_SECRET_KEY") {
        Ok(key) => key,
        Err(err) => "".to_string(),
    };

    let jwt_client_id = match std::env::var("JWT_CLIENT_ID") {
        Ok(client_id) => client_id,
        Err(err) => "".to_string(),
    };

    let token_str = bearer.token;
    if jwt_secret_key.is_empty() && jwt_client_id.is_empty() {
        warn!("You don't set JWT_SECRET_KEY and JWT_CLIENT_ID environment variable, so we will skip JWT verification, but users also need to set the Authorization header to access the API.");
        return default_user;
    } else {
        debug!("JWT_SECRET_KEY: {}", jwt_secret_key);
        debug!("JWT_CLIENT_ID: {}", jwt_client_id);
        debug!("Token: {}", token_str);
    }

    // Detect which algorithm to use from the token
    let algorithm = match detect_algrithom(&token_str) {
        Ok(algorithm) => algorithm,
        Err(err) => {
            error!("Error: invalid algorithm, we only support HS256 and RS256.");
            debug!("Token: {}", token_str);
            return None;
        }
    };

    // Verify the token
    match algorithm {
        Algorithm::HS256 => {
            debug!("JWT_SECRET_KEY: {}", jwt_secret_key);

            match validate_token_with_hs256(&token_str, &jwt_secret_key) {
                Ok(user) => return Some(user),
                Err(err) => {
                    error!("Error: {}", err);
                    debug!("Token: {}", token_str);
                    return None;
                }
            }
        }
        Algorithm::RS256 => {
            let kid = match detect_kid(&token_str) {
                Some(kid) => kid,
                None => {
                    error!("Error: invalid kid.");
                    debug!("Token: {}, JWT_CLIENT_ID: {}", token_str, jwt_client_id);
                    return None;
                }
            };
            // Get JWKs from cache
            let jwks = match get_jwks_from_cache(&kid) {
                Some(jwks) => jwks,
                None => {
                    error!("Error: invalid jwks.");
                    debug!("Token: {}, JWT_CLIENT_ID: {}", token_str, jwt_client_id);
                    return None;
                }
            };

            debug!("JWKs: {:?}, kid: {}, token: {}", jwks, kid, token_str);

            let claims = validate_token_with_rs256(&jwt_client_id, &token_str, &jwks, &kid).await;
            match claims {
                Ok(claims) => {
                    // Get the username from the claims, the priority is: username > email > name
                    let username = match get_username_from_claims(&claims) {
                        Some(username) => username,
                        None => {
                            error!("No username/name/email field in the token.");
                            return None;
                        }
                    };

                    let email = &claims.email;

                    debug!("Claims: {:?}, username: {}", claims, username);

                    return Some(User::new(&username, email));
                }
                Err(err) => {
                    error!("Error: {}", err);
                    debug!("Token: {}", token_str);
                    None
                }
            }
        }
        _ => {
            error!("Error: invalid algorithm, we only support HS256 and RS256.");
            debug!("Token: {}", token_str);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::init_logger;
    use log::LevelFilter;

    #[tokio::test]
    async fn test_valid_token() {
        let _ = init_logger("biomedgps-test", LevelFilter::Debug);

        let token = "Please replace this token with your own token.";

        // Cache the public key
        let url = "https://biomedgps.jp.auth0.com/.well-known/jwks.json";
        let _ = match fetch_and_store_jwks(url).await {
            Ok(keys) => {
                info!("Fetch and store jwks successfully.");
                info!("{:?}", keys);

                assert!(keys.keys.len() > 0);
            }
            Err(err) => error!("Error: {}", err),
        };
        let client_id = "Y08FauV1dAEiocNIZt5LiOifzNgXr6Uo";
        let kid = detect_kid(token).unwrap();

        let jwks = get_jwks_from_cache(&kid).unwrap();
        let validated_claims = validate_token_with_rs256(client_id, token, &jwks, &kid)
            .await
            .unwrap();
        assert_eq!(validated_claims.name, "Craig Yang");
    }
}
