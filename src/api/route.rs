//! This module defines the routes of the API.

use crate::model::core::{
    Entity, Entity2D, EntityMetadata, KnowledgeCuration, RecordResponse, Relation,
    RelationMetadata, Subgraph,
};
use crate::model::graph::Graph;
use log::{debug, info, warn};
use poem::web::Data;
use poem_openapi::Object;
use poem_openapi::{param::Path, param::Query, payload::Json, ApiResponse, OpenApi, Tags};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Tags)]
enum ApiTags {
    KnowledgeGraph,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Object)]
struct ErrorMessage {
    msg: String,
}

#[derive(ApiResponse)]
enum GetGraphResponse {
    #[oai(status = 200)]
    Ok(Json<Graph>),

    #[oai(status = 400)]
    BadRequest(Json<ErrorMessage>),

    #[oai(status = 404)]
    NotFound(Json<ErrorMessage>),
}

#[derive(ApiResponse)]
enum GetWholeTableResponse<
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

#[derive(ApiResponse)]
enum GetRecordsResponse<
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
}

#[derive(ApiResponse)]
enum PostResponse<
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

#[derive(ApiResponse)]
enum DeleteResponse {
    #[oai(status = 204)]
    NoContent,

    #[oai(status = 400)]
    BadRequest(Json<ErrorMessage>),

    #[oai(status = 404)]
    NotFound(Json<ErrorMessage>),
}

pub struct BiomedgpsApi;

#[OpenApi]
impl BiomedgpsApi {
    /// Call `/api/v1/entity-metadata` with query params to fetch all entity metadata.
    #[oai(
        path = "/api/v1/entity-metadata",
        method = "get",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "fetchEntityMetadata"
    )]
    async fn fetch_entity_metadata(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
    ) -> GetWholeTableResponse<EntityMetadata> {
        let pool_arc = pool.clone();

        match EntityMetadata::get_entity_metadata(&pool_arc).await {
            Ok(entity_metadata) => GetWholeTableResponse::Ok(Json(entity_metadata)),
            Err(e) => {
                let err = format!("Failed to fetch entity metadata: {}", e);
                warn!("{}", err);
                return GetWholeTableResponse::BadRequest(Json(ErrorMessage { msg: err }));
            }
        }
    }

    /// Call `/api/v1/relation-metadata` with query params to fetch all relation metadata.
    #[oai(
        path = "/api/v1/relation-metadata",
        method = "get",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "fetchRelationMetadata"
    )]
    async fn fetch_relation_metadata(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
    ) -> GetWholeTableResponse<RelationMetadata> {
        let pool_arc = pool.clone();

        match RelationMetadata::get_relation_metadata(&pool_arc).await {
            Ok(relation_metadata) => GetWholeTableResponse::Ok(Json(relation_metadata)),
            Err(e) => {
                let err = format!("Failed to fetch relation metadata: {}", e);
                warn!("{}", err);
                return GetWholeTableResponse::BadRequest(Json(ErrorMessage { msg: err }));
            }
        }
    }

    /// Call `/api/v1/entities` with query params to fetch entities.
    #[oai(
        path = "/api/v1/entities",
        method = "get",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "fetchEntities"
    )]
    async fn fetch_entities(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        page: Query<Option<u64>>,
        page_size: Query<Option<u64>>,
        query_str: Query<Option<String>>,
    ) -> GetRecordsResponse<Entity> {
        let pool_arc = pool.clone();
        let page = page.0;
        let page_size = page_size.0;

        let query_str = match query_str.0 {
            Some(query_str) => query_str,
            None => {
                warn!("Query string is empty.");
                "".to_string()
            }
        };

        let query = if query_str == "" {
            None
        } else {
            debug!("Query string: {}", &query_str);
            // Parse query string as json
            match serde_json::from_str(&query_str) {
                Ok(query) => Some(query),
                Err(e) => {
                    let err = format!("Failed to parse query string: {}", e);
                    warn!("{}", err);
                    return GetRecordsResponse::BadRequest(Json(ErrorMessage { msg: err }));
                }
            }
        };

        match RecordResponse::<Entity>::get_records(
            &pool_arc,
            "biomedgps_entity",
            &query,
            page,
            page_size,
            Some("id ASC"),
        )
        .await
        {
            Ok(entities) => GetRecordsResponse::Ok(Json(entities)),
            Err(e) => {
                let err = format!("Failed to fetch datasets: {}", e);
                warn!("{}", err);
                return GetRecordsResponse::BadRequest(Json(ErrorMessage { msg: err }));
            }
        }
    }

    /// Call `/api/v1/curated-knowledges` with query params to fetch curated knowledges.
    #[oai(
        path = "/api/v1/curated-knowledges",
        method = "get",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "fetchCuratedKnowledges"
    )]
    async fn fetch_curated_knowledges(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        page: Query<Option<u64>>,
        page_size: Query<Option<u64>>,
        query_str: Query<Option<String>>,
    ) -> GetRecordsResponse<KnowledgeCuration> {
        let pool_arc = pool.clone();
        let page = page.0;
        let page_size = page_size.0;

        let query_str = match query_str.0 {
            Some(query_str) => query_str,
            None => {
                warn!("Query string is empty.");
                "".to_string()
            }
        };

        let query = if query_str == "" {
            None
        } else {
            debug!("Query string: {}", &query_str);
            // Parse query string as json
            match serde_json::from_str(&query_str) {
                Ok(query) => Some(query),
                Err(e) => {
                    let err = format!("Failed to parse query string: {}", e);
                    warn!("{}", err);
                    return GetRecordsResponse::BadRequest(Json(ErrorMessage { msg: err }));
                }
            }
        };

        match RecordResponse::<KnowledgeCuration>::get_records(
            &pool_arc,
            "biomedgps_knowledge_curation",
            &query,
            page,
            page_size,
            Some("relation_id ASC"),
        )
        .await
        {
            Ok(entities) => GetRecordsResponse::Ok(Json(entities)),
            Err(e) => {
                let err = format!("Failed to fetch datasets: {}", e);
                warn!("{}", err);
                return GetRecordsResponse::BadRequest(Json(ErrorMessage { msg: err }));
            }
        }
    }

    /// Call `/api/v1/curated-knowledges` with payload to create a curated knowledge.
    #[oai(
        path = "/api/v1/curated-knowledges",
        method = "post",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "postCuratedKnowledge"
    )]
    async fn post_curated_knowledge(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        payload: Json<KnowledgeCuration>,
    ) -> PostResponse<KnowledgeCuration> {
        let pool_arc = pool.clone();
        let payload = payload.0;

        match payload.insert(&pool_arc).await {
            Ok(kc) => PostResponse::Created(Json(kc)),
            Err(e) => {
                let err = format!("Failed to insert curated knowledge: {}", e);
                warn!("{}", err);
                return PostResponse::BadRequest(Json(ErrorMessage { msg: err }));
            }
        }
    }

    /// Call `/api/v1/curated-knowledges/:id` with payload to create a curated knowledge.
    #[oai(
        path = "/api/v1/curated-knowledges/:id",
        method = "put",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "putCuratedKnowledge"
    )]
    async fn put_curated_knowledge(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        payload: Json<KnowledgeCuration>,
        id: Path<String>,
    ) -> PostResponse<KnowledgeCuration> {
        let pool_arc = pool.clone();
        let payload = payload.0;
        let id = id.0;

        match payload.update(&pool_arc, &id).await {
            Ok(kc) => PostResponse::Created(Json(kc)),
            Err(e) => {
                let err = format!("Failed to insert curated knowledge: {}", e);
                warn!("{}", err);
                return PostResponse::BadRequest(Json(ErrorMessage { msg: err }));
            }
        }
    }

    /// Call `/api/v1/curated-knowledges/:id` with payload to delete a curated knowledge.
    #[oai(
        path = "/api/v1/curated-knowledges/:id",
        method = "delete",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "deleteCuratedKnowledge"
    )]
    async fn delete_curated_knowledge(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        id: Path<String>,
    ) -> DeleteResponse {
        let pool_arc = pool.clone();
        let id = id.0;

        match KnowledgeCuration::delete(&pool_arc, &id).await {
            Ok(_) => DeleteResponse::NoContent,
            Err(e) => {
                let err = format!("Failed to delete curated knowledge: {}", e);
                warn!("{}", err);
                DeleteResponse::NotFound(Json(ErrorMessage { msg: err }))
            }
        }
    }

    /// Call `/api/v1/relations` with query params to fetch relations.
    #[oai(
        path = "/api/v1/relations",
        method = "get",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "fetchRelations"
    )]
    async fn fetch_relations(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        page: Query<Option<u64>>,
        page_size: Query<Option<u64>>,
        query_str: Query<Option<String>>,
    ) -> GetRecordsResponse<Relation> {
        let pool_arc = pool.clone();
        let page = page.0;
        let page_size = page_size.0;

        let query_str = match query_str.0 {
            Some(query_str) => query_str,
            None => {
                warn!("Query string is empty.");
                "".to_string()
            }
        };

        let query = if query_str == "" {
            None
        } else {
            debug!("Query string: {}", &query_str);
            // Parse query string as json
            match serde_json::from_str(&query_str) {
                Ok(query) => Some(query),
                Err(e) => {
                    let err = format!("Failed to parse query string: {}", e);
                    warn!("{}", err);
                    return GetRecordsResponse::BadRequest(Json(ErrorMessage { msg: err }));
                }
            }
        };

        match RecordResponse::<Relation>::get_records(
            &pool_arc,
            "biomedgps_relation",
            &query,
            page,
            page_size,
            Some("id ASC"),
        )
        .await
        {
            Ok(entities) => GetRecordsResponse::Ok(Json(entities)),
            Err(e) => {
                let err = format!("Failed to fetch datasets: {}", e);
                warn!("{}", err);
                return GetRecordsResponse::BadRequest(Json(ErrorMessage { msg: err }));
            }
        }
    }

    /// Call `/api/v1/entity2d` with query params to fetch entity2d.
    #[oai(
        path = "/api/v1/entity2d",
        method = "get",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "fetchEntity2d"
    )]
    async fn fetch_entity2d(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        page: Query<Option<u64>>,
        page_size: Query<Option<u64>>,
        query_str: Query<Option<String>>,
    ) -> GetRecordsResponse<Entity2D> {
        let pool_arc = pool.clone();
        let page = page.0;
        let page_size = page_size.0;

        let query_str = match query_str.0 {
            Some(query_str) => query_str,
            None => {
                warn!("Query string is empty.");
                "".to_string()
            }
        };

        let query = if query_str == "" {
            None
        } else {
            debug!("Query string: {}", &query_str);
            // Parse query string as json
            match serde_json::from_str(&query_str) {
                Ok(query) => Some(query),
                Err(e) => {
                    let err = format!("Failed to parse query string: {}", e);
                    warn!("{}", err);
                    return GetRecordsResponse::BadRequest(Json(ErrorMessage { msg: err }));
                }
            }
        };

        match RecordResponse::<Entity2D>::get_records(
            &pool_arc,
            "biomedgps_entity2d",
            &query,
            page,
            page_size,
            Some("embedding_id ASC"),
        )
        .await
        {
            Ok(entities) => GetRecordsResponse::Ok(Json(entities)),
            Err(e) => {
                let err = format!("Failed to fetch datasets: {}", e);
                warn!("{}", err);
                return GetRecordsResponse::BadRequest(Json(ErrorMessage { msg: err }));
            }
        }
    }

    /// Call `/api/v1/subgraphs` with query params to fetch subgraphs.
    #[oai(
        path = "/api/v1/subgraphs",
        method = "get",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "fetchSubgraphs"
    )]
    async fn fetch_subgraphs(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        page: Query<Option<u64>>,
        page_size: Query<Option<u64>>,
        query_str: Query<Option<String>>,
    ) -> GetRecordsResponse<Subgraph> {
        let pool_arc = pool.clone();
        let page = page.0;
        let page_size = page_size.0;

        let query_str = match query_str.0 {
            Some(query_str) => query_str,
            None => {
                warn!("Query string is empty.");
                "".to_string()
            }
        };

        let query = if query_str == "" {
            None
        } else {
            debug!("Query string: {}", &query_str);
            // Parse query string as json
            match serde_json::from_str(&query_str) {
                Ok(query) => Some(query),
                Err(e) => {
                    let err = format!("Failed to parse query string: {}", e);
                    warn!("{}", err);
                    return GetRecordsResponse::BadRequest(Json(ErrorMessage { msg: err }));
                }
            }
        };

        match RecordResponse::<Subgraph>::get_records(
            &pool_arc,
            "biomedgps_subgraph",
            &query,
            page,
            page_size,
            Some("created_time DESC"),
        )
        .await
        {
            Ok(entities) => GetRecordsResponse::Ok(Json(entities)),
            Err(e) => {
                let err = format!("Failed to fetch datasets: {}", e);
                warn!("{}", err);
                return GetRecordsResponse::BadRequest(Json(ErrorMessage { msg: err }));
            }
        }
    }

    /// Call `/api/v1/subgraphs` with payload to create a subgraph.
    #[oai(
        path = "/api/v1/subgraphs",
        method = "post",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "postSubgraph"
    )]
    async fn post_subgraph(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        payload: Json<Subgraph>,
    ) -> PostResponse<Subgraph> {
        let pool_arc = pool.clone();
        let payload = payload.0;

        match payload.insert(&pool_arc).await {
            Ok(kc) => PostResponse::Created(Json(kc)),
            Err(e) => {
                let err = format!("Failed to insert curated knowledge: {}", e);
                warn!("{}", err);
                return PostResponse::BadRequest(Json(ErrorMessage { msg: err }));
            }
        }
    }

    /// Call `/api/v1/subgraphs/:id` with payload to update a subgraph.
    #[oai(
        path = "/api/v1/subgraphs",
        method = "put",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "putSubgraph"
    )]
    async fn put_subgraph(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        id: Path<String>,
        payload: Json<Subgraph>,
    ) -> PostResponse<Subgraph> {
        let pool_arc = pool.clone();
        let id = id.0;
        let payload = payload.0;

        match payload.update(&pool_arc, &id).await {
            Ok(kc) => PostResponse::Created(Json(kc)),
            Err(e) => {
                let err = format!("Failed to update curated knowledge: {}", e);
                warn!("{}", err);
                return PostResponse::BadRequest(Json(ErrorMessage { msg: err }));
            }
        }
    }

    /// Call `/api/v1/subgraphs/:id` with payload to create subgraph.
    #[oai(
        path = "/api/v1/subgraphs/:id",
        method = "delete",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "deleteSubgraph"
    )]
    async fn delete_subgraph(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        id: Path<String>,
    ) -> DeleteResponse {
        let pool_arc = pool.clone();
        let id = id.0;

        match Subgraph::delete(&pool_arc, &id).await {
            Ok(_) => DeleteResponse::NoContent,
            Err(e) => {
                let err = format!("Failed to delete a subgraph: {}", e);
                warn!("{}", err);
                DeleteResponse::NotFound(Json(ErrorMessage { msg: err }))
            }
        }
    }

    /// Call `/api/v1/nodes` with query params to fetch nodes.
    #[oai(
        path = "/api/v1/nodes",
        method = "get",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "fetchNodes"
    )]
    async fn fetch_nodes(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        node_ids: Query<String>,
    ) -> GetGraphResponse {
        let pool_arc = pool.clone();
        let node_ids = node_ids.0;

        let mut graph = Graph::new();

        if node_ids == "" {
            return GetGraphResponse::Ok(Json(graph));
        }

        let node_ids: Vec<&str> = node_ids.split(",").collect();
        match graph.fetch_nodes_by_ids(&pool_arc, &node_ids).await {
            Ok(graph) => GetGraphResponse::Ok(Json(graph.to_owned().get_graph(None).unwrap())),
            Err(e) => {
                let err = format!("Failed to fetch nodes: {}", e);
                warn!("{}", err);
                return GetGraphResponse::BadRequest(Json(ErrorMessage { msg: err }));
            }
        }
    }

    /// Call `/api/v1/auto-connect-nodes` with query params to fetch edges which connect the input nodes.
    #[oai(
        path = "/api/v1/auto-connect-nodes",
        method = "get",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "fetchEdgesAutoConnectNodes"
    )]
    async fn fetch_edges_auto_connect_nodes(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        node_ids: Query<String>,
    ) -> GetGraphResponse {
        let pool_arc = pool.clone();
        let node_ids = node_ids.0;

        let mut graph = Graph::new();

        if node_ids == "" {
            return GetGraphResponse::Ok(Json(graph));
        }

        let node_ids: Vec<&str> = node_ids.split(",").collect();
        match graph.auto_connect_nodes(&pool_arc, &node_ids).await {
            Ok(graph) => GetGraphResponse::Ok(Json(graph.to_owned().get_graph(None).unwrap())),
            Err(e) => {
                let err = format!("Failed to fetch nodes: {}", e);
                warn!("{}", err);
                return GetGraphResponse::BadRequest(Json(ErrorMessage { msg: err }));
            }
        }
    }

    /// Call `/api/v1/one-step-linked-nodes` with query params to fetch linked nodes with one step.
    #[oai(
        path = "/api/v1/one-step-linked-nodes",
        method = "get",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "fetchOneStepLinkedNodes"
    )]
    async fn fetch_one_step_linked_nodes(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        page: Query<Option<u64>>,
        page_size: Query<Option<u64>>,
        query_str: Query<Option<String>>,
    ) -> GetGraphResponse {
        let pool_arc = pool.clone();
        let page = page.0;
        let page_size = page_size.0;

        let query_str = match query_str.0 {
            Some(query_str) => query_str,
            None => {
                warn!("Query string is empty.");
                "".to_string()
            }
        };

        let query = if query_str == "" {
            None
        } else {
            debug!("Query string: {}", &query_str);
            // Parse query string as json
            match serde_json::from_str(&query_str) {
                Ok(query) => Some(query),
                Err(e) => {
                    let err = format!("Failed to parse query string: {}", e);
                    warn!("{}", err);
                    return GetGraphResponse::BadRequest(Json(ErrorMessage { msg: err }));
                }
            }
        };

        let mut graph = Graph::new();
        match graph
            .fetch_linked_nodes(&pool_arc, &query, page, page_size, None)
            .await
        {
            Ok(graph) => GetGraphResponse::Ok(Json(graph.to_owned().get_graph(None).unwrap())),
            Err(e) => {
                let err = format!("Failed to fetch linked nodes: {}", e);
                warn!("{}", err);
                return GetGraphResponse::BadRequest(Json(ErrorMessage { msg: err }));
            }
        }
    }
}
