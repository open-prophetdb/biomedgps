use anyhow;
use log::{debug, info, warn};
use lol_html::element;
use lol_html::html_content::ContentType;
use lol_html::{rewrite_str, RewriteStrSettings};
use poem::http::header::{HeaderName as PoemHeaderName, HeaderValue as PoemHeaderValue};
use poem::http::Method;
use poem::{handler, http::StatusCode, web, IntoResponse, Request, Response, Result};
use reqwest::header::{
    HeaderMap as ReqwestHeaderMap, HeaderName as ReqwestHeaderName,
    HeaderValue as ReqwestHeaderValue, CONTENT_ENCODING, CONTENT_TYPE, HOST, TRANSFER_ENCODING,
};
use reqwest::Client;
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
            redirect_url: "http://localhost:3000/proxy-data/sanger_cosmic",
            // redirect_url: "https://drugs.3steps.cn/proxy-data/sanger_cosmic",
            target_url: "https://cancer.sanger.ac.uk/cosmic/gene/analysis?ln=",
            format_target_url: format_sanger_cosmic_target_url,
            which_tags: vec!["a", "link", "script", "img", "[data-url]", "table[title]"],
            additional_css: Some(".body { background-color: #fff; } #ccc { visibility: hidden; display: none; } .external > img { visibility: hidden; display: none; } #sidebar { visibility: hidden; display: none; } #section-list { margin-left: 0px; padding-top: 0px; } .dataTable { width: 100%; } .cosmic, .logo_grch38, .subhead, footer { visibility: hidden; display: none; } #section-genome-browser { visibility: hidden; display: none !important; }")
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
    pub additional_css: Option<&'static str>, // Additional CSS content that we want to append to the head tag.
}

impl Website {
    /// Modify the HTML content.
    ///
    /// # Arguments
    ///
    /// * `input` - The HTML content.
    /// * `tags` - The tags that we want to modify. Such as ["a", "link", "script", "img", "[data-url]", "table[title]"].
    /// * `css` - The CSS content that we want to append to the head tag.
    ///
    /// # Returns
    ///
    /// * The modified HTML content.
    /// * An error if the content cannot be modified.
    ///
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

        match self.additional_css {
            Some(css_str) => handlers.push(element!("head", move |el| {
                let style = &format!("<style>{}</style>", css_str);
                el.append(style, ContentType::Html);
                Ok(())
            })),
            _ => {}
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
    debug!("Proxy website: {:?}", req);
    let url = req.uri().path().to_string();
    let client = reqwest::Client::new();
    // Whether the url starts with "/proxy/".
    if !url.starts_with("/proxy/") {
        return Err(poem::Error::from_string(
            "Invalid URL, must set the URL prefix to /proxy/",
            StatusCode::BAD_REQUEST,
        ));
    };

    // Get the website name from the URL. The URL format is "/proxy/{website_name}".
    let website_name = match url.split('/').nth(2) {
        Some(name) => name,
        None => {
            return Err(poem::Error::from_string(
                "Invalid URL",
                StatusCode::BAD_REQUEST,
            ))
        }
    };

    info!("Proxy website: {}", website_name);
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

/// Transfer the request to the target website. All other pages from the first request will be transferred to the target website.
///
/// # Arguments
///
/// * `req` - The request.
/// * `body` - The request body.
///
/// # Returns
///
/// * The response.
///
#[handler]
pub async fn proxy_website_data(
    req: &Request,
    // body: web::Json<Option<JsonValue>>
) -> poem::Result<impl IntoResponse> {
    let method: Method = req.method().clone();
    let reqwest_method = match method {
        Method::GET => reqwest::Method::GET,
        Method::POST => reqwest::Method::POST,
        Method::PUT => reqwest::Method::PUT,
        Method::DELETE => reqwest::Method::DELETE,
        Method::PATCH => reqwest::Method::PATCH,
        Method::HEAD => reqwest::Method::HEAD,
        Method::OPTIONS => reqwest::Method::OPTIONS,
        Method::CONNECT => reqwest::Method::CONNECT,
        Method::TRACE => reqwest::Method::TRACE,
        _ => reqwest::Method::GET,
    };

    let uri = req.uri().clone();
    let path = uri.path().to_string();
    debug!("Proxy website data: {:?}", path);

    let proxy_prefix = "/proxy-data";
    let first_segment = get_first_segment(&path, Some(proxy_prefix));
    // Please note that the prefix must match the setting in the biomedgps router.
    let new_path = remove_prefix(&path, &format!("{}/{}", proxy_prefix, first_segment));
    debug!(
        "Transfer request to {} with method: {}",
        new_path, reqwest_method
    );

    let website = match WEBSITES.get(&first_segment.as_str()) {
        Some(website) => website,
        None => {
            return Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body("Website not found"));
        }
    };
    let base_url = Url::parse(website.base_url).unwrap();
    let url = base_url.join(&new_path).unwrap().to_string();

    let mut headers = ReqwestHeaderMap::new();
    for (name, value) in req.headers() {
        let reqwest_name = match ReqwestHeaderName::from_bytes(name.as_str().as_bytes()) {
            Ok(name) => name,
            Err(_) => continue,
        };
        let value = match ReqwestHeaderValue::from_bytes(value.as_bytes()) {
            Ok(value) => value,
            Err(_) => continue,
        };

        if reqwest_name != HOST {
            headers.insert(reqwest_name, value);
        }
    }
    headers.insert("Accept-Encoding", "gzip".parse().unwrap());

    let client = Client::new();
    let query = req.uri().query().unwrap_or_default();
    let url = if query.is_empty() {
        url
    } else {
        format!("{}?{}", url, query)
    };
    debug!(
        "Transfer request to {} with body and headers: {:?}",
        url, headers
    );
    // debug!("Transfer request to {} with body and headers: {:?} {:?}", url, body, headers);
    // let body_bytes = match body.0 {
    //     Some(body) => body.to_string().into_bytes(),
    //     None => Vec::new(),
    // };

    let res = client
        .request(reqwest_method, &url)
        .headers(headers)
        // .body(body_bytes)
        .send()
        .await
        .map_err(|e| poem::error::InternalServerError(e))?;

    let status_code =
        StatusCode::from_u16(res.status().as_u16()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let mut response = Response::builder().status(status_code);

    for (key, value) in res.headers().iter() {
        // Filter out 'Content-Encoding' and 'Transfer-Encoding' headers
        if let (Ok(header_name), Ok(header_value)) = (
            PoemHeaderName::try_from(key.as_str()),
            PoemHeaderValue::try_from(value.as_bytes()),
        ) {
            response = response.header(header_name, header_value);
        }
    }

    let body = res.bytes().await.map_err(|_| {
        poem::Error::from_string("Failed to read body", StatusCode::INTERNAL_SERVER_ERROR)
    })?;
    Ok(response.body(body))
}

fn remove_prefix(path: &str, prefix: &str) -> String {
    if path.starts_with(prefix) {
        path[prefix.len()..].to_string()
    } else {
        path.to_string()
    }
}

fn get_first_segment(path: &str, prefix: Option<&str>) -> String {
    if prefix.is_some() && path.starts_with(prefix.unwrap()) {
        let segments = path.split('/').collect::<Vec<&str>>();
        segments[2].to_string()
    } else {
        let segments = path.split('/').collect::<Vec<&str>>();
        segments[1].to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_modify_html() {
        let website = Website {
            name: "sanger_cosmic",
            base_url: "https://cancer.sanger.ac.uk",
            redirect_url: "https://omics-data.3steps.cn/sanger_cosmic",
            target_url: "https://cancer.sanger.ac.uk/cosmic/gene/analysis?ln=",
            format_target_url: format_sanger_cosmic_target_url,
            which_tags: vec!["a", "link", "script", "img", "[data-url]", "table[title]"],
            additional_css: Some(".test { color: red; }"),
        };
        let input = r#"<html>
<head>
<link rel="stylesheet" href="/style.css">
</head>
<div>
<a href="/about">About</a>
<a href="/contact">Contact</a>
<link rel="stylesheet" href="style.css">
<script src="/script.js"></script>
<img src="/logo.png">
<img src="/banner.jpg">
<div data-url="/page"></div>
<div data-url="https://example.com/page"></div>
<table title="/table-page"></table>
<table title="https://example.com/table-page"></table>
</div>
</html>
"#;

        let modified_html_v1 = match website.modify_html(input, website.which_tags.clone()) {
            Ok(modified_html) => modified_html,
            Err(e) => {
                eprintln!("Error: {}", e);
                return;
            }
        };

        assert_eq!(
            modified_html_v1,
            r#"<html>
<head>
<link rel="stylesheet" href="https://cancer.sanger.ac.uk/style.css">
<style>.test { color: red; }</style></head>
<div>
<a href="https://omics-data.3steps.cn/sanger_cosmic/about" target="_blank">About</a>
<a href="https://omics-data.3steps.cn/sanger_cosmic/contact" target="_blank">Contact</a>
<link rel="stylesheet" href="https://cancer.sanger.ac.uk/style.css">
<script src="https://cancer.sanger.ac.uk/script.js"></script>
<img src="https://cancer.sanger.ac.uk/logo.png">
<img src="https://cancer.sanger.ac.uk/banner.jpg">
<div data-url="https://omics-data.3steps.cn/sanger_cosmic/page"></div>
<div data-url="https://example.com/page"></div>
<table title="https://omics-data.3steps.cn/sanger_cosmic/table-page"></table>
<table title="https://example.com/table-page"></table>
</div>
</html>
"#
        );
    }
}
