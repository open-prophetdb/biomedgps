use anyhow;
use poem_openapi::Object;
use reqwest;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Object)]
pub struct PublicationRecords {
    pub records: Vec<Publication>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Object)]
pub struct Publication {
    pub authors: Vec<String>,
    pub citation_count: Option<u64>,
    pub summary: String,
    pub journal: String,
    pub title: String,
    pub year: Option<u64>,
    pub doc_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Object)]
pub struct PublicationDetail {
    pub authors: Vec<String>,
    pub citation_count: Option<u64>,
    pub summary: String,
    pub journal: String,
    pub title: String,
    pub year: Option<u64>,
    pub doc_id: String,
    pub article_abstract: Option<String>,
    pub doi: Option<String>,
    pub provider_url: Option<String>,
}

impl Publication {
    pub async fn fetch_publication(id: &str) -> Result<PublicationDetail, anyhow::Error> {
        let api_token = match std::env::var("GUIDESCOPER_API_TOKEN") {
            Ok(token) => token,
            Err(_) => {
                return Err(anyhow::anyhow!("GUIDESCOPER_API_TOKEN not found"));
            }
        };

        let detail_api = match std::env::var("GUIDESCOPER_DETAIL_API") {
            Ok(token) => token,
            Err(_) => {
                return Err(anyhow::anyhow!("GUIDESCOPER_DETAIL_API not found"));
            }
        };

        let url = format!("{}{}", detail_api, id);
        let cookie = format!("_session={}", api_token);
        let client = reqwest::Client::new();
        let res = client.get(&url).header("Cookie", cookie).send().await?;

        if res.status().is_success() {
            let body = res.text().await?;
            let json: serde_json::Value = serde_json::from_str(&body)?;
            let authors = json["authors"].as_array().unwrap();
            let mut authors_vec = Vec::new();
            for author in authors {
                authors_vec.push(author.as_str().unwrap().to_string());
            }
            let citation_count = json["citation_count"].as_u64();
            let summary = json["abstract_takeaway"].as_str().unwrap().to_string();
            // Such as { "journal": { "title": "The Journal of biological chemistry","scimago_quartile": 1 }}
            let journal = json["journal"]["title"].as_str().unwrap().to_string();
            let title = json["title"].as_str().unwrap().to_string();
            let year = json["year"].as_u64();
            let doc_id = json["id"].as_str().unwrap().to_string();
            let article_abstract = json["abstract"].as_str().map(|s| s.to_string());
            let doi = json["doi"].as_str().map(|s| s.to_string());
            let provider_url = json["provider_url"].as_str().map(|s| s.to_string());

            Ok(PublicationDetail {
                authors: authors_vec,
                citation_count: citation_count,
                summary: summary,
                journal: journal,
                title: title,
                year: year,
                doc_id: doc_id,
                article_abstract: article_abstract,
                doi: doi,
                provider_url: provider_url,
            })
        } else {
            Err(anyhow::anyhow!("Failed to fetch publication"))
        }
    }

    pub async fn fetch_publications(
        query_str: &str,
        page: Option<u64>,
        page_size: Option<u64>,
    ) -> Result<PublicationRecords, anyhow::Error> {
        let api_token = match std::env::var("GUIDESCOPER_API_TOKEN") {
            Ok(token) => token,
            Err(_) => {
                return Err(anyhow::anyhow!("GUIDESCOPER_API_TOKEN not found"));
            }
        };

        let guidescoper_api = match std::env::var("GUIDESCOPER_API") {
            Ok(token) => token,
            Err(_) => {
                return Err(anyhow::anyhow!("GUIDESCOPER_API not found"));
            }
        };

        // We only need to fetch the top 10 results currently.
        let total = 10;
        let page = 0;
        let page_size = 10;

        let mut records = Vec::new();
        let url = format!(
            "{}?query={}&page={}&size={}",
            guidescoper_api, query_str, page, page_size
        );
        let cookie = format!("_session={}", api_token);
        let client = reqwest::Client::new();
        let res = client.get(&url).header("Cookie", cookie).send().await?;

        if res.status().is_success() {
            let body = res.text().await?;
            let json: serde_json::Value = serde_json::from_str(&body)?;
            let items = json["papers"].as_array().unwrap();
            for item in items {
                let authors = item["authors"].as_array().unwrap();
                let mut authors_vec = Vec::new();
                for author in authors {
                    authors_vec.push(author.as_str().unwrap().to_string());
                }
                let citation_count = item["citation_count"].as_u64();
                let summary = item["display_text"].as_str().unwrap().to_string();
                let journal = item["journal"].as_str().unwrap().to_string();
                let title = item["title"].as_str().unwrap().to_string();
                let year = item["year"].as_u64();
                let doc_id = item["doc_id"].as_str().unwrap().to_string();
                records.push(Publication {
                    authors: authors_vec,
                    citation_count: citation_count,
                    summary: summary,
                    journal: journal,
                    title: title,
                    year: year,
                    doc_id: doc_id,
                });
            }
        }

        Ok(PublicationRecords {
            records: records,
            total: total,
            page: page,
            page_size: page_size,
        })
    }
}
