use anyhow;
use log::{debug, info, warn};
use lol_html::html_content::ContentType;
use lol_html::{element, text, Selector};
use lol_html::{rewrite_str, RewriteStrSettings};
use poem::http::header::{HeaderName as PoemHeaderName, HeaderValue as PoemHeaderValue};
use poem::http::Method;
use poem::{handler, http::StatusCode, web, IntoResponse, Request, Response, Result};
use regex::Regex;
use reqwest::header::{
    HeaderMap as ReqwestHeaderMap, HeaderName as ReqwestHeaderName,
    HeaderValue as ReqwestHeaderValue, CONTENT_ENCODING, CONTENT_TYPE, HOST, TRANSFER_ENCODING,
};
use reqwest::Client;
use serde_json::Value as JsonValue;
use serde_urlencoded;
use std::borrow::Cow;
use std::collections::HashMap;
use url::Url;

pub const PROXY_PREFIX: &str = "/proxy";
pub const PROXY_DATA_PREFIX: &str = "/proxy-data";

fn get_sanger_cosmic_redirect_url(
    raw_url: &str,
    base_url: &str,
    enable_redirect: bool,
    upstream_host: &str,
    _tag_name: &str,
) -> String {
    let raw_redirect_url = format!("{}/{}/sanger_cosmic", upstream_host, PROXY_DATA_PREFIX);

    let url = Url::parse(base_url).unwrap();
    let resolved_url = url.join(raw_url).unwrap();
    let modified_url = if enable_redirect {
        resolved_url
            .to_string()
            .replace(base_url, &raw_redirect_url)
    } else {
        resolved_url.to_string()
    };

    modified_url
}

fn get_protein_atlas_redirect_url(
    raw_url: &str,
    base_url: &str,
    enable_redirect: bool,
    upstream_host: &str,
    _tag_name: &str,
) -> String {
    let host = Url::parse(upstream_host).unwrap();
    let raw_redirect_url = host
        .join(&format!("{}/protein_atlas", PROXY_DATA_PREFIX))
        .unwrap();

    // We want to open the link same as the proxy link, not a proxy-data link. So we can load it in the same iframe.
    // Such as <a href="/ENSG00000130234-ACE2/tissue" title="Tissue - Enhanced">
    if raw_url.starts_with("/ENSG") {
        let path = format!(
            "{}/protein_atlas/{}",
            PROXY_PREFIX,
            raw_url.strip_prefix("/").unwrap()
        );

        let resolved_url = host.join(&path).unwrap();
        return resolved_url.to_string();
    } else if raw_url.contains("images.proteinatlas.org") {
        let raw_url = raw_url
            .trim_start_matches("https://images.proteinatlas.org")
            .trim_start_matches("//images.proteinatlas.org")
            .strip_prefix("/")
            .unwrap();
        let url = format!("{}/protein_atlas/{}", PROXY_DATA_PREFIX, raw_url);
        let resolved_url = host.join(&url).unwrap();
        return format!(
            "{}?raw_base_url={}",
            resolved_url, "https://images.proteinatlas.org"
        );
    } else if raw_url.contains("humanproteome/proteinevidence") {
        let redirect_url = host
            .join(&format!(
                "{}/protein_atlas/{}",
                PROXY_PREFIX,
                raw_url.strip_prefix("/").unwrap()
            ))
            .unwrap();
        return redirect_url.to_string();
    } else {
        let url = Url::parse(base_url).unwrap();
        let resolved_url = url.join(raw_url).unwrap();
        if enable_redirect {
            return resolved_url
                .to_string()
                .replace(base_url, &raw_redirect_url.to_string());
        } else {
            return resolved_url.to_string();
        }
    }
}

fn get_rndsystems_redirect_url(
    raw_url: &str,
    base_url: &str,
    enable_redirect: bool,
    upstream_host: &str,
    tag_name: &str,
) -> String {
    if raw_url.starts_with("#") && tag_name == "a" {
        return raw_url.to_string();
    }

    let host = Url::parse(upstream_host).unwrap();
    let raw_redirect_url = host.join(&format!("{}/rndsystems", PROXY_PREFIX)).unwrap();

    let url = Url::parse(base_url).unwrap();
    let resolved_url = url.join(raw_url).unwrap();
    let modified_url = if enable_redirect {
        resolved_url
            .to_string()
            .replace(base_url, &raw_redirect_url.to_string())
    } else {
        resolved_url.to_string()
    };

    return modified_url;
}

lazy_static::lazy_static! {
    pub static ref WEBSITES: HashMap<&'static str, Website> = {
        let mut map = HashMap::new();
        map.insert("sanger_cosmic", Website {
            name: "sanger_cosmic",
            base_url: "https://cancer.sanger.ac.uk",
            get_redirect_url: get_sanger_cosmic_redirect_url,
            which_tags: vec!["a", "link", "script", "img", "[data-url]", "table[title]"],
            additional_css: Some("body { background-color: #fff !important; } #ccc { visibility: hidden; display: none; } .external > img { visibility: hidden; display: none; } #sidebar { visibility: hidden; display: none; } #section-list { margin-left: 0px; padding-top: 0px; } .dataTable { width: 100%; } .cosmic, .logo_grch38, .subhead, footer { visibility: hidden; display: none; } #section-genome-browser { visibility: hidden; display: none !important; }"),
            additional_js: Some("function addTargetAttribute() { const links = document.querySelectorAll('a'); links.forEach(link => { link.setAttribute('target', '_blank'); }); }; document.addEventListener('DOMContentLoaded', (event) => { addTargetAttribute(); }); setInterval(addTargetAttribute, 10000);"),
            enable_redirect: vec!["img[src]", "link[src]", "[data-url]", "table[title]"],
            open_at_new_tab: true,
        });

        map.insert("protein_atlas", Website {
            name: "protein_atlas",
            base_url: "https://www.proteinatlas.org",
            get_redirect_url: get_protein_atlas_redirect_url,
            which_tags: vec!["a", "link", "script", "img", "li[style]", "a[style]", "object[data]", "iframe"],
            additional_css: Some(".tabrow { padding-inline-start: unset !important; width: revert !important; } .page_header { visibility: hidden; display: none; } div.atlas_header, div.atlas_border { top: 0px !important; } table.main_table { top: 0px !important; margin: 0 0px 0 200px !important; }div.celltype_detail { width: unset !important; } table.menu_margin, div.menu_margin { margin: 0 !important; } div.page_footer { display: none; } div#cookie_statement { display: none; } div.atlas_header div.atlas_nav.show_small { top: 0px !important; } div.menu { top: 100px !important; left: 0px !important } div.atlas_header div.gene_name, div.page_header div.gene_name { left: -180px !important; width: 80px !important; } div.atlas_header div.atlas_nav { left: 180px !important; width: 1000px !important; margin: unset !important; padding-top: 0px !important; top: 0px !important; } #NGLViewer { display: none !important; } table.main_table td { padding: 5px !important; } body.general_body { background-image: none !important; } a { cursor: pointer !important; }"),
            additional_js: Some("const matchLists = { text: ['all genes', '特定文本2'], href: ['https://example.com', 'http://example.net'] }; function addTargetAttribute() { const links = document.querySelectorAll('a'); links.forEach(link => { if (matchLists.text.some(text => link.textContent.includes(text))) { link.setAttribute('target', '_blank'); } else if (matchLists.href.some(href => link.href === href)) { link.setAttribute('target', '_blank'); } }); }; document.addEventListener('DOMContentLoaded', (event) => { addTargetAttribute(); }); setInterval(addTargetAttribute, 10000); document.addEventListener('click', function(e) { const link = e.target.closest('a'); if (link) { window.parent.postMessage({ type: 'linkClicked', href: e.target.href }, '*');  } });"),
            enable_redirect: vec!["a[href]", "img[src]", "link[src]", "li[style]", "a[style]", "object[data]", "iframe"],
            open_at_new_tab: false,
        });

        map.insert("rndsystems", Website {
            name: "rndsystems",
            base_url: "https://www.rndsystems.com",
            get_redirect_url: get_rndsystems_redirect_url,
            which_tags: vec!["a", "link", "script", "img", "li[style]", "a[style]", "object[data]", "iframe"],
            additional_css: Some("#header, .breadcrumbs_wrapper, #search_facets, #footer_wrapper, #copyright_wrapper, .compare_tool_select, .search_products_area, #content_column, #sidebar, .search_results_top, .search_results_bottom, .helpButton { visibility: hidden !important; display: none !important; } #search_results { width: 100% !important; left: 0 !important; padding: 0 !important; } .main-container { width: auto !important; margin: 0px !important; }"),
            additional_js: Some("function addTargetAttribute() { const links = document.querySelectorAll('a'); links.forEach(link => { link.setAttribute('target', '_blank'); }); }; document.addEventListener('DOMContentLoaded', (event) => { addTargetAttribute(); }); setInterval(addTargetAttribute, 10000);"),
            enable_redirect: vec!["a[href]", "img[src]", "link[src]", "li[style]", "a[style]", "object[data]", "iframe"],
            open_at_new_tab: false,
        });

        map
    };
}

/// Description of a website that we want to proxy.
pub struct Website {
    pub name: &'static str,     // Name of the website, such as "sanger_cosmic".
    pub base_url: &'static str, // Base URL, such as "https://cancer.sanger.ac.uk". We use this URL to replace with the redirect URL.
    pub get_redirect_url: fn(&str, &str, bool, &str, &str) -> String, // Get the redirect URL. The first parameter is the raw URL (It might be a relative URL or an absolute URL), the second parameter is the base URL, the third parameter is whether to enable the redirect for the specific URL, the fourth parameter is the upstream host, and the fifth parameter is the tag name.
    pub which_tags: Vec<&'static str>, // Which tag to modify. Such as ["a", "link", "script", "img", "[data-url]", "table[title]"]. If you want to know which tags are supported, you can check the modify_html function in the Website struct.
    pub additional_css: Option<&'static str>, // Additional CSS content that we want to append to the head tag.
    pub additional_js: Option<&'static str>, // Additional JS content that we want to append to the body tag.
    pub enable_redirect: Vec<&'static str>, // Enable redirect for the specific URL. Such as "https://cancer.sanger.ac.uk" => "https://omics-data.3steps.cn/sanger_cosmic". It supports the a[href], link[href], script[src], img[src], [data-url], table[title] attributes. Some links are not necessary to redirect, such as the css, js, and other static files.
    pub open_at_new_tab: bool,              // Open the link at a new tab.
}

impl Website {
    pub fn format_target_url(&self, path: &str) -> String {
        let target_url = self.base_url.to_string();
        let target_url = Url::parse(&target_url).unwrap();
        let target_url = target_url.join(path).unwrap();
        target_url.to_string()
    }

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
    pub fn modify_html(
        &self,
        input: &str,
        tags: Vec<&str>,
        upstream_host: &str,
    ) -> Result<String, anyhow::Error> {
        let website_baseurl = self.base_url;
        let mut handlers = Vec::new();

        for tag in tags {
            match tag {
                "a" => {
                    handlers.push(element!("a[href]", |el| {
                        let href = el
                            .get_attribute("href")
                            .ok_or_else(|| anyhow::Error::msg("Missing href attribute"))?;
                        let modified_href = (self.get_redirect_url)(
                            &href,
                            website_baseurl,
                            self.enable_redirect.contains(&"a[href]"),
                            upstream_host,
                            "a"
                        );
                        el.set_attribute("href", &modified_href)?;

                        if self.open_at_new_tab {
                            el.set_attribute("target", "_blank")?;
                        }

                        Ok(())
                    }));
                }
                "link" => {
                    handlers.push(element!("link[href]", |el| {
                        let href = el
                            .get_attribute("href")
                            .ok_or_else(|| anyhow::Error::msg("Missing href attribute"))?;
                        let modified_url = (self.get_redirect_url)(
                            &href,
                            website_baseurl,
                            self.enable_redirect.contains(&"link[href]"),
                            upstream_host,
                            "link"
                        );
                        el.set_attribute("href", &modified_url)?;
                        Ok(())
                    }));
                }
                "script" | "img" => {
                    handlers.push(element!(format!("{}[src]", tag), |el| {
                        let src = el
                            .get_attribute("src")
                            .ok_or_else(|| anyhow::Error::msg("Missing src attribute"))?;
                        let tag_attr = format!("{}[src]", tag.to_string());
                        let modified_url = (self.get_redirect_url)(
                            &src,
                            website_baseurl,
                            self.enable_redirect.contains(&tag_attr.as_str()),
                            upstream_host,
                            tag
                        );
                        el.set_attribute("src", &modified_url)?;
                        Ok(())
                    }));
                }
                "script[content]" => {
                    handlers.push(text!("script", |text| {
                        let script_content = text.as_str().to_string();
                        let modified_content = (self.get_redirect_url)(
                            &script_content,
                            website_baseurl,
                            self.enable_redirect.contains(&"script[content]"),
                            upstream_host,
                            "script[content]"
                        );
                        text.replace(&modified_content, ContentType::Html);
                        Ok(())
                    }));
                }
                "[data-url]" => {
                    handlers.push(element!("[data-url]", |el| {
                        let data_url = el
                            .get_attribute("data-url")
                            .ok_or_else(|| anyhow::Error::msg("Missing data-url attribute"))?;
                        let modified_url = (self.get_redirect_url)(
                            &data_url,
                            website_baseurl,
                            self.enable_redirect.contains(&"[data-url]"),
                            upstream_host,
                            "[data-url]"
                        );
                        el.set_attribute("data-url", &modified_url)?;
                        Ok(())
                    }));
                }
                "table[title]" => {
                    handlers.push(element!("table[title]", |el| {
                        let title = el
                            .get_attribute("title")
                            .ok_or_else(|| anyhow::Error::msg("Missing title attribute"))?;
                        let modified_url = (self.get_redirect_url)(
                            &title,
                            website_baseurl,
                            self.enable_redirect.contains(&"table[title]"),
                            upstream_host,
                            "table[title]"
                        );
                        el.set_attribute("title", &modified_url)?;
                        Ok(())
                    }));
                }
                "iframe" => {
                    handlers.push(element!("iframe", |el| {
                        let src = el
                            .get_attribute("src")
                            .ok_or_else(|| anyhow::Error::msg("Missing src attribute"))?;
                        let modified_url = (self.get_redirect_url)(
                            &src,
                            website_baseurl,
                            self.enable_redirect.contains(&"iframe"),
                            upstream_host,
                            "iframe"
                        );
                        el.set_attribute("src", &modified_url)?;
                        Ok(())
                    }));
                }
                "object[data]" => {
                    handlers.push(element!("object[data]", |el| {
                        let object = el
                            .get_attribute("data")
                            .ok_or_else(|| anyhow::Error::msg("Missing data attribute"))?;
                        let modified_url = (self.get_redirect_url)(
                            &object,
                            website_baseurl,
                            self.enable_redirect.contains(&"object[data]"),
                            upstream_host,
                            "object[data]"
                        );
                        el.set_attribute("data", &modified_url)?;
                        Ok(())
                    }));
                }
                "li[style]" | "a[style]" => handlers.push(element!(tag, |el| {
                    let tag = tag.to_owned();
                    if let Some(style_attr) = el.get_attribute("style") {
                        info!("li/a Style: {}", style_attr);
                        let new_style = style_attr
                            .split(";")
                            .map(|style| {
                                // Remove the leading and trailing whitespace and quotes.
                                let style = style.trim();
                                if style.starts_with("background-image:url(") {
                                    let re = Regex::new(r"url\([']?(.*?)[']?\)").unwrap();

                                    let modified_style =
                                        re.replace_all(style, |caps: &regex::Captures| {
                                            let url = &caps[1];

                                            let modified_url = (self.get_redirect_url)(
                                                url,
                                                website_baseurl,
                                                self.enable_redirect.contains(&tag.as_str()),
                                                upstream_host,
                                                &tag
                                            );
                                            format!("url('{}')", modified_url)
                                        });

                                    Cow::Owned(modified_style.to_string())
                                } else {
                                    Cow::Borrowed(style)
                                }
                            })
                            .collect::<Vec<Cow<'_, str>>>()
                            .join(";");

                        info!("New style: {}", new_style);
                        el.set_attribute("style", &new_style)?;
                    }
                    Ok(())
                })),
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

        match self.additional_js {
            Some(js_str) => handlers.push(element!("body", move |el| {
                let script = &format!("<script>{}</script>", js_str);
                el.append(script, ContentType::Html);
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
    debug!("Proxy website - req: {:?}", req);
    let url = req.uri().path().to_string();
    let headers = req.headers();
    info!("Proxy website - headers: {:?}", headers);
    let proto_header = match headers.get("x-forwarded-proto") {
        Some(proto) => proto,
        None => {
            return Err(poem::Error::from_string(
                "X-Forwarded-Proto not found, please set the X-Forwarded-Proto header.",
                StatusCode::BAD_REQUEST,
            ))
        }
    };
    let upstream_host = match headers.get("host") {
        Some(host) => match host.to_str() {
            Ok(host) => {
                let proto = proto_header.to_str().unwrap_or("http");
                format!("{}://{}", proto, host)
            }
            Err(_) => {
                return Err(poem::Error::from_string(
                    "Invalid host, please set or check the host header.",
                    StatusCode::BAD_REQUEST,
                ))
            }
        },
        None => {
            return Err(poem::Error::from_string(
                "Host not found, please set the host header.",
                StatusCode::BAD_REQUEST,
            ))
        }
    };
    info!("Proxy website - upstream_host: {}", upstream_host);

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

    let remaing_path = url.split('/').skip(3).collect::<Vec<&str>>().join("/");

    info!("Proxy website: {}, {}, {}", url, website_name, remaing_path);
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

    let target_url = website.format_target_url(&remaing_path);

    let response = client
        .get(target_url)
        .query(&query_params)
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
            let modified_html =
                match website.modify_html(&html, website.which_tags.clone(), &upstream_host) {
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
    info!("Proxy website data: {}, {:?}", uri, path);

    let first_segment = get_first_segment(&path, Some(PROXY_DATA_PREFIX));
    // Please note that the prefix must match the setting in the biomedgps router.
    let new_path = remove_prefix(&path, &format!("{}/{}", PROXY_DATA_PREFIX, first_segment));
    info!(
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
        let query_obj = serde_urlencoded::from_str::<HashMap<String, String>>(query).unwrap();
        let raw_base_url = match query_obj.get("raw_base_url") {
            Some(raw_base_url) => raw_base_url.to_string(),
            None => website.base_url.to_string(),
        };
        let base_url = Url::parse(&raw_base_url).unwrap();
        let url = base_url.join(&new_path).unwrap().to_string();
        format!("{}?{}", url, query)
    };
    info!(
        "Transfer request to {} with body and headers: {:?}",
        url, headers
    );

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
        let host = Url::parse("https://drugs.3steps.cn").unwrap();
        let raw_url = "https://images.proteinatlas.org/123456/789012.png"
            .trim_start_matches("https://images.proteinatlas.org")
            .trim_start_matches("//images.proteinatlas.org");
        assert_eq!(raw_url, "/123456/789012.png");
        let url = format!("{}/protein_atlas/{}", PROXY_PREFIX, raw_url);
        let resolved_url = host.join(&url).unwrap();
        assert_eq!(
            resolved_url.to_string(),
            "https://drugs.3steps.cn/proxy/protein_atlas/123456/789012.png"
        );

        let website = Website {
            name: "sanger_cosmic",
            base_url: "https://cancer.sanger.ac.uk",
            get_redirect_url: get_sanger_cosmic_redirect_url,
            which_tags: vec!["a", "link", "script", "img", "[data-url]", "table[title]"],
            additional_css: Some(".test { color: red; }"),
            additional_js: None,
            enable_redirect: vec![
                "a[href]",
                // "img[src]",
                // "script[src]",
                "link[src]",
                "[data-url]",
                "table[title]",
            ],
            open_at_new_tab: true,
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

        let modified_html_v1 =
            match website.modify_html(input, website.which_tags.clone(), "localhost:3000") {
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
<a href="/proxy-data/sanger_cosmic/about" target="_blank">About</a>
<a href="/proxy-data/sanger_cosmic/contact" target="_blank">Contact</a>
<link rel="stylesheet" href="https://cancer.sanger.ac.uk/style.css">
<script src="https://cancer.sanger.ac.uk/script.js"></script>
<img src="https://cancer.sanger.ac.uk/logo.png">
<img src="https://cancer.sanger.ac.uk/banner.jpg">
<div data-url="/proxy-data/sanger_cosmic/page"></div>
<div data-url="https://example.com/page"></div>
<table title="/proxy-data/sanger_cosmic/table-page"></table>
<table title="https://example.com/table-page"></table>
</div>
</html>
"#
        );
    }
}
