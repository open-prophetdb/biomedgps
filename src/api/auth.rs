use hmac::{Hmac, Mac};
use jwt::VerifyWithKey;
use log::{debug, error, info, warn};
use poem::Request;
use poem_openapi::auth::Bearer;
use poem_openapi::SecurityScheme;
use serde_json::Value;
use sha2::Sha256;
use std::collections::BTreeMap;

pub const USERNAME_PLACEHOLDER: &str = "ANONYMOUS-USER-PLACEHOLDER";

#[derive(Debug)]
pub struct User {
    pub username: String,
    pub organizations: Vec<i32>,
    pub projects: Vec<i32>
}

impl User {
    fn new(username: String) -> Self {
        Self { 
            username,
            organizations: vec![-1],
            projects: vec![-1]
        }
    }

    fn add_organizations(&mut self, organizations: Vec<i32>) {
        self.organizations = organizations;
    }

    fn add_projects(&mut self, projects: Vec<i32>) {
        self.projects = projects;
    }
}

#[derive(SecurityScheme)]
#[oai(type = "bearer", checker = "jwt_token_checker")]
pub struct CustomSecurityScheme(pub User);

async fn jwt_token_checker(_: &Request, bearer: Bearer) -> Option<User> {
    // Get jwt_secret_key from environment variable
    let default_user = Some(User::new(USERNAME_PLACEHOLDER.to_string()));
    let jwt_secret_key = match std::env::var("JWT_SECRET_KEY") {
        Ok(key) => {
            if key.is_empty() {
                warn!("You don't set JWT_SECRET_KEY environment variable, so we will skip JWT verification, but users also need to set the Authorization header to access the API.");
                return default_user;
            }
            key
        }
        Err(_) => return default_user,
    };

    debug!("JWT_SECRET_KEY: {}", jwt_secret_key);

    let key: Hmac<Sha256> = Hmac::new_from_slice(jwt_secret_key.as_bytes()).unwrap();
    let token_str = bearer.token;
    let claims: BTreeMap<String, Value> = match token_str.verify_with_key(&key) {
        Ok(claims) => claims,
        Err(err) => {
            error!("Error: {}", err);
            return None;
        }
    };

    let username = match claims.get("username").and_then(Value::as_str) {
        Some(username) => username,
        None => {
            error!("Error: {}", "cannot find username field in claims.");
            return None;
        }
    };

    let organizations = match claims.get("organizations").and_then(Value::as_array) {
        Some(organizations) => organizations
            .iter()
            .filter_map(|org| org.as_i64())
            .map(|org| org as i32)
            .collect::<Vec<i32>>(),
        None => {
            // Be compatible with the old version, the token might not contain the organizations field.
            vec![-1]
        }
    };

    let projects = match claims.get("projects").and_then(Value::as_array) {
        Some(projects) => projects
            .iter()
            .filter_map(|project| project.as_i64())
            .map(|project| project as i32)
            .collect::<Vec<i32>>(),
        None => {
            // Be compatible with the old version, the token might not contain the projects field.
            vec![-1]
        }
    };

    let mut current_user = User::new(username.to_string());
    current_user.add_organizations(organizations);
    current_user.add_projects(projects);

    info!("current_user: {:?}", current_user);

    Some(current_user)
}
