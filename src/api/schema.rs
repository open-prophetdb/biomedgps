use std::collections::HashMap;

use crate::model::core::{RecordResponse, RelationCount};
use crate::model::core::{JSON_REGEX, SUBGRAPH_UUID_REGEX};
use crate::model::graph::Graph;
use crate::model::graph::{COMPOSED_ENTITIES_REGEX, COMPOSED_ENTITY_REGEX, RELATION_TYPE_REGEX};
use crate::model::publication::PublicationRecords;
use log::warn;
use poem_openapi::Object;
use poem_openapi::{
    payload::{Binary, Json},
    types::multipart::Upload,
    ApiResponse, Multipart, Tags,
};
use serde::{Deserialize, Serialize};
use validator::Validate;
use validator::ValidationErrors;

#[derive(Tags)]
pub enum ApiTags {
    KnowledgeGraph,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Object)]
pub struct ErrorMessage {
    msg: String,
}

#[derive(Debug, Serialize, Deserialize, Validate, Object)]

pub struct PromptList {
    /// data
    pub records: Vec<HashMap<String, String>>,
    /// total num
    pub total: u64,
    /// current page index
    pub page: u64,
    /// default 10
    pub page_size: u64,
}

#[derive(ApiResponse)]
pub enum GetPromptResponse {
    #[oai(status = 200)]
    Ok(Json<PromptList>),

    #[oai(status = 400)]
    BadRequest(Json<ErrorMessage>),

    #[oai(status = 404)]
    NotFound(Json<ErrorMessage>),
}

impl GetPromptResponse {
    pub fn ok(prompts: PromptList) -> Self {
        Self::Ok(Json(prompts))
    }

    pub fn bad_request(msg: String) -> Self {
        Self::BadRequest(Json(ErrorMessage { msg }))
    }

    pub fn not_found(msg: String) -> Self {
        Self::NotFound(Json(ErrorMessage { msg }))
    }
}

#[derive(ApiResponse)]
pub enum GetGraphResponse {
    #[oai(status = 200)]
    Ok(Json<Graph>),

    #[oai(status = 400)]
    BadRequest(Json<ErrorMessage>),

    #[oai(status = 404)]
    NotFound(Json<ErrorMessage>),
}

impl GetGraphResponse {
    pub fn ok(graph: Graph) -> Self {
        Self::Ok(Json(graph))
    }

    pub fn bad_request(msg: String) -> Self {
        Self::BadRequest(Json(ErrorMessage { msg }))
    }

    pub fn not_found(msg: String) -> Self {
        Self::NotFound(Json(ErrorMessage { msg }))
    }
}

#[derive(ApiResponse)]
pub enum GetEntityColorMapResponse {
    #[oai(status = 200)]
    Ok(Json<HashMap<String, String>>),

    #[oai(status = 400)]
    BadRequest(Json<ErrorMessage>),

    #[oai(status = 404)]
    NotFound(Json<ErrorMessage>),
}

impl GetEntityColorMapResponse {
    pub fn ok(h: HashMap<String, String>) -> Self {
        Self::Ok(Json(h))
    }

    pub fn bad_request(msg: String) -> Self {
        Self::BadRequest(Json(ErrorMessage { msg }))
    }

    pub fn not_found(msg: String) -> Self {
        Self::NotFound(Json(ErrorMessage { msg }))
    }
}

#[derive(ApiResponse)]
pub enum GetRelationCountResponse {
    #[oai(status = 200)]
    Ok(Json<Vec<RelationCount>>),

    #[oai(status = 400)]
    BadRequest(Json<ErrorMessage>),

    #[oai(status = 404)]
    NotFound(Json<ErrorMessage>),
}

impl GetRelationCountResponse {
    pub fn ok(relation_counts: Vec<RelationCount>) -> Self {
        Self::Ok(Json(relation_counts))
    }

    pub fn bad_request(msg: String) -> Self {
        Self::BadRequest(Json(ErrorMessage { msg }))
    }

    pub fn not_found(msg: String) -> Self {
        Self::NotFound(Json(ErrorMessage { msg }))
    }
}

#[derive(ApiResponse)]
pub enum GetRecordResponse<
    S: Serialize
        + std::fmt::Debug
        + std::marker::Unpin
        + Send
        + Sync
        + poem_openapi::types::Type
        + poem_openapi::types::ParseFromJSON
        + poem_openapi::types::ToJSON,
> {
    #[oai(status = 200)]
    Ok(Json<S>),

    #[oai(status = 400)]
    BadRequest(Json<ErrorMessage>),

    #[oai(status = 404)]
    NotFound(Json<ErrorMessage>),

    #[oai(status = 500)]
    InternalServerError(Json<ErrorMessage>),
}

impl<
        S: Serialize
            + std::fmt::Debug
            + std::marker::Unpin
            + Send
            + Sync
            + poem_openapi::types::Type
            + poem_openapi::types::ParseFromJSON
            + poem_openapi::types::ToJSON,
    > GetRecordResponse<S>
{
    pub fn ok(record_response: S) -> Self {
        Self::Ok(Json(record_response))
    }

    pub fn bad_request(msg: String) -> Self {
        Self::BadRequest(Json(ErrorMessage { msg }))
    }

    pub fn not_found(msg: String) -> Self {
        Self::NotFound(Json(ErrorMessage { msg }))
    }

    pub fn internal_server_error(msg: String) -> Self {
        Self::InternalServerError(Json(ErrorMessage { msg }))
    }
}

#[derive(ApiResponse)]
pub enum GetWholeTableResponse<
    T: Serialize
        + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>
        + std::fmt::Debug
        + std::marker::Unpin
        + Send
        + Sync
        + poem_openapi::types::Type
        + poem_openapi::types::ParseFromJSON
        + poem_openapi::types::ToJSON,
> {
    #[oai(status = 200)]
    Ok(Json<Vec<T>>),

    #[oai(status = 400)]
    BadRequest(Json<ErrorMessage>),

    #[oai(status = 404)]
    NotFound(Json<ErrorMessage>),
}

impl<
        T: Serialize
            + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>
            + std::fmt::Debug
            + std::marker::Unpin
            + Send
            + Sync
            + poem_openapi::types::Type
            + poem_openapi::types::ParseFromJSON
            + poem_openapi::types::ToJSON,
    > GetWholeTableResponse<T>
{
    pub fn ok(vec_t: Vec<T>) -> Self {
        Self::Ok(Json(vec_t))
    }

    pub fn bad_request(msg: String) -> Self {
        Self::BadRequest(Json(ErrorMessage { msg }))
    }

    pub fn not_found(msg: String) -> Self {
        Self::NotFound(Json(ErrorMessage { msg }))
    }
}

#[derive(ApiResponse)]
pub enum GetPublicationsResponse {
    #[oai(status = 200)]
    Ok(Json<PublicationRecords>),

    #[oai(status = 400)]
    BadRequest(Json<ErrorMessage>),

    #[oai(status = 404)]
    NotFound(Json<ErrorMessage>),
}

impl GetPublicationsResponse {
    pub fn ok(publication_records: PublicationRecords) -> Self {
        Self::Ok(Json(publication_records))
    }

    pub fn bad_request(msg: String) -> Self {
        Self::BadRequest(Json(ErrorMessage { msg }))
    }

    pub fn not_found(msg: String) -> Self {
        Self::NotFound(Json(ErrorMessage { msg }))
    }
}

#[derive(ApiResponse)]
pub enum GetRecordsResponse<
    S: Serialize
        + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>
        + std::fmt::Debug
        + std::marker::Unpin
        + Send
        + Sync
        + poem_openapi::types::Type
        + poem_openapi::types::ParseFromJSON
        + poem_openapi::types::ToJSON,
> {
    #[oai(status = 200)]
    Ok(Json<RecordResponse<S>>),

    #[oai(status = 400)]
    BadRequest(Json<ErrorMessage>),

    #[oai(status = 404)]
    NotFound(Json<ErrorMessage>),

    #[oai(status = 500)]
    InternalServerError(Json<ErrorMessage>),
}

impl<
        S: Serialize
            + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>
            + std::fmt::Debug
            + std::marker::Unpin
            + Send
            + Sync
            + poem_openapi::types::Type
            + poem_openapi::types::ParseFromJSON
            + poem_openapi::types::ToJSON,
    > GetRecordsResponse<S>
{
    pub fn ok(record_response: RecordResponse<S>) -> Self {
        Self::Ok(Json(record_response))
    }

    pub fn bad_request(msg: String) -> Self {
        Self::BadRequest(Json(ErrorMessage { msg }))
    }

    pub fn not_found(msg: String) -> Self {
        Self::NotFound(Json(ErrorMessage { msg }))
    }

    pub fn internal_server_error(msg: String) -> Self {
        Self::InternalServerError(Json(ErrorMessage { msg }))
    }
}

#[derive(ApiResponse)]
pub enum PostResponse<
    S: Serialize
        + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>
        + std::fmt::Debug
        + std::marker::Unpin
        + Send
        + Sync
        + poem_openapi::types::Type
        + poem_openapi::types::ParseFromJSON
        + poem_openapi::types::ToJSON,
> {
    #[oai(status = 201)]
    Created(Json<S>),

    #[oai(status = 400)]
    BadRequest(Json<ErrorMessage>),

    #[oai(status = 404)]
    NotFound(Json<ErrorMessage>),
}

impl<
        S: Serialize
            + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow>
            + std::fmt::Debug
            + std::marker::Unpin
            + Send
            + Sync
            + poem_openapi::types::Type
            + poem_openapi::types::ParseFromJSON
            + poem_openapi::types::ToJSON,
    > PostResponse<S>
{
    pub fn created(s: S) -> Self {
        Self::Created(Json(s))
    }

    pub fn bad_request(msg: String) -> Self {
        Self::BadRequest(Json(ErrorMessage { msg }))
    }

    pub fn not_found(msg: String) -> Self {
        Self::NotFound(Json(ErrorMessage { msg }))
    }
}

#[derive(ApiResponse)]
pub enum DeleteResponse {
    #[oai(status = 204)]
    NoContent,

    #[oai(status = 400)]
    BadRequest(Json<ErrorMessage>),

    #[oai(status = 404)]
    NotFound(Json<ErrorMessage>),
}

impl DeleteResponse {
    pub fn no_content() -> Self {
        Self::NoContent
    }

    pub fn bad_request(msg: String) -> Self {
        Self::BadRequest(Json(ErrorMessage { msg }))
    }

    pub fn not_found(msg: String) -> Self {
        Self::NotFound(Json(ErrorMessage { msg }))
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct SubgraphIdQuery {
    /// The ID of a subgraph.
    #[validate(regex(
        path = "SUBGRAPH_UUID_REGEX",
        message = "Invalid subgraph id, it must be a valid UUID."
    ))]
    pub subgraph_id: String,
}

impl SubgraphIdQuery {
    pub fn new(subgraph_id: &str) -> Result<Self, ValidationErrors> {
        let subgraph_id = subgraph_id.to_string();
        let query = Self { subgraph_id };
        match query.validate() {
            Ok(_) => Ok(query),
            Err(e) => {
                let err = format!("Invalid query: {}", e);
                warn!("{}", err);
                Err(e)
            }
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct NodeIdQuery {
    /// The ID of the object.
    #[validate(regex(
        path = "COMPOSED_ENTITY_REGEX",
        message = "Invalid node id, it must be composed of entity type, ::, and entity id. e.g. Disease::MESH:D001"
    ))]
    pub node_id: String,
}

impl NodeIdQuery {
    pub fn new(node_id: &str) -> Result<Self, ValidationErrors> {
        let node_id = node_id.to_string();
        let query = Self { node_id };
        match query.validate() {
            Ok(_) => Ok(query),
            Err(e) => {
                let err = format!("Invalid query: {}", e);
                warn!("{}", err);
                Err(e)
            }
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct NodeIdsQuery {
    /// The ID of the object.
    #[validate(regex(
        path = "COMPOSED_ENTITIES_REGEX",
        message = "Invalid node ids, each node id in it must be composed of entity type, ::, and entity id. There is a comma between each node id. e.g. Disease::MESH:D001,Disease::MESH:D002"
    ))]
    pub node_ids: String,
}

impl NodeIdsQuery {
    pub fn new(node_ids: &str) -> Result<Self, ValidationErrors> {
        let node_ids = node_ids.to_string();
        let query = Self { node_ids };
        match query.validate() {
            Ok(_) => Ok(query),
            Err(e) => {
                let err = format!("Invalid query: {}", e);
                warn!("{}", err);
                Err(e)
            }
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct PredictedNodeQuery {
    /// The ID of the object.
    #[validate(regex(
        path = "COMPOSED_ENTITIES_REGEX",
        message = "Invalid node id, it must be composed of entity type, ::, and entity id. e.g. Disease::MESH:D001"
    ))]
    pub node_id: String,

    #[validate(regex(
        path = "RELATION_TYPE_REGEX",
        message = "Invalid relation type, it must be a valid relation type. e.g. biomedgps::treats::Compound:Disease"
    ))]
    pub relation_type: String,

    #[validate(regex(
        path = "JSON_REGEX",
        message = "Invalid query string, it must be a json string"
    ))]
    pub query_str: Option<String>,

    #[validate(range(
        min = 0,
        max = 500,
        message = "Invalid threshold, it must be between 0 and 500"
    ))]
    pub topk: Option<u64>,
}

impl PredictedNodeQuery {
    pub fn new(
        node_id: &str,
        relation_type: &str,
        query_str: &Option<String>,
        topk: Option<u64>,
    ) -> Result<Self, ValidationErrors> {
        let query = Self {
            node_id: node_id.to_string(),
            relation_type: relation_type.to_string(),
            query_str: query_str.clone(),
            topk,
        };

        match query.validate() {
            Ok(_) => Ok(query),
            Err(e) => {
                let err = format!("Invalid query: {}", e);
                warn!("{}", err);
                Err(e)
            }
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct Pagination {
    #[validate(range(min = 1, message = "Invalid page number, it must be greater than 0"))]
    pub page: Option<u64>,

    #[validate(range(min = 1, message = "Invalid page size, it must be greater than 0"))]
    pub page_size: Option<u64>,
}

impl Pagination {
    pub fn new(page: Option<u64>, page_size: Option<u64>) -> Result<Self, ValidationErrors> {
        let pagination = match (page, page_size) {
            (Some(page), Some(page_size)) => {
                let p = Self {
                    page: Some(page),
                    page_size: Some(page_size),
                };
                match p.validate() {
                    Ok(_) => p,
                    Err(e) => {
                        let err = format!("Invalid pagination: {}", e);
                        warn!("{}", err);
                        return Err(e);
                    }
                }
            }
            _ => Self {
                page: Some(1),
                page_size: Some(10),
            },
        };

        Ok(pagination)
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct PaginationQuery {
    #[validate(range(min = 1, message = "Invalid page number, it must be greater than 0"))]
    pub page: Option<u64>,

    #[validate(range(min = 1, message = "Invalid page size, it must be greater than 0"))]
    pub page_size: Option<u64>,

    #[validate(regex(
        path = "JSON_REGEX",
        message = "Invalid query string, it must be a json string"
    ))]
    pub query_str: Option<String>,
}

impl PaginationQuery {
    pub fn new(
        page: Option<u64>,
        page_size: Option<u64>,
        query_str: Option<String>,
    ) -> Result<Self, ValidationErrors> {
        let pagination = match (page, page_size, query_str) {
            (Some(page), Some(page_size), Some(query_str)) => {
                let p = Self {
                    page: Some(page),
                    page_size: Some(page_size),
                    query_str: Some(query_str),
                };
                match p.validate() {
                    Ok(_) => p,
                    Err(e) => {
                        let err = format!("Invalid pagination query: {}", e);
                        warn!("{}", err);
                        return Err(e);
                    }
                }
            }
            _ => Self {
                page: Some(1),
                page_size: Some(10),
                query_str: None,
            },
        };

        Ok(pagination)
    }
}

#[derive(Multipart)]
pub struct UploadImage {
    pub raw_image_url: String,
    pub raw_image_src: String,
    pub name: String,
    pub image: Upload,
}

#[derive(ApiResponse)]
pub enum FileResponse {
    #[oai(status = 200)]
    File(Binary<Vec<u8>>),
    #[oai(status = 500)]
    InternalServerError(Json<ErrorMessage>),
    #[oai(status = 400)]
    BadRequest(Json<ErrorMessage>),
    #[oai(status = 404)]
    NotFound(Json<ErrorMessage>),
}

impl FileResponse {
    pub fn file(file: Vec<u8>) -> Self {
        Self::File(Binary(file))
    }

    pub fn internal_server_error(msg: String) -> Self {
        Self::InternalServerError(Json(ErrorMessage { msg }))
    }

    pub fn bad_request(msg: String) -> Self {
        Self::BadRequest(Json(ErrorMessage { msg }))
    }

    pub fn not_found(msg: String) -> Self {
        Self::NotFound(Json(ErrorMessage { msg }))
    }
}