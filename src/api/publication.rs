use anyhow;
use log::info;
use poem_openapi::Object;
use reqwest;
use serde::{Deserialize, Serialize};
use urlencoding;

const GUIDESCOPER_PUBLICATIONS_API: &str = "/api/paper_search/";
const GUIDESCOPER_DETAILS_API: &str = "/api/papers/details/";
const GUIDESCOPER_SUMMARY_API: &str = "/api/summary/?search_id=";
const GUIDESCOPER_CONSENSUS_API: &str = "/api/yes_no/?search_id=";
const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/92.0.4515.131 Safari/537.36";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Object)]
pub struct PublicationRecords {
    pub records: Vec<Publication>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
    pub search_id: Option<String>,
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
    pub article_abstract: Option<String>,
    pub doi: Option<String>,
    pub provider_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Object)]
pub struct PublicationsSummary {
    pub summary: String,
    pub daily_limit_reached: bool,
    pub is_disputed: bool,
    pub is_incomplete: bool,
    pub results_analyzed_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object)]
pub struct ConsensusResult {
    pub results_analyzed_count: u64,
    pub yes_percent: f64,
    pub no_percent: f64,
    pub possibly_percent: f64,
    pub yes_doc_ids: Vec<String>,
    pub no_doc_ids: Vec<String>,
    pub possibly_doc_ids: Vec<String>,
    pub is_incomplete: bool,
    pub is_disputed: bool,
}

impl Publication {
    pub async fn fetch_publication(id: &str) -> Result<Publication, anyhow::Error> {
        let api_token = match std::env::var("GUIDESCOPER_API_TOKEN") {
            Ok(token) => token,
            Err(_) => {
                return Err(anyhow::anyhow!("GUIDESCOPER_API_TOKEN not found"));
            }
        };

        let guidescoper_server = match std::env::var("GUIDESCOPER_SERVER") {
            Ok(token) => token,
            Err(_) => {
                return Err(anyhow::anyhow!("GUIDESCOPER_SERVER not found"));
            }
        };

        let detail_api = format!("{}{}", guidescoper_server, GUIDESCOPER_DETAILS_API);
        info!("detail_api: {}", detail_api);

        let url = format!("{}{}", detail_api, id);
        let cookie = format!("_session={}", api_token);
        let client = reqwest::Client::new();
        let res = client
            .get(&url)
            .header("Cookie", cookie)
            .header("USER_AGENT", USER_AGENT)
            .send()
            .await?;

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

            Ok(Publication {
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

        let guidescoper_server = match std::env::var("GUIDESCOPER_SERVER") {
            Ok(token) => token,
            Err(_) => {
                return Err(anyhow::anyhow!("GUIDESCOPER_SERVER not found"));
            }
        };

        let guidescoper_api = format!("{}{}", guidescoper_server, GUIDESCOPER_PUBLICATIONS_API);
        info!("guidescoper_api: {}", guidescoper_api);

        // We only need to fetch the top 10 results currently.
        let page = page.unwrap_or(1);
        let page_size = page_size.unwrap_or(10);
        let mut total = page_size;

        let mut records = Vec::new();
        let encoded_query_str = urlencoding::encode(query_str);
        let url = format!(
            "{}?query={}&page={}&size={}",
            guidescoper_api, encoded_query_str, page, page_size
        );
        info!("Query url: {}", url);
        let cookie = format!("_session={}", api_token);
        let client = reqwest::Client::new();
        let res = client
            .get(&url)
            .header("Cookie", cookie)
            .header("USER_AGENT", USER_AGENT)
            .send()
            .await?;

        let mut search_id = String::new();

        if res.status().is_success() {
            let body = res.text().await?;
            let json: serde_json::Value = serde_json::from_str(&body)?;
            search_id = json["search_id"].as_str().unwrap().to_string();
            total = json["numTopResults"].as_u64().unwrap();
            // TODO: do we need to add the adjusted query into the response? It seems not necessary?
            // let query_str = json["adjustedQuery"].as_str().unwrap().to_string();
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
                let doi_id = item["doi"].as_str().unwrap().to_string();

                records.push(Publication {
                    authors: authors_vec,
                    citation_count: citation_count,
                    summary: summary,
                    journal: journal,
                    title: title,
                    year: year,
                    doc_id: doc_id,
                    article_abstract: None,
                    doi: Some(doi_id),
                    provider_url: None,
                });
            }
        } else {
            let err_msg = format!("Failed to fetch publications: {}", res.text().await?);
            return Err(anyhow::anyhow!(err_msg));
        }

        Ok(PublicationRecords {
            records: records,
            total: total,
            page: page,
            page_size: page_size,
            search_id: Some(search_id),
        })
    }

    pub async fn fetch_summary(search_id: &str) -> Result<PublicationsSummary, anyhow::Error> {
        let api_token = match std::env::var("GUIDESCOPER_API_TOKEN") {
            Ok(token) => token,
            Err(_) => {
                return Err(anyhow::anyhow!("GUIDESCOPER_API_TOKEN not found"));
            }
        };

        let guidescoper_server = match std::env::var("GUIDESCOPER_SERVER") {
            Ok(token) => token,
            Err(_) => {
                return Err(anyhow::anyhow!("GUIDESCOPER_SERVER not found"));
            }
        };

        let summary_api = format!("{}{}", guidescoper_server, GUIDESCOPER_SUMMARY_API);

        let url = format!("{}{}", summary_api, search_id);
        let cookie = format!("_session={}", api_token);
        let client = reqwest::Client::new();
        let res = client
            .get(&url)
            .header("Cookie", cookie)
            .header("USER_AGENT", USER_AGENT)
            .send()
            .await?;

        if res.status().is_success() {
            let body = res.text().await?;
            let json: serde_json::Value = serde_json::from_str(&body)?;
            let summary = json["summary"].as_str().unwrap().to_string();
            let daily_limit_reached = json["dailyLimitReached"].as_bool().unwrap();
            let is_disputed = json["isDisputed"].as_bool().unwrap();
            let is_incomplete = json["isIncomplete"].as_bool().unwrap();
            let results_analyzed_count = json["resultsAnalyzedCount"].as_u64().unwrap();

            Ok(PublicationsSummary {
                summary: summary,
                daily_limit_reached: daily_limit_reached,
                is_disputed: is_disputed,
                is_incomplete: is_incomplete,
                results_analyzed_count: results_analyzed_count,
            })
        } else {
            let err_msg = format!("Failed to fetch summary: {}", res.text().await?);
            Err(anyhow::anyhow!(err_msg))
        }
    }

    pub async fn fetch_consensus(search_id: &str) -> Result<ConsensusResult, anyhow::Error> {
        let api_token = match std::env::var("GUIDESCOPER_API_TOKEN") {
            Ok(token) => token,
            Err(_) => {
                return Err(anyhow::anyhow!("GUIDESCOPER_API_TOKEN not found"));
            }
        };

        let guidescoper_server = match std::env::var("GUIDESCOPER_SERVER") {
            Ok(token) => token,
            Err(_) => {
                return Err(anyhow::anyhow!("GUIDESCOPER_SERVER not found"));
            }
        };

        let consensus_api = format!("{}{}", guidescoper_server, GUIDESCOPER_CONSENSUS_API);

        let url = format!("{}{}", consensus_api, search_id);
        let cookie = format!("_session={}", api_token);
        let client = reqwest::Client::new();
        let res = client
            .get(&url)
            .header("Cookie", cookie)
            .header("USER_AGENT", USER_AGENT)
            .send()
            .await?;

        if res.status().is_success() {
            let body = res.text().await?;
            let json: serde_json::Value = serde_json::from_str(&body)?;

            let results_analyzed_count = json["resultsAnalyzedCount"].as_u64().unwrap();

            let yes_no_answer_percents = &json["yesNoAnswerPercents"];
            let yes_percent = yes_no_answer_percents["YES"].as_f64().unwrap();
            let no_percent = yes_no_answer_percents["NO"].as_f64().unwrap();
            let possibly_percent = yes_no_answer_percents["POSSIBLY"].as_f64().unwrap();

            let result_id_to_yes_no_answer = json["resultIdToYesNoAnswer"].as_object().unwrap();

            let mut yes_doc_ids_vec = Vec::new();
            let mut no_doc_ids_vec = Vec::new();
            let mut possibly_doc_ids_vec = Vec::new();

            for (doc_id, answer) in result_id_to_yes_no_answer {
                match answer.as_str().unwrap() {
                    "YES" => yes_doc_ids_vec.push(doc_id.clone()),
                    "NO" => no_doc_ids_vec.push(doc_id.clone()),
                    "POSSIBLY" => possibly_doc_ids_vec.push(doc_id.clone()),
                    _ => {}
                }
            }

            let is_incomplete = json["isIncomplete"].as_bool().unwrap();
            let is_disputed = json["isDisputed"].as_bool().unwrap();

            Ok(ConsensusResult {
                results_analyzed_count: results_analyzed_count,
                yes_percent: yes_percent,
                no_percent: no_percent,
                possibly_percent: possibly_percent,
                yes_doc_ids: yes_doc_ids_vec,
                no_doc_ids: no_doc_ids_vec,
                possibly_doc_ids: possibly_doc_ids_vec,
                is_incomplete: is_incomplete,
                is_disputed: is_disputed,
            })
        } else {
            let err_msg = format!("Failed to fetch consensus: {}", res.text().await?);
            Err(anyhow::anyhow!(err_msg))
        }
    }
}
