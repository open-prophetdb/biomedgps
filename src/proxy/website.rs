use anyhow;
use log::warn;
use lol_html::element;
use lol_html::{rewrite_str, RewriteStrSettings};
use poem::{handler, http::StatusCode, Endpoint, Request, Response, Result, Route};
use reqwest::header::CONTENT_TYPE;
use serde_json::Value as JsonValue;
use serde_urlencoded;
use std::collections::HashMap;
use url::Url;

/// Format the target URL for the Sanger COSMIC website.
///
/// # Arguments
///
/// * `target_url` - The target URL template.
/// * `value` - The query parameters. We use the gene_symbol to format the target URL.
///
/// # Returns
///
/// * The formatted target URL.
/// * An error if the gene_symbol is missing.
///
pub fn format_sanger_cosmic_target_url(
    target_url: &str,
    value: &JsonValue,
) -> Result<String, anyhow::Error> {
    let gene_symbol = match value.get("gene_symbol").map(|v| v.as_str()) {
        Some(Some(gene_symbol)) => gene_symbol,
        _ => {
            return Err(anyhow::anyhow!("Missing gene_symbol"));
        }
    };

    Ok(format!(
        "{target_url}{gene_symbol}",
        target_url = target_url,
        gene_symbol = gene_symbol
    ))
}

lazy_static::lazy_static! {
    pub static ref WEBSITES: HashMap<&'static str, Website> = {
        let mut map = HashMap::new();
        map.insert("sanger_cosmic", Website {
            name: "sanger_cosmic",
            base_url: "https://cancer.sanger.ac.uk",
            redirect_url: "https://omics-data.3steps.cn/sanger_cosmic",
            target_url: "https://cancer.sanger.ac.uk/cosmic/gene/analysis?ln=",
            format_target_url: format_sanger_cosmic_target_url,
            which_tags: vec!["a", "link", "script", "img", "[data-url]", "table[title]"],
        });

        map
    };
}

/// Description of a website that we want to proxy.
pub struct Website {
    pub name: &'static str,     // Name of the website, such as "sanger_cosmic".
    pub base_url: &'static str, // Base URL, such as "https://cancer.sanger.ac.uk". We use this URL to replace with the redirect URL.
    pub redirect_url: &'static str, // Redirect URL, such as "https://omics-data.3steps.cn/sanger_cosmic". The last part of the URL is the same as the name.
    pub target_url: &'static str, // Target URL, such as ""https://cancer.sanger.ac.uk/cosmic/gene/analysis?ln="". We use this URL to fetch the content that we want to proxy.
    pub format_target_url: fn(target_url: &str, value: &JsonValue) -> Result<String, anyhow::Error>, // Function to format the target URL. We use this function to format the target URL with the query parameters. NOTE: At most times, we only need a url and get method to fetch the content.
    pub which_tags: Vec<&'static str>, // Which tag to modify. Such as ["a", "link", "script", "img", "[data-url]", "table[title]"]. If you want to know which tags are supported, you can check the modify_html function in the Website struct.
}

impl Website {
    pub fn modify_html(&self, input: &str, tags: Vec<&str>) -> Result<String, anyhow::Error> {
        let website_baseurl = self.base_url;
        let redirect_url = self.redirect_url;
        let mut handlers = Vec::new();

        for tag in tags {
            match tag {
                "a" => {
                    handlers.push(element!("a[href]", |el| {
                        let href = el
                            .get_attribute("href")
                            .ok_or_else(|| anyhow::Error::msg("Missing href attribute"))?;
                        let base_url = Url::parse(website_baseurl)?;
                        let resolved_url = base_url.join(&href)?;
                        let modified_href = resolved_url
                            .to_string()
                            .replace(website_baseurl, redirect_url);
                        el.set_attribute("href", &modified_href)?;
                        el.set_attribute("target", "_blank")?;
                        Ok(())
                    }));
                }
                "link" => {
                    handlers.push(element!("link[href]", |el| {
                        let href = el
                            .get_attribute("href")
                            .ok_or_else(|| anyhow::Error::msg("Missing href attribute"))?;
                        let base_url = Url::parse(website_baseurl)?;
                        let resolved_url = base_url.join(&href)?;
                        el.set_attribute("href", &resolved_url.to_string())?;
                        Ok(())
                    }));
                }
                "script" | "img" => {
                    handlers.push(element!(format!("{}[src]", tag), |el| {
                        let src = el
                            .get_attribute("src")
                            .ok_or_else(|| anyhow::Error::msg("Missing src attribute"))?;
                        let base_url = Url::parse(website_baseurl)?;
                        let resolved_url = base_url.join(&src)?;
                        el.set_attribute("src", &resolved_url.to_string())?;
                        Ok(())
                    }));
                }
                "[data-url]" => {
                    handlers.push(element!("[data-url]", |el| {
                        let data_url = el
                            .get_attribute("data-url")
                            .ok_or_else(|| anyhow::Error::msg("Missing data-url attribute"))?;
                        let base_url = Url::parse(website_baseurl)?;
                        let resolved_url = base_url.join(&data_url)?;
                        let modified_url = resolved_url
                            .to_string()
                            .replace(website_baseurl, redirect_url);
                        el.set_attribute("data-url", &modified_url)?;
                        Ok(())
                    }));
                }
                "table[title]" => {
                    handlers.push(element!("table[title]", |el| {
                        let title = el
                            .get_attribute("title")
                            .ok_or_else(|| anyhow::Error::msg("Missing title attribute"))?;
                        let base_url = Url::parse(website_baseurl)?;
                        let resolved_url = base_url.join(&title)?;
                        let modified_url = resolved_url
                            .to_string()
                            .replace(website_baseurl, redirect_url);
                        el.set_attribute("title", &modified_url)?;
                        Ok(())
                    }));
                }
                _ => {
                    warn!("Unknown tag: {}", tag);
                }
            }
        }

        let modified_html = rewrite_str(
            input,
            RewriteStrSettings {
                element_content_handlers: handlers,
                ..RewriteStrSettings::default()
            },
        )?;

        Ok(modified_html)
    }
}

/// Proxy the website.
///
/// # Arguments
///
/// * `req` - The request.
///
/// # Returns
///
/// * The response.
///
/// # Errors
///
/// * If the URL is invalid.
/// * If the website is not found.
/// * If the target URL is invalid.
/// * If the content type is not supported.
/// * If the content cannot be fetched.
/// * If the content cannot be modified.
/// * If an unknown error occurs.
///
/// # Example
///
/// ```no_run
/// use poem::web::proxy_website;
/// use poem::http::Method;
/// use poem::Request;
///
/// #[tokio::main]
/// async fn main() {
///    let req = Request::builder()
///       .method(Method::GET)
///       .uri("http://localhost:3000/proxy/sanger_cosmic?gene_symbol=TP53")
///       .finish();
///    let resp = proxy_website(&req).await.unwrap();
///    assert_eq!(resp.status(), 200);
/// }
/// ```
#[handler]
pub async fn proxy_website(req: &Request) -> Result<Response> {
    let url = req
        .uri()
        .path_and_query()
        .map(|x| x.as_str())
        .unwrap_or_default();
    let client = reqwest::Client::new();
    let website_name = match url.split('/').nth(1) {
        Some(name) => name,
        None => {
            return Err(poem::Error::from_string(
                "Invalid URL",
                StatusCode::BAD_REQUEST,
            ))
        }
    };

    let website = match WEBSITES.get(website_name) {
        Some(website) => website,
        None => {
            return Err(poem::Error::from_string(
                "Website not found",
                StatusCode::NOT_FOUND,
            ))
        }
    };

    let query_params: JsonValue = req
        .uri()
        .query()
        .map(|query| serde_urlencoded::from_str(query).unwrap_or_default())
        .unwrap_or_default();

    let target_url = match (website.format_target_url)(website.target_url, &query_params) {
        Ok(target_url) => target_url,
        Err(e) => {
            return Err(poem::Error::from_string(
                e.to_string(),
                StatusCode::BAD_REQUEST,
            ))
        }
    };

    let response = client
        .get(target_url)
        .send()
        .await
        .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_GATEWAY))?;

    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .map(|v| v.to_str().unwrap_or_default())
        .unwrap_or_default();

    match content_type {
        ct if ct.contains("application/json") => {
            let json: JsonValue = response
                .json()
                .await
                .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_GATEWAY))?;
            Ok(Response::builder()
                .status(StatusCode::OK)
                .content_type("application/json")
                .body(json.to_string()))
        }
        ct if ct.contains("text/html") => {
            let html = response
                .text()
                .await
                .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_GATEWAY))?;
            let modified_html = match website.modify_html(&html, website.which_tags.clone()) {
                Ok(modified_html) => modified_html,
                Err(e) => {
                    return Err(poem::Error::from_string(
                        e.to_string(),
                        StatusCode::INTERNAL_SERVER_ERROR,
                    ))
                }
            };

            Ok(Response::builder()
                .status(StatusCode::OK)
                .content_type("text/html")
                .body(modified_html))
        }
        _ => {
            // 对于其他类型的响应，可以根据需要调整
            Err(poem::Error::from_string(
                "Unsupported Content-Type",
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
            ))
        }
    }
}
