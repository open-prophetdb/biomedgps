//! This module defines the routes of the API.

use crate::api::auth::{CustomSecurityScheme, USERNAME_PLACEHOLDER};
use crate::api::schema::{
    ApiTags, DeleteResponse, FileResponse, GetEntityColorMapResponse, GetGraphResponse,
    GetPromptResponse, GetPublicationsResponse, GetRecordResponse, GetRecordsResponse,
    GetRelationCountResponse, GetWholeTableResponse, NodeIdsQuery, Pagination, PaginationQuery,
    PostResponse, PredictedNodeQuery, PromptList, SubgraphIdQuery, UploadImage, LogMessage
};
use crate::model::core::{
    Configuration, Entity, Entity2D, EntityCuration, EntityMetadata, EntityMetadataCuration, Image,
    KeySentenceCuration, KnowledgeCuration, RecordResponse, Relation, RelationCount,
    RelationMetadata, Statistics, Subgraph, WebpageMetadata,
};
use crate::model::embedding::Embedding;
use crate::model::entity::compound::CompoundAttr;
use crate::model::entity_attr::{EntityAttr, EntityAttrRecordResponse};
use crate::model::graph::Graph;
use crate::model::init_db::get_kg_score_table_name;
use crate::model::kge::DEFAULT_MODEL_NAME;
use crate::model::llm::{ChatBot, Context, LlmResponse, PROMPTS};
use crate::model::publication::{ConsensusResult, Publication, PublicationsSummary};
use crate::model::util::match_color;
use crate::model::workspace::{
    ExpandedTask, Notification, Task, Workflow, WorkflowSchema, Workspace,
};
use crate::query_builder::cypher_builder::{query_nhops, query_shared_nodes};
use crate::query_builder::sql_builder::{
    get_all_field_pairs, make_order_clause_by_pairs, ComposeQuery,
};
use log::{debug, info, warn};
use poem::web::Data;
use poem_openapi::{param::Path, param::Query, payload::Json, OpenApi};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use validator::Validate;

pub struct BiomedgpsApi;

#[OpenApi(prefix_path = "/api/v1")]
impl BiomedgpsApi {
    /// Call `/api/v1/publications-summary` with query params to fetch publication summary.
    #[oai(
        path = "/publications-summary/:search_id",
        method = "get",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "fetchPublicationsSummary"
    )]
    async fn fetch_publications_summary(
        &self,
        search_id: Path<String>,
        _token: CustomSecurityScheme,
    ) -> GetRecordResponse<PublicationsSummary> {
        let search_id = search_id.0;
        match Publication::fetch_summary(&search_id).await {
            Ok(result) => GetRecordResponse::ok(result),
            Err(e) => {
                let err = format!("Failed to fetch publications summary: {}", e);
                warn!("{}", err);
                return GetRecordResponse::bad_request(err);
            }
        }
    }

    /// Call `/api/v1/publications-consensus` with query params to fetch publication consensus.
    #[oai(
        path = "/publications-consensus/:search_id",
        method = "get",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "fetchPublicationsConsensus"
    )]
    async fn fetch_publications_consensus(
        &self,
        search_id: Path<String>,
        _token: CustomSecurityScheme,
    ) -> GetRecordResponse<ConsensusResult> {
        let search_id = search_id.0;
        match Publication::fetch_consensus(&search_id).await {
            Ok(result) => GetRecordResponse::ok(result),
            Err(e) => {
                let err = format!("Failed to fetch publications consensus: {}", e);
                warn!("{}", err);
                return GetRecordResponse::bad_request(err);
            }
        }
    }

    /// Call `/api/v1/publications-summary` with query params to fetch publication summary.
    #[oai(
        path = "/publications-summary",
        method = "post",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "answerQuestionWithPublications"
    )]
    async fn answer_question_with_publications(
        &self,
        publications: Json<Vec<Publication>>,
        question: Query<String>,
        pool: Data<&Arc<sqlx::PgPool>>,
        _token: CustomSecurityScheme,
    ) -> GetRecordResponse<PublicationsSummary> {
        let question = question.0;
        let publications = publications.0;
        let pool_arc = pool.clone();
        match Publication::fetch_summary_by_chatgpt(&question, &publications, Some(&pool_arc)).await
        {
            Ok(result) => GetRecordResponse::ok(result),
            Err(e) => {
                let err = format!("Failed to fetch publications summary: {}", e);
                warn!("{}", err);
                return GetRecordResponse::bad_request(err);
            }
        }
    }

    /// Call `/api/v1/publications/:id` to fetch a publication.
    #[oai(
        path = "/publications/:id",
        method = "get",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "fetchPublication"
    )]
    async fn fetch_publication(&self, id: Path<String>) -> GetRecordResponse<Publication> {
        let id = id.0;
        match Publication::fetch_publication(&id).await {
            Ok(publication) => GetRecordResponse::ok(publication),
            Err(e) => {
                let err = format!("Failed to fetch publication: {}", e);
                warn!("{}", err);
                return GetRecordResponse::bad_request(err);
            }
        }
    }

    /// Call `/api/v1/publications` with query params to fetch publications.
    #[oai(
        path = "/publications",
        method = "get",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "fetchPublications"
    )]
    async fn fetch_publications(
        &self,
        query_str: Query<String>,
        page: Query<Option<u64>>,
        page_size: Query<Option<u64>>,
    ) -> GetPublicationsResponse {
        let query_str = query_str.0;
        let page = page.0;
        let page_size = page_size.0;

        info!("Fetch publications with query: {}", query_str);

        match Publication::fetch_publications(&query_str, page, page_size).await {
            Ok(records) => GetPublicationsResponse::ok(records),
            Err(e) => {
                let err = format!("Failed to fetch publications: {}", e);
                warn!("{}", err);
                return GetPublicationsResponse::bad_request(err);
            }
        }
    }

    /// Call `/api/v1/statistics` with query params to fetch all entity & relation metadata.
    #[oai(
        path = "/statistics",
        method = "get",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "fetchStatistics"
    )]
    async fn fetch_statistics(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        _token: CustomSecurityScheme,
    ) -> GetRecordResponse<Statistics> {
        info!("Username: {}", _token.0.username);
        let pool_arc = pool.clone();

        let entity_metadata = match EntityMetadata::get_entity_metadata(&pool_arc).await {
            Ok(entity_metadata) => entity_metadata,
            Err(e) => {
                let err = format!("Failed to fetch entity metadata: {}", e);
                warn!("{}", err);
                return GetRecordResponse::bad_request(err);
            }
        };

        let relation_metadata = match RelationMetadata::get_relation_metadata(&pool_arc).await {
            Ok(relation_metadata) => relation_metadata,
            Err(e) => {
                let err = format!("Failed to fetch relation metadata: {}", e);
                warn!("{}", err);
                return GetRecordResponse::bad_request(err);
            }
        };

        let statistics = Statistics::new(entity_metadata, relation_metadata);

        GetRecordResponse::ok(statistics)
    }

    /// Call `/api/v1/entity-metadata` with query params to fetch all entity metadata.
    #[oai(
        path = "/entity-metadata",
        method = "get",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "fetchEntityMetadata"
    )]
    async fn fetch_entity_metadata(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        _token: CustomSecurityScheme,
    ) -> GetWholeTableResponse<EntityMetadata> {
        let pool_arc = pool.clone();

        match EntityMetadata::get_entity_metadata(&pool_arc).await {
            Ok(entity_metadata) => GetWholeTableResponse::ok(entity_metadata),
            Err(e) => {
                let err = format!("Failed to fetch entity metadata: {}", e);
                warn!("{}", err);
                return GetWholeTableResponse::bad_request(err);
            }
        }
    }

    /// Call `/api/v1/entity-attr` with query params to fetch all entity attributes.
    #[oai(
        path = "/entity-attr",
        method = "get",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "fetchEntityAttributes"
    )]
    async fn fetch_entity_attributes(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        query_str: Query<Option<String>>,
        page: Query<Option<u64>>,
        page_size: Query<Option<u64>>,
        entity_type: Query<String>,
        _token: CustomSecurityScheme,
    ) -> GetRecordResponse<EntityAttr> {
        let pool_arc = pool.clone();
        let entity_type = entity_type.0;
        let page = page.0;
        let page_size = page_size.0;
        let query_str = query_str.0;

        let query = match query_str {
            Some(query_str) => match ComposeQuery::from_str(&query_str) {
                Ok(query) => query,
                Err(e) => {
                    let err = format!("Failed to parse query string: {}", e);
                    warn!("{}", err);
                    return GetRecordResponse::bad_request(err);
                }
            },
            None => None,
        };

        let order_by_clause = match query.clone() {
            Some(q) => {
                let pairs = get_all_field_pairs(&q);
                if pairs.len() == 0 {
                    "id ASC".to_string()
                } else {
                    // More fields will cause bad performance
                    make_order_clause_by_pairs(pairs, 1)
                }
            }
            None => "id ASC".to_string(),
        };

        match entity_type.to_lowercase().as_str() {
            "compound" => {
                match EntityAttrRecordResponse::<CompoundAttr>::fetch_records(
                    &pool_arc,
                    &query,
                    page,
                    page_size,
                    Some(order_by_clause.as_str()),
                )
                .await
                {
                    Ok(resp) => {
                        let resp = EntityAttr {
                            compounds: Some(resp),
                        };
                        GetRecordResponse::ok(resp)
                    }
                    Err(e) => {
                        let err = format!("Failed to fetch entity attributes: {}", e);
                        warn!("{}", err);
                        return GetRecordResponse::bad_request(err);
                    }
                }
            }
            _ => {
                let err = format!("Invalid entity type: {}", entity_type);
                warn!("{}", err);
                return GetRecordResponse::bad_request(err);
            }
        }
    }

    /// Call `/api/v1/entity-colormap` with query params to fetch all entity colormap.
    #[oai(
        path = "/entity-colormap",
        method = "get",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "fetchEntityColorMap"
    )]
    async fn fetch_entity_colormap(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        _token: CustomSecurityScheme,
    ) -> GetEntityColorMapResponse {
        let pool_arc = pool.clone();

        let entity_metadata = match EntityMetadata::get_entity_metadata(&pool_arc).await {
            Ok(entity_metadata) => entity_metadata,
            Err(e) => {
                let err = format!("Failed to fetch entity metadata: {}", e);
                warn!("{}", err);
                return GetEntityColorMapResponse::bad_request(err);
            }
        };

        let color_map = entity_metadata
            .iter()
            .map(|em| (em.entity_type.clone(), match_color(&em.entity_type)))
            .collect();

        return GetEntityColorMapResponse::ok(color_map);
    }

    /// Call `/api/v1/relation-metadata` with query params to fetch all relation metadata.
    #[oai(
        path = "/relation-metadata",
        method = "get",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "fetchRelationMetadata"
    )]
    async fn fetch_relation_metadata(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        _token: CustomSecurityScheme,
    ) -> GetWholeTableResponse<RelationMetadata> {
        let pool_arc = pool.clone();

        match RelationMetadata::get_relation_metadata(&pool_arc).await {
            Ok(relation_metadata) => GetWholeTableResponse::ok(relation_metadata),
            Err(e) => {
                let err = format!("Failed to fetch relation metadata: {}", e);
                warn!("{}", err);
                return GetWholeTableResponse::bad_request(err);
            }
        }
    }

    /// Call `/api/v1/entities` with query params to fetch entities.
    #[oai(
        path = "/entities",
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
        model_table_prefix: Query<Option<String>>, // A prefix of the entity embedding table name, such as "biomedgps"
        _token: CustomSecurityScheme,
    ) -> GetRecordsResponse<Entity> {
        let pool_arc = pool.clone();
        let page = page.0;
        let page_size = page_size.0;
        let model_table_prefix = model_table_prefix.0;
        let query_str = query_str.0;

        let query = match query_str {
            Some(query_str) => match ComposeQuery::from_str(&query_str) {
                Ok(query) => query,
                Err(e) => {
                    let err = format!("Failed to parse query string: {}", e);
                    warn!("{}", err);
                    return GetRecordsResponse::bad_request(err);
                }
            },
            None => None,
        };

        let order_by_clause = match query.clone() {
            Some(q) => {
                let pairs = get_all_field_pairs(&q);
                if pairs.len() == 0 {
                    "id ASC".to_string()
                } else {
                    // More fields will cause bad performance
                    make_order_clause_by_pairs(pairs, 1)
                }
            }
            None => "id ASC".to_string(),
        };

        let resp = if model_table_prefix.is_none() {
            match RecordResponse::<Entity>::get_records(
                &pool_arc,
                "biomedgps_entity",
                &query,
                page,
                page_size,
                Some(order_by_clause.as_str()),
                None,
            )
            .await
            {
                Ok(entities) => GetRecordsResponse::ok(entities),
                Err(e) => {
                    let err = format!("Failed to fetch entities: {}", e);
                    warn!("{}", err);
                    return GetRecordsResponse::bad_request(err);
                }
            }
        } else {
            match Entity::get_valid_records(
                &pool_arc,
                &model_table_prefix.unwrap(),
                &query,
                page,
                page_size,
                Some(order_by_clause.as_str()),
            )
            .await
            {
                Ok(entities) => GetRecordsResponse::ok(entities),
                Err(e) => {
                    let err = format!("Failed to fetch entities: {}", e);
                    warn!("{}", err);
                    return GetRecordsResponse::bad_request(err);
                }
            }
        };

        resp
    }

    /// Call `/api/v1/curated-graph` with query params to fetch curated graph.
    #[oai(
        path = "/curated-graph",
        method = "get",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "fetchCuratedGraph"
    )]
    async fn fetch_curated_graph(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        curator: Query<Option<String>>,
        project_id: Query<Option<String>>,
        organization_id: Query<Option<String>>,
        page: Query<Option<u64>>,
        page_size: Query<Option<u64>>,
        strict_mode: Query<bool>,
        _token: CustomSecurityScheme,
    ) -> GetGraphResponse {
        let pool_arc = pool.clone();
        let curator = match curator.0 {
            Some(curator) => curator,
            None => _token.0.username.clone(),
        };

        // if curator != _token.0.username {
        //     let err = format!(
        //         "You cannot query curated graph from other users. You are {} and you are querying {}'s curated graph.",
        //         _token.0.username, curator
        //     );
        //     warn!("{}", err);
        //     return GetGraphResponse::bad_request(err);
        // }

        let project_id = match project_id.0 {
            Some(project_id) => {
                // Convert project_id to i32
                match project_id.parse::<i32>() {
                    Ok(project_id) => project_id,
                    Err(e) => {
                        let err = format!("Failed to parse project id: {}", e);
                        warn!("{}", err);
                        return GetGraphResponse::bad_request(err);
                    }
                }
            }
            None => {
                warn!("Project id is empty.");
                -1
            }
        };

        let organization_id = match organization_id.0 {
            Some(organization_id) => {
                // Convert organization_id to i32
                match organization_id.parse::<i32>() {
                    Ok(organization_id) => organization_id,
                    Err(e) => {
                        let err = format!("Failed to parse organization id: {}", e);
                        warn!("{}", err);
                        return GetGraphResponse::bad_request(err);
                    }
                }
            }
            None => {
                warn!("Organization id is empty.");
                -1
            }
        };

        // Get organizations and projects from the token
        let user = &_token.0;
        if organization_id != -1 && !user.organizations.contains(&organization_id) {
            let err = format!(
                "User {} doesn't have access to organization {}. Your system might not support querying curated graph by organization or you don't have access to this organization.",
                user.username, organization_id
            );
            warn!("{}", err);
            return GetGraphResponse::bad_request(err);
        };

        if project_id != -1 && !user.projects.contains(&project_id) {
            let err = format!(
                "User {} doesn't have access to project {}. Your system might not support querying curated graph by project or you don't have access to this project.",
                user.username, project_id
            );
            warn!("{}", err);
            return GetGraphResponse::bad_request(err);
        };

        let mut graph = Graph::new();
        let page = page.0;
        let page_size = page_size.0;
        let strict_mode = strict_mode.0;

        match graph
            .fetch_curated_knowledges(
                &pool_arc,
                &curator[..],
                project_id,
                organization_id,
                page,
                page_size,
                Some("id ASC"),
                strict_mode,
            )
            .await
        {
            Ok(data) => GetGraphResponse::ok(data.to_owned().get_graph(None).unwrap()),
            Err(e) => {
                let err = format!("Failed to fetch curated graph: {}", e);
                warn!("{}", err);
                return GetGraphResponse::bad_request(err);
            }
        }
    }

    /// Call `/api/v1/curated-knowledges-by-owner` with query params to fetch curated knowledges by owner.
    #[oai(
        path = "/curated-knowledges-by-owner",
        method = "get",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "fetchCuratedKnowledgesByOwner"
    )]
    async fn fetch_curated_knowledges_by_owner(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        curator: Query<Option<String>>,
        fingerprint: Query<Option<String>>,
        project_id: Query<Option<String>>,
        organization_id: Query<Option<String>>,
        query_str: Query<Option<String>>,
        page: Query<Option<u64>>,
        page_size: Query<Option<u64>>,
        // We need to confirm the token is valid and contains all projects and organizations which the user has access to.
        _token: CustomSecurityScheme,
    ) -> GetRecordsResponse<KnowledgeCuration> {
        let pool_arc = pool.clone();
        let curator = curator.0;
        let fingerprint = fingerprint.0;
        let query_str = query_str.0;

        let curator = match curator {
            Some(curator) => {
                if curator != _token.0.username {
                    let err = format!(
                        "You cannot query curated knowledges from other users. You are {} and you are querying other users' curated knowledges.",
                        _token.0.username
                    );
                    warn!("{}", err);
                    return GetRecordsResponse::bad_request(err);
                } else {
                    curator
                }
            }
            None => _token.0.username.clone(),
        };

        let project_id = match project_id.0 {
            Some(project_id) => {
                // Convert project_id to i32
                match project_id.parse::<i32>() {
                    Ok(project_id) => project_id,
                    Err(e) => {
                        let err = format!("Failed to parse project id: {}", e);
                        warn!("{}", err);
                        return GetRecordsResponse::bad_request(err);
                    }
                }
            }
            None => {
                warn!("Project id is empty.");
                -1
            }
        };

        let organization_id = match organization_id.0 {
            Some(organization_id) => {
                // Convert organization_id to i32
                match organization_id.parse::<i32>() {
                    Ok(organization_id) => organization_id,
                    Err(e) => {
                        let err = format!("Failed to parse organization id: {}", e);
                        warn!("{}", err);
                        return GetRecordsResponse::bad_request(err);
                    }
                }
            }
            None => {
                warn!("Organization id is empty.");
                -1
            }
        };

        // Get organizations and projects from the token
        let user = &_token.0;
        if organization_id != -1 && !user.organizations.contains(&organization_id) {
            let err = format!(
                "User {} doesn't have access to organization {}. Your system might not support querying curated knowledges by organization or you don't have access to this organization.",
                user.username, organization_id
            );
            warn!("{}", err);
            return GetRecordsResponse::bad_request(err);
        };

        if project_id != -1 && !user.projects.contains(&project_id) {
            let err = format!(
                "User {} doesn't have access to project {}. Your system might not support querying curated knowledges by project or you don't have access to this project.",
                user.username, project_id
            );
            warn!("{}", err);
            return GetRecordsResponse::bad_request(err);
        };

        let query = match query_str {
            Some(query_str) => match ComposeQuery::from_str(&query_str) {
                Ok(query) => query,
                Err(e) => {
                    let err = format!("Failed to parse query string: {}", e);
                    warn!("{}", err);
                    return GetRecordsResponse::bad_request(err);
                }
            },
            None => None,
        };

        match KnowledgeCuration::get_records_by_owner(
            &pool_arc,
            &curator,
            fingerprint.as_deref(),
            project_id,
            organization_id,
            query,
            page.0,
            page_size.0,
            // TODO: get an order_by clause from query
            Some("id ASC"),
        )
        .await
        {
            Ok(entities) => GetRecordsResponse::ok(entities),
            Err(e) => {
                let err = format!("Failed to fetch curated knowledges: {}", e);
                warn!("{}", err);
                return GetRecordsResponse::bad_request(err);
            }
        }
    }

    /// Call `/api/v1/curated-knowledges` with query params to fetch curated knowledges.
    #[oai(
        path = "/curated-knowledges",
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
        _token: CustomSecurityScheme,
    ) -> GetRecordsResponse<KnowledgeCuration> {
        let pool_arc = pool.clone();
        let page = page.0;
        let page_size = page_size.0;
        let query_str = query_str.0;

        match PaginationQuery::new(page.clone(), page_size.clone(), query_str.clone()) {
            Ok(_) => {}
            Err(e) => {
                let err = format!("Failed to parse query string: {}", e);
                warn!("{}", err);
                return GetRecordsResponse::bad_request(err);
            }
        }

        let query = match query_str {
            Some(query_str) => match ComposeQuery::from_str(&query_str) {
                Ok(query) => query,
                Err(e) => {
                    let err = format!("Failed to parse query string: {}", e);
                    warn!("{}", err);
                    return GetRecordsResponse::bad_request(err);
                }
            },
            None => None,
        };

        match RecordResponse::<KnowledgeCuration>::get_records(
            &pool_arc,
            "biomedgps_knowledge_curation",
            &query,
            page,
            page_size,
            Some("id ASC"),
            None,
        )
        .await
        {
            Ok(entities) => GetRecordsResponse::ok(entities),
            Err(e) => {
                let err = format!("Failed to fetch curated knowledges: {}", e);
                warn!("{}", err);
                return GetRecordsResponse::bad_request(err);
            }
        }
    }

    /// Call `/api/v1/curated-knowledges` with payload to create a curated knowledge.
    #[oai(
        path = "/curated-knowledges",
        method = "post",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "postCuratedKnowledge"
    )]
    async fn post_curated_knowledge(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        payload: Json<KnowledgeCuration>,
        _token: CustomSecurityScheme,
    ) -> PostResponse<KnowledgeCuration> {
        let pool_arc = pool.clone();
        let mut payload = payload.0;

        // We don't like the owner field to be set by the frontend, so we set it to the curator.
        payload.update_curator(_token.0.username);

        match payload.validate() {
            Ok(_) => {}
            Err(e) => {
                let err = format!("Failed to validate payload: {}", e);
                warn!("{}", err);
                return PostResponse::bad_request(err);
            }
        };

        match payload.insert(&pool_arc).await {
            Ok(kc) => PostResponse::created(kc),
            Err(e) => {
                let err = format!("Failed to insert curated knowledge: {}", e);
                warn!("{}", err);
                return PostResponse::bad_request(err);
            }
        }
    }

    /// Call `/api/v1/curated-knowledges/:id` with payload to create a curated knowledge.
    #[oai(
        path = "/curated-knowledges/:id",
        method = "put",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "putCuratedKnowledge"
    )]
    async fn put_curated_knowledge(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        payload: Json<KnowledgeCuration>,
        id: Path<i64>,
        _token: CustomSecurityScheme,
    ) -> PostResponse<KnowledgeCuration> {
        let pool_arc = pool.clone();
        let mut payload = payload.0;
        let id = id.0;

        // We don't like the owner field to be set by the frontend, so we set it to the curator.
        payload.update_curator(_token.0.username);

        if id < 0 {
            let err = format!("Invalid id: {}", id);
            warn!("{}", err);
            return PostResponse::bad_request(err);
        }

        match payload.validate() {
            Ok(_) => {}
            Err(e) => {
                let err = format!("Failed to validate payload: {}", e);
                warn!("{}", err);
                return PostResponse::bad_request(err);
            }
        };

        match payload.update(&pool_arc, id).await {
            Ok(kc) => PostResponse::created(kc),
            Err(e) => {
                let err = format!("Failed to insert curated knowledge: {}", e);
                warn!("{}", err);
                return PostResponse::bad_request(err);
            }
        }
    }

    /// Call `/api/v1/curated-knowledges/:id` with payload to delete a curated knowledge.
    #[oai(
        path = "/curated-knowledges/:id",
        method = "delete",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "deleteCuratedKnowledge"
    )]
    async fn delete_curated_knowledge(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        id: Path<i64>,
        _token: CustomSecurityScheme,
    ) -> DeleteResponse {
        let pool_arc = pool.clone();
        let id = id.0;

        if id < 0 {
            let err = format!("Invalid id: {}", id);
            warn!("{}", err);
            return DeleteResponse::bad_request(err);
        }

        let username = _token.0.username;
        match KnowledgeCuration::delete(&pool_arc, id, &username).await {
            Ok(_) => DeleteResponse::no_content(),
            Err(e) => {
                let err = format!("Failed to delete curated knowledge: {}", e);
                warn!("{}", err);
                DeleteResponse::not_found(err)
            }
        }
    }

    /// Call `/api/v1/entity-curations` with query params to fetch entity curations.
    #[oai(
        path = "/entity-curations",
        method = "get",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "fetchEntityCuration"
    )]
    async fn fetch_entity_curation(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        page: Query<Option<u64>>,
        page_size: Query<Option<u64>>,
        query_str: Query<Option<String>>,
        _token: CustomSecurityScheme,
    ) -> GetRecordsResponse<EntityCuration> {
        let pool_arc = pool.clone();
        let page = page.0;
        let page_size = page_size.0;
        let query_str = query_str.0;

        match PaginationQuery::new(page.clone(), page_size.clone(), query_str.clone()) {
            Ok(_) => {}
            Err(e) => {
                let err = format!("Failed to parse query string: {}", e);
                warn!("{}", err);
                return GetRecordsResponse::bad_request(err);
            }
        }

        let query = match query_str {
            Some(query_str) => match ComposeQuery::from_str(&query_str) {
                Ok(query) => query,
                Err(e) => {
                    let err = format!("Failed to parse query string: {}", e);
                    warn!("{}", err);
                    return GetRecordsResponse::bad_request(err);
                }
            },
            None => None,
        };

        match RecordResponse::<EntityCuration>::get_records(
            &pool_arc,
            "biomedgps_entity_curation",
            &query,
            page,
            page_size,
            Some("id ASC"),
            None,
        )
        .await
        {
            Ok(entities) => GetRecordsResponse::ok(entities),
            Err(e) => {
                let err = format!("Failed to fetch entity curations: {}", e);
                warn!("{}", err);
                return GetRecordsResponse::bad_request(err);
            }
        }
    }

    /// Call `/api/v1/entity-curations-by-owner` with query params to fetch entity curations by owner.
    #[oai(
        path = "/entity-curations-by-owner",
        method = "get",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "fetchEntityCurationByOwner"
    )]
    async fn fetch_entity_curation_by_owner(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        fingerprint: Query<Option<String>>,
        project_id: Query<Option<String>>,
        organization_id: Query<Option<String>>,
        page: Query<Option<u64>>,
        page_size: Query<Option<u64>>,
        query_str: Query<Option<String>>,
        // We need to confirm the token is valid and contains all projects and organizations which the user has access to.
        _token: CustomSecurityScheme,
    ) -> GetRecordsResponse<EntityCuration> {
        let pool_arc = pool.clone();
        let curator = &_token.0.username;
        let query_str = query_str.0;
        let fingerprint = match &fingerprint.0 {
            Some(fingerprint) => fingerprint,
            None => {
                warn!("Fingerprint is empty.");
                ""
            }
        };

        let project_id = match project_id.0 {
            Some(project_id) => {
                // Convert project_id to i32
                match project_id.parse::<i32>() {
                    Ok(project_id) => project_id,
                    Err(e) => {
                        let err = format!("Failed to parse project id: {}", e);
                        warn!("{}", err);
                        return GetRecordsResponse::bad_request(err);
                    }
                }
            }
            None => {
                warn!("Project id is empty.");
                -1
            }
        };

        let organization_id = match organization_id.0 {
            Some(organization_id) => {
                // Convert organization_id to i32
                match organization_id.parse::<i32>() {
                    Ok(organization_id) => organization_id,
                    Err(e) => {
                        let err = format!("Failed to parse organization id: {}", e);
                        warn!("{}", err);
                        return GetRecordsResponse::bad_request(err);
                    }
                }
            }
            None => {
                warn!("Organization id is empty.");
                -1
            }
        };

        // Get organizations and projects from the token
        let user = &_token.0;
        if organization_id != -1 && !user.organizations.contains(&organization_id) {
            let err = format!(
                "User {} doesn't have access to organization {}. Your system might not support querying curated knowledges by organization or you don't have access to this organization.",
                user.username, organization_id
            );
            warn!("{}", err);
            return GetRecordsResponse::bad_request(err);
        };

        if project_id != -1 && !user.projects.contains(&project_id) {
            let err = format!(
                "User {} doesn't have access to project {}. Your system might not support querying curated knowledges by project or you don't have access to this project.",
                user.username, project_id
            );
            warn!("{}", err);
            return GetRecordsResponse::bad_request(err);
        };

        let query = match query_str {
            Some(query_str) => match ComposeQuery::from_str(&query_str) {
                Ok(query) => query,
                Err(e) => {
                    let err = format!("Failed to parse query string: {}", e);
                    warn!("{}", err);
                    return GetRecordsResponse::bad_request(err);
                }
            },
            None => None,
        };

        match EntityCuration::get_records_by_owner(
            &pool_arc,
            &curator,
            &fingerprint,
            project_id,
            organization_id,
            &query,
            page.0,
            page_size.0,
            // TODO: get an order_by clause from query
            Some("id ASC"),
        )
        .await
        {
            Ok(entities) => GetRecordsResponse::ok(entities),
            Err(e) => {
                let err = format!("Failed to fetch curated knowledges: {}", e);
                warn!("{}", err);
                return GetRecordsResponse::bad_request(err);
            }
        }
    }

    /// Call `/api/v1/entity-curations` with payload to create a entity curation.
    #[oai(
        path = "/entity-curations",
        method = "post",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "postEntityCuration"
    )]
    async fn post_entity_curation(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        payload: Json<EntityCuration>,
        _token: CustomSecurityScheme,
    ) -> PostResponse<EntityCuration> {
        let pool_arc = pool.clone();
        let mut payload = payload.0;

        let username = _token.0.username;
        payload.update_curator(&username);

        match payload.validate() {
            Ok(_) => {}
            Err(e) => {
                let err = format!("Failed to validate payload: {}", e);
                warn!("{}", err);
                return PostResponse::bad_request(err);
            }
        };

        match payload.insert(&pool_arc).await {
            Ok(ec) => PostResponse::created(ec),
            Err(e) => {
                let err = format!("Failed to insert entity curation: {}", e);
                warn!("{}", err);
                return PostResponse::bad_request(err);
            }
        }
    }

    /// Call `/api/v1/entity-curations/:id` with payload to update a entity curation.
    #[oai(
        path = "/entity-curations/:id",
        method = "put",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "putEntityCuration"
    )]
    async fn put_entity_curation(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        payload: Json<EntityCuration>,
        id: Path<i64>,
        _token: CustomSecurityScheme,
    ) -> PostResponse<EntityCuration> {
        let pool_arc = pool.clone();
        let mut payload = payload.0;
        let id = id.0;

        let username = _token.0.username;
        payload.update_curator(&username);

        if id < 0 {
            let err = format!("Invalid id: {}", id);
            warn!("{}", err);
            return PostResponse::bad_request(err);
        }

        match payload.validate() {
            Ok(_) => {}
            Err(e) => {
                let err = format!("Failed to validate payload: {}", e);
                warn!("{}", err);
                return PostResponse::bad_request(err);
            }
        };

        match payload.update(&pool_arc, id, &username).await {
            Ok(ec) => PostResponse::created(ec),
            Err(e) => {
                let err = format!("Failed to update entity curation: {}", e);
                warn!("{}", err);
                return PostResponse::bad_request(err);
            }
        }
    }

    /// Call `/api/v1/entity-curations` with payload to delete a entity curation.
    #[oai(
        path = "/entity-curations",
        method = "delete",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "deleteEntityCurationRecord"
    )]
    async fn delete_entity_curation_record(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        fingerprint: Query<String>,
        curator: Query<String>,
        entity_id: Query<String>,
        entity_type: Query<String>,
        entity_name: Query<String>,
        _token: CustomSecurityScheme,
    ) -> DeleteResponse {
        let pool_arc = pool.clone();
        let fingerprint = &fingerprint.0;
        let curator = &_token.0.username;
        let entity_id = &entity_id.0;
        let entity_type = &entity_type.0;
        let entity_name = &entity_name.0;

        match EntityCuration::delete_record(
            &pool_arc,
            &fingerprint,
            &curator,
            &entity_id,
            &entity_type,
            &entity_name,
        )
        .await
        {
            Ok(_) => DeleteResponse::no_content(),
            Err(e) => {
                let err = format!("Failed to delete entity curation: {}", e);
                warn!("{}", err);
                DeleteResponse::not_found(err)
            }
        }
    }

    /// Call `/api/v1/entity-curations/:id` with payload to delete a entity curation.
    #[oai(
        path = "/entity-curations/:id",
        method = "delete",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "deleteEntityCuration"
    )]
    async fn delete_entity_curation(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        id: Path<i64>,
        _token: CustomSecurityScheme,
    ) -> DeleteResponse {
        let pool_arc = pool.clone();
        let id = id.0;
        let username = _token.0.username;

        if id < 0 {
            let err = format!("Invalid id: {}", id);
            warn!("{}", err);
            return DeleteResponse::bad_request(err);
        }

        match EntityCuration::delete(&pool_arc, id, &username).await {
            Ok(_) => DeleteResponse::no_content(),
            Err(e) => {
                let err = format!("Failed to delete entity curation: {}", e);
                warn!("{}", err);
                DeleteResponse::not_found(err)
            }
        }
    }

    /// Call `/api/v1/entity-metadata-curations` with query params to fetch entity metadata curations.
    #[oai(
        path = "/entity-metadata-curations",
        method = "get",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "fetchEntityMetadataCuration"
    )]
    async fn fetch_entity_metadata_curation(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        page: Query<Option<u64>>,
        page_size: Query<Option<u64>>,
        query_str: Query<Option<String>>,
        _token: CustomSecurityScheme,
    ) -> GetRecordsResponse<EntityMetadataCuration> {
        let pool_arc = pool.clone();
        let page = page.0;
        let page_size = page_size.0;
        let query_str = query_str.0;

        match PaginationQuery::new(page.clone(), page_size.clone(), query_str.clone()) {
            Ok(_) => {}
            Err(e) => {
                let err = format!("Failed to parse query string: {}", e);
                warn!("{}", err);
                return GetRecordsResponse::bad_request(err);
            }
        }

        let query = match query_str {
            Some(query_str) => match ComposeQuery::from_str(&query_str) {
                Ok(query) => query,
                Err(e) => {
                    let err = format!("Failed to parse query string: {}", e);
                    warn!("{}", err);
                    return GetRecordsResponse::bad_request(err);
                }
            },
            None => None,
        };

        match RecordResponse::<EntityMetadataCuration>::get_records(
            &pool_arc,
            "biomedgps_entity_metadata_curation",
            &query,
            page,
            page_size,
            Some("id ASC"),
            None,
        )
        .await
        {
            Ok(entities) => GetRecordsResponse::ok(entities),
            Err(e) => {
                let err = format!("Failed to fetch entity metadata curations: {}", e);
                warn!("{}", err);
                return GetRecordsResponse::bad_request(err);
            }
        }
    }

    /// Call `/api/v1/entity-metadata-curations-by-owner` with query params to fetch entity metadata curations by owner.
    #[oai(
        path = "/entity-metadata-curations-by-owner",
        method = "get",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "fetchEntityMetadataCurationByOwner"
    )]
    async fn fetch_entity_metadata_curation_by_owner(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        fingerprint: Query<Option<String>>,
        project_id: Query<Option<String>>,
        organization_id: Query<Option<String>>,
        query_str: Query<Option<String>>,
        page: Query<Option<u64>>,
        page_size: Query<Option<u64>>,
        _token: CustomSecurityScheme,
    ) -> GetRecordsResponse<EntityMetadataCuration> {
        let pool_arc = pool.clone();
        let curator = &_token.0.username;
        let fingerprint = match &fingerprint.0 {
            Some(fingerprint) => fingerprint,
            None => {
                warn!("Fingerprint is empty.");
                ""
            }
        };
        let page = page.0;
        let page_size = page_size.0;
        let query_str = query_str.0;

        let project_id = match project_id.0 {
            Some(project_id) => {
                // Convert project_id to i32
                match project_id.parse::<i32>() {
                    Ok(project_id) => project_id,
                    Err(e) => {
                        let err = format!("Failed to parse project id: {}", e);
                        warn!("{}", err);
                        return GetRecordsResponse::bad_request(err);
                    }
                }
            }
            None => {
                warn!("Project id is empty.");
                -1
            }
        };

        let organization_id = match organization_id.0 {
            Some(organization_id) => {
                // Convert organization_id to i32
                match organization_id.parse::<i32>() {
                    Ok(organization_id) => organization_id,
                    Err(e) => {
                        let err = format!("Failed to parse organization id: {}", e);
                        warn!("{}", err);
                        return GetRecordsResponse::bad_request(err);
                    }
                }
            }
            None => {
                warn!("Organization id is empty.");
                -1
            }
        };

        // Get organizations and projects from the token
        let user = &_token.0;
        if organization_id != -1 && !user.organizations.contains(&organization_id) {
            let err = format!(
                "User {} doesn't have access to organization {}. Your system might not support querying curated knowledges by organization or you don't have access to this organization.",
                user.username, organization_id
            );
            warn!("{}", err);
            return GetRecordsResponse::bad_request(err);
        }

        if project_id != -1 && !user.projects.contains(&project_id) {
            let err = format!(
                "User {} doesn't have access to project {}. Your system might not support querying curated knowledges by project or you don't have access to this project.",
                user.username, project_id
            );
            warn!("{}", err);
            return GetRecordsResponse::bad_request(err);
        }

        let query = match query_str {
            Some(query_str) => match ComposeQuery::from_str(&query_str) {
                Ok(query) => query,
                Err(e) => {
                    let err = format!("Failed to parse query string: {}", e);
                    warn!("{}", err);
                    return GetRecordsResponse::bad_request(err);
                }
            },
            None => None,
        };

        match EntityMetadataCuration::get_records_by_owner(
            &pool_arc,
            &fingerprint,
            &curator,
            project_id,
            organization_id,
            query,
            page,
            page_size,
            Some("id ASC"),
        )
        .await
        {
            Ok(entities) => GetRecordsResponse::ok(entities),
            Err(e) => {
                let err = format!("Failed to fetch entity metadata curations: {}", e);
                warn!("{}", err);
                return GetRecordsResponse::bad_request(err);
            }
        }
    }

    /// Call `/api/v1/entity-metadata-curations` with payload to create a entity metadata curation.
    #[oai(
        path = "/entity-metadata-curations",
        method = "post",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "postEntityMetadataCuration"
    )]
    async fn post_entity_metadata_curation(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        payload: Json<EntityMetadataCuration>,
        _token: CustomSecurityScheme,
    ) -> PostResponse<EntityMetadataCuration> {
        let pool_arc = pool.clone();
        let mut payload = payload.0;
        let username = _token.0.username;
        payload.update_curator(&username);

        match payload.validate() {
            Ok(_) => {}
            Err(e) => {
                let err = format!("Failed to validate payload: {}", e);
                warn!("{}", err);
                return PostResponse::bad_request(err);
            }
        };

        match payload.insert(&pool_arc).await {
            Ok(emc) => PostResponse::created(emc),
            Err(e) => {
                let err = format!("Failed to insert entity metadata curation: {}", e);
                warn!("{}", err);
                return PostResponse::bad_request(err);
            }
        }
    }

    /// Call `/api/v1/entity-metadata-curations/:id` with payload to update a entity metadata curation.
    #[oai(
        path = "/entity-metadata-curations/:id",
        method = "put",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "putEntityMetadataCuration"
    )]
    async fn put_entity_metadata_curation(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        payload: Json<EntityMetadataCuration>,
        id: Path<i64>,
        _token: CustomSecurityScheme,
    ) -> PostResponse<EntityMetadataCuration> {
        let pool_arc = pool.clone();
        let mut payload = payload.0;
        let id = id.0;

        let username = _token.0.username;
        payload.update_curator(&username);

        if id < 0 {
            let err = format!("Invalid id: {}", id);
            warn!("{}", err);
            return PostResponse::bad_request(err);
        }

        match payload.validate() {
            Ok(_) => {}
            Err(e) => {
                let err = format!("Failed to validate payload: {}", e);
                warn!("{}", err);
                return PostResponse::bad_request(err);
            }
        };

        match payload.update(&pool_arc, id, &username).await {
            Ok(emc) => PostResponse::created(emc),
            Err(e) => {
                let err = format!("Failed to update entity metadata curation: {}", e);
                warn!("{}", err);
                return PostResponse::bad_request(err);
            }
        }
    }

    /// Call `/api/v1/entity-metadata-curations` with payload to delete a entity metadata curation.
    #[oai(
        path = "/entity-metadata-curations",
        method = "delete",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "deleteEntityMetadataCurationRecord"
    )]
    async fn delete_entity_metadata_curation_record(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        fingerprint: Query<String>,
        entity_id: Query<String>,
        entity_type: Query<String>,
        entity_name: Query<String>,
        field_name: Query<String>,
        field_value: Query<String>,
        _token: CustomSecurityScheme,
    ) -> DeleteResponse {
        let pool_arc = pool.clone();
        let fingerprint = fingerprint.0;
        let username = _token.0.username;
        let entity_id = entity_id.0;
        let entity_type = entity_type.0;
        let entity_name = entity_name.0;
        let field_name = field_name.0;
        let field_value = field_value.0;

        match EntityMetadataCuration::delete_record(
            &pool_arc,
            &fingerprint,
            &username,
            &entity_id,
            &entity_type,
            &entity_name,
            &field_name,
            &field_value,
        )
        .await
        {
            Ok(_) => DeleteResponse::no_content(),
            Err(e) => {
                let err = format!("Failed to delete entity metadata curation: {}", e);
                warn!("{}", err);
                DeleteResponse::not_found(err)
            }
        }
    }

    /// Call `/api/v1/entity-metadata-curations/:id` with payload to delete a entity metadata curation.
    #[oai(
        path = "/entity-metadata-curations/:id",
        method = "delete",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "deleteEntityMetadataCuration"
    )]
    async fn delete_entity_metadata_curation(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        id: Path<i64>,
        _token: CustomSecurityScheme,
    ) -> DeleteResponse {
        let pool_arc = pool.clone();
        let id = id.0;
        let username = _token.0.username;

        if id < 0 {
            let err = format!("Invalid id: {}", id);
            warn!("{}", err);
            return DeleteResponse::bad_request(err);
        }

        match EntityMetadataCuration::delete(&pool_arc, id, &username).await {
            Ok(_) => DeleteResponse::no_content(),
            Err(e) => {
                let err = format!("Failed to delete entity metadata curation: {}", e);
                warn!("{}", err);
                DeleteResponse::not_found(err)
            }
        }
    }

    /// Call `/api/v1/webpage-metadata` with query params to fetch webpage metadata.
    #[oai(
        path = "/webpage-metadata",
        method = "get",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "fetchWebpageMetadata"
    )]
    async fn fetch_webpage_metadata(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        page: Query<Option<u64>>,
        page_size: Query<Option<u64>>,
        query_str: Query<Option<String>>,
        _token: CustomSecurityScheme,
    ) -> GetRecordsResponse<WebpageMetadata> {
        let pool_arc = pool.clone();
        let page = page.0;
        let page_size = page_size.0;
        let query_str = query_str.0;

        match PaginationQuery::new(page.clone(), page_size.clone(), query_str.clone()) {
            Ok(_) => {}
            Err(e) => {
                let err = format!("Failed to parse query string: {}", e);
                warn!("{}", err);
                return GetRecordsResponse::bad_request(err);
            }
        }

        let query = match query_str {
            Some(query_str) => match ComposeQuery::from_str(&query_str) {
                Ok(query) => query,
                Err(e) => {
                    let err = format!("Failed to parse query string: {}", e);
                    warn!("{}", err);
                    return GetRecordsResponse::bad_request(err);
                }
            },
            None => None,
        };

        match RecordResponse::<WebpageMetadata>::get_records(
            &pool_arc,
            "biomedgps_webpage_metadata",
            &query,
            page,
            page_size,
            Some("id ASC"),
            None,
        )
        .await
        {
            Ok(records) => GetRecordsResponse::ok(records),
            Err(e) => {
                let err = format!("Failed to fetch webpage metadata: {}", e);
                warn!("{}", err);
                return GetRecordsResponse::bad_request(err);
            }
        }
    }

    /// Call `/api/v1/webpage-metadata` with payload to create a webpage metadata.
    #[oai(
        path = "/webpage-metadata",
        method = "post",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "postWebpageMetadata"
    )]
    async fn post_webpage_metadata(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        payload: Json<WebpageMetadata>,
        _token: CustomSecurityScheme,
    ) -> PostResponse<WebpageMetadata> {
        let pool_arc = pool.clone();
        let mut payload = payload.0;
        let username = _token.0.username;
        payload.update_curator(&username);

        match payload.validate() {
            Ok(_) => {}
            Err(e) => {
                let err = format!("Failed to validate payload: {}", e);
                warn!("{}", err);
                return PostResponse::bad_request(err);
            }
        }

        match payload.insert(&pool_arc).await {
            Ok(wm) => PostResponse::created(wm),
            Err(e) => {
                let err = format!("Failed to insert webpage metadata: {}", e);
                warn!("{}", err);
                return PostResponse::bad_request(err);
            }
        }
    }

    /// Call `/api/v1/webpage-metadata/:id` with payload to update a webpage metadata.
    #[oai(
        path = "/webpage-metadata/:id",
        method = "put",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "putWebpageMetadata"
    )]
    async fn put_webpage_metadata(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        payload: Json<WebpageMetadata>,
        id: Path<i64>,
        _token: CustomSecurityScheme,
    ) -> PostResponse<WebpageMetadata> {
        let pool_arc = pool.clone();
        let mut payload = payload.0;
        let id = id.0;
        let username = _token.0.username;
        payload.update_curator(&username);

        if id < 0 {
            let err = format!("Invalid id: {}", id);
            warn!("{}", err);
            return PostResponse::bad_request(err);
        }

        match payload.validate() {
            Ok(_) => {}
            Err(e) => {
                let err = format!("Failed to validate payload: {}", e);
                warn!("{}", err);
                return PostResponse::bad_request(err);
            }
        }

        match payload.update(&pool_arc, id, &username).await {
            Ok(wm) => PostResponse::created(wm),
            Err(e) => {
                let err = format!("Failed to update webpage metadata: {}", e);
                warn!("{}", err);
                return PostResponse::bad_request(err);
            }
        }
    }

    /// Call `/api/v1/webpage-metadata` with payload to delete a webpage metadata.
    #[oai(
        path = "/webpage-metadata",
        method = "delete",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "deleteWebpageMetadataByFingerprint"
    )]
    async fn delete_webpage_metadata_record(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        fingerprint: Query<String>,
        _token: CustomSecurityScheme,
    ) -> DeleteResponse {
        let pool_arc = pool.clone();
        let fingerprint = fingerprint.0;
        let username = _token.0.username;

        match WebpageMetadata::delete_record(&pool_arc, &fingerprint, &username).await {
            Ok(_) => DeleteResponse::no_content(),
            Err(e) => {
                let err = format!("Failed to delete webpage metadata: {}", e);
                warn!("{}", err);
                DeleteResponse::not_found(err)
            }
        }
    }

    /// Call `/api/v1/webpage-metadata/:id` with payload to delete a webpage metadata.
    #[oai(
        path = "/webpage-metadata/:id",
        method = "delete",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "deleteWebpageMetadata"
    )]
    async fn delete_webpage_metadata(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        id: Path<i64>,
        _token: CustomSecurityScheme,
    ) -> DeleteResponse {
        let pool_arc = pool.clone();
        let id = id.0;
        let username = _token.0.username;

        if id < 0 {
            let err = format!("Invalid id: {}", id);
            warn!("{}", err);
            return DeleteResponse::bad_request(err);
        }

        match WebpageMetadata::delete(&pool_arc, id, &username).await {
            Ok(_) => DeleteResponse::no_content(),
            Err(e) => {
                let err = format!("Failed to delete webpage metadata: {}", e);

                warn!("{}", err);
                DeleteResponse::not_found(err)
            }
        }
    }

    /// Call `/api/v1/key-sentence-curations` with query params to fetch key sentence curations.
    #[oai(
        path = "/key-sentence-curations",
        method = "get",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "fetchKeySentenceCuration"
    )]
    async fn fetch_key_sentence_curation(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        page: Query<Option<u64>>,
        page_size: Query<Option<u64>>,
        query_str: Query<Option<String>>,
        _token: CustomSecurityScheme,
    ) -> GetRecordsResponse<KeySentenceCuration> {
        let pool_arc = pool.clone();
        let page = page.0;
        let page_size = page_size.0;
        let query_str = query_str.0;

        match PaginationQuery::new(page.clone(), page_size.clone(), query_str.clone()) {
            Ok(_) => {}
            Err(e) => {
                let err = format!("Failed to parse query string: {}", e);
                warn!("{}", err);
                return GetRecordsResponse::bad_request(err);
            }
        }

        let query = match query_str {
            Some(query_str) => match ComposeQuery::from_str(&query_str) {
                Ok(query) => query,
                Err(e) => {
                    let err = format!("Failed to parse query string: {}", e);
                    warn!("{}", err);
                    return GetRecordsResponse::bad_request(err);
                }
            },
            None => None,
        };

        match RecordResponse::<KeySentenceCuration>::get_records(
            &pool_arc,
            "biomedgps_key_sentence_curation",
            &query,
            page,
            page_size,
            Some("id ASC"),
            None,
        )
        .await
        {
            Ok(records) => GetRecordsResponse::ok(records),
            Err(e) => {
                let err = format!("Failed to fetch key sentence curations: {}", e);
                warn!("{}", err);
                return GetRecordsResponse::bad_request(err);
            }
        }
    }

    /// Call `/api/v1/key-sentence-curations-by-owner` with query params to fetch key sentence curations by owner.
    #[oai(
        path = "/key-sentence-curations-by-owner",
        method = "get",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "fetchKeySentenceCurationByOwner"
    )]
    async fn fetch_key_sentence_curation_by_owner(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        curator: Query<Option<String>>,
        fingerprint: Query<Option<String>>,
        project_id: Query<Option<String>>,
        organization_id: Query<Option<String>>,
        page: Query<Option<u64>>,
        page_size: Query<Option<u64>>,
        query_str: Query<Option<String>>,
        // We need to confirm the token is valid and contains all projects and organizations which the user has access to.
        _token: CustomSecurityScheme,
    ) -> GetRecordsResponse<KeySentenceCuration> {
        let pool_arc = pool.clone();
        let curator = curator.0;
        let query_str = query_str.0;

        let curator = match curator {
            Some(curator) => {
                if curator != _token.0.username {
                    let err = format!(
                        "You cannot query curated key sentences from other users. You are {} and you are querying other users' curated key sentences.",
                        _token.0.username
                    );
                    warn!("{}", err);
                    return GetRecordsResponse::bad_request(err);
                } else {
                    curator
                }
            }
            None => _token.0.username.clone(),
        };

        let project_id = match project_id.0 {
            Some(project_id) => {
                // Convert project_id to i32
                match project_id.parse::<i32>() {
                    Ok(project_id) => project_id,
                    Err(e) => {
                        let err = format!("Failed to parse project id: {}", e);
                        warn!("{}", err);
                        return GetRecordsResponse::bad_request(err);
                    }
                }
            }
            None => {
                warn!("Project id is empty.");
                -1
            }
        };

        let organization_id = match organization_id.0 {
            Some(organization_id) => {
                // Convert organization_id to i32
                match organization_id.parse::<i32>() {
                    Ok(organization_id) => organization_id,
                    Err(e) => {
                        let err = format!("Failed to parse organization id: {}", e);
                        warn!("{}", err);
                        return GetRecordsResponse::bad_request(err);
                    }
                }
            }
            None => {
                warn!("Organization id is empty.");
                -1
            }
        };

        // Get organizations and projects from the token
        let user = &_token.0;
        if organization_id != -1 && !user.organizations.contains(&organization_id) {
            let err = format!(
                "User {} doesn't have access to organization {}. Your system might not support querying curated knowledges by organization or you don't have access to this organization.",
                user.username, organization_id
            );
            warn!("{}", err);
            return GetRecordsResponse::bad_request(err);
        };

        if project_id != -1 && !user.projects.contains(&project_id) {
            let err = format!(
                "User {} doesn't have access to project {}. Your system might not support querying curated knowledges by project or you don't have access to this project.",
                user.username, project_id
            );
            warn!("{}", err);
            return GetRecordsResponse::bad_request(err);
        };

        let fingerprint = match fingerprint.0 {
            Some(fingerprint) => fingerprint,
            None => {
                warn!("Fingerprint is empty.");
                "".to_string()
            }
        };

        let query = match query_str {
            Some(query_str) => match ComposeQuery::from_str(&query_str) {
                Ok(query) => query,
                Err(e) => {
                    let err = format!("Failed to parse query string: {}", e);
                    warn!("{}", err);
                    return GetRecordsResponse::bad_request(err);
                }
            },
            None => None,
        };

        match KeySentenceCuration::get_records_by_owner(
            &pool_arc,
            &fingerprint,
            &curator,
            project_id,
            organization_id,
            &query,
            page.0,
            page_size.0,
            // TODO: get an order_by clause from query
            Some("id ASC"),
        )
        .await
        {
            Ok(records) => GetRecordsResponse::ok(records),
            Err(e) => {
                let err = format!("Failed to fetch curated knowledges: {}", e);
                warn!("{}", err);
                return GetRecordsResponse::bad_request(err);
            }
        }
    }

    /// Call `/api/v1/key-sentence-curations` with payload to create a key sentence curation.
    #[oai(
        path = "/key-sentence-curations",
        method = "post",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "postKeySentenceCuration"
    )]
    async fn post_key_sentence_curation(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        payload: Json<KeySentenceCuration>,
        _token: CustomSecurityScheme,
    ) -> PostResponse<KeySentenceCuration> {
        let pool_arc = pool.clone();
        let mut payload = payload.0;
        let username = _token.0.username;
        payload.update_curator(&username);

        match payload.validate() {
            Ok(_) => {}
            Err(e) => {
                let err = format!("Failed to validate payload: {}", e);
                warn!("{}", err);
                return PostResponse::bad_request(err);
            }
        }

        match payload.insert(&pool_arc).await {
            Ok(ksc) => PostResponse::created(ksc),
            Err(e) => {
                let err = format!("Failed to insert key sentence curation: {}", e);
                warn!("{}", err);
                return PostResponse::bad_request(err);
            }
        }
    }

    /// Call `/api/v1/key-sentence-curations/:id` with payload to update a key sentence curation.
    #[oai(
        path = "/key-sentence-curations/:id",
        method = "put",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "putKeySentenceCuration"
    )]
    async fn put_key_sentence_curation(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        payload: Json<KeySentenceCuration>,
        id: Path<i64>,
        _token: CustomSecurityScheme,
    ) -> PostResponse<KeySentenceCuration> {
        let pool_arc = pool.clone();
        let mut payload = payload.0;
        let id = id.0;
        let username = _token.0.username;
        payload.update_curator(&username);

        if id < 0 {
            let err = format!("Invalid id: {}", id);
            warn!("{}", err);
            return PostResponse::bad_request(err);
        }

        match payload.validate() {
            Ok(_) => {}
            Err(e) => {
                let err = format!("Failed to validate payload: {}", e);
                warn!("{}", err);
                return PostResponse::bad_request(err);
            }
        }

        match payload.update(&pool_arc, id, &username).await {
            Ok(ksc) => PostResponse::created(ksc),
            Err(e) => {
                let err = format!("Failed to update key sentence curation: {}", e);
                warn!("{}", err);
                return PostResponse::bad_request(err);
            }
        }
    }

    /// Call `/api/v1/key-sentence-curations/:id/images` with payload to add an image to a key sentence curation.
    #[oai(
        path = "/key-sentence-curations/:id/images",
        method = "post",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "postKeySentenceCurationImage"
    )]
    async fn post_key_sentence_curation_image(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        id: Path<i64>,
        upload_image: UploadImage,
        _token: CustomSecurityScheme,
    ) -> PostResponse<KeySentenceCuration> {
        let pool_arc = pool.clone();
        let id = id.0;
        let username = _token.0.username;

        if id < 0 {
            let err = format!("Invalid id: {}", id);
            warn!("{}", err);
            return PostResponse::bad_request(err);
        }

        let destdir = match std::env::var("UPLOAD_DIR") {
            Ok(upload_dir) => PathBuf::from(upload_dir),
            Err(e) => {
                let err = format!("Failed to get upload directory: {}", e);
                warn!("{}", err);
                return PostResponse::bad_request(err);
            }
        };

        let image = upload_image.image;
        let filename = match image.file_name() {
            Some(filename) => filename.to_string(),
            None => "".to_string(),
        };

        let mime_type = match image.content_type() {
            Some(mime) => mime.to_string(),
            None => {
                let err = format!("Failed to get image content type");
                warn!("{}", err);
                return PostResponse::bad_request(err);
            }
        };

        let image_bytes = match image.into_vec().await {
            Ok(image_bytes) => image_bytes,
            Err(e) => {
                let err = format!("Failed to get image bytes: {}", e);
                warn!("{}", err);
                return PostResponse::bad_request(err);
            }
        };

        let image = match Image::upload(
            &destdir,
            &filename,
            &image_bytes,
            &mime_type,
            &upload_image.raw_image_url,
            &upload_image.raw_image_src,
        ) {
            Ok(image) => image,
            Err(e) => {
                let err = format!("Failed to upload image: {}", e);
                warn!("{}", err);
                return PostResponse::bad_request(err);
            }
        };

        match KeySentenceCuration::add_image_to_payload(&pool_arc, id, &username, &image).await {
            Ok(ksc) => return PostResponse::created(ksc),
            Err(e) => {
                let err = format!("Failed to add image to key sentence curation: {}", e);
                warn!("{}", err);
                return PostResponse::bad_request(err);
            }
        }
    }

    /// Call `/api/v1/key-sentence-curations` with payload to delete a key sentence curation.
    #[oai(
        path = "/key-sentence-curations",
        method = "delete",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "deleteKeySentenceCurationByFingerprint"
    )]
    async fn delete_key_sentence_curation_record(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        fingerprint: Query<String>,
        key_sentence: Query<String>,
        _token: CustomSecurityScheme,
    ) -> DeleteResponse {
        let pool_arc = pool.clone();
        let fingerprint = fingerprint.0;
        let key_sentence = key_sentence.0;
        let curator = _token.0.username;

        match KeySentenceCuration::delete_record(&pool_arc, &fingerprint, &curator, &key_sentence)
            .await
        {
            Ok(_) => DeleteResponse::no_content(),
            Err(e) => {
                let err = format!("Failed to delete key sentence curation: {}", e);
                warn!("{}", err);
                DeleteResponse::not_found(err)
            }
        }
    }

    /// Call `/api/v1/key-sentence-curations/:id` with payload to delete a key sentence curation.
    #[oai(
        path = "/key-sentence-curations/:id",
        method = "delete",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "deleteKeySentenceCuration"
    )]
    async fn delete_key_sentence_curation(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        id: Path<i64>,
        _token: CustomSecurityScheme,
    ) -> DeleteResponse {
        let pool_arc = pool.clone();
        let id = id.0;
        let username = _token.0.username;

        if id < 0 {
            let err = format!("Invalid id: {}", id);
            warn!("{}", err);
            return DeleteResponse::bad_request(err);
        }

        match KeySentenceCuration::delete(&pool_arc, id, &username).await {
            Ok(_) => DeleteResponse::no_content(),
            Err(e) => {
                let err = format!("Failed to delete key sentence curation: {}", e);
                warn!("{}", err);
                DeleteResponse::not_found(err)
            }
        }
    }

    /// Call `/api/v1/embeddings` with query params to fetch embeddings.
    #[oai(
        path = "/embeddings",
        method = "get",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "fetchEmbeddings"
    )]
    async fn fetch_embeddings(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        question: Query<String>,
        text_source_type: Query<String>,
        top_k: Query<usize>,
        _token: CustomSecurityScheme,
    ) -> GetRecordsResponse<Embedding> {
        let pool_arc = pool.clone();
        let question = question.0;
        let text_source_type = text_source_type.0;
        let top_k = top_k.0;

        let username = _token.0.username;

        match Embedding::get_records(
            &pool_arc,
            &question,
            &text_source_type,
            None,
            &username,
            top_k,
        )
        .await
        {
            Ok(records) => GetRecordsResponse::ok(RecordResponse {
                total: records.len() as u64,
                records: records,
                page: 1,
                page_size: top_k as u64,
            }),
            Err(e) => {
                let err = format!("Failed to fetch embeddings: {}", e);
                warn!("{}", err);
                return GetRecordsResponse::bad_request(err);
            }
        }
    }

    /// Call `/api/v1/workflows` with query params to fetch workflows.
    #[oai(
        path = "/workflows",
        method = "get",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "fetchWorkflows"
    )]
    async fn fetch_workflows(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        page: Query<Option<u64>>,
        page_size: Query<Option<u64>>,
        query_str: Query<Option<String>>,
        _token: CustomSecurityScheme,
    ) -> GetRecordsResponse<Workflow> {
        let pool_arc = pool.clone();
        let page = page.0;
        let page_size = page_size.0;
        let query_str = query_str.0;

        match PaginationQuery::new(page.clone(), page_size.clone(), query_str.clone()) {
            Ok(_) => {}
            Err(e) => {
                let err = format!("Failed to parse query string: {}", e);
                warn!("{}", err);
                return GetRecordsResponse::bad_request(err);
            }
        }

        let query = match query_str {
            Some(query_str) => match ComposeQuery::from_str(&query_str) {
                Ok(query) => query,
                Err(e) => {
                    let err = format!("Failed to parse query string: {}", e);
                    warn!("{}", err);
                    return GetRecordsResponse::bad_request(err);
                }
            },
            None => None,
        };

        match RecordResponse::<Workflow>::get_records(
            &pool_arc,
            "biomedgps_workflow",
            &query,
            page,
            page_size,
            Some("id ASC"),
            None,
        )
        .await
        {
            Ok(records) => GetRecordsResponse::ok(records),
            Err(e) => {
                let err = format!("Failed to fetch workflows: {}", e);
                warn!("{}", err);
                return GetRecordsResponse::bad_request(err);
            }
        }
    }

    /// Call `/api/v1/workflows/:id/schema` with query params to fetch workflow schema.
    #[oai(
        path = "/workflows/:id/schema",
        method = "get",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "fetchWorkflowSchema"
    )]
    async fn fetch_workflow_schema(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        id: Path<String>,
        _token: CustomSecurityScheme,
    ) -> GetRecordResponse<WorkflowSchema> {
        let pool_arc = pool.clone();
        let id = id.0;

        let workflow_root_dir = match std::env::var("WORKFLOW_ROOT_DIR") {
            Ok(workflow_root_dir) => PathBuf::from(workflow_root_dir),
            Err(e) => {
                let err = format!("The WORKFLOW_ROOT_DIR environment variable is not set: {}", e);
                warn!("{}", err);
                return GetRecordResponse::internal_server_error(err);
            }
        };

        match Workflow::get_workflow_schema(&pool_arc, &id, &workflow_root_dir).await {
            Ok(schema) => GetRecordResponse::ok(schema),
            Err(e) => {
                let err = format!("Failed to fetch workflow schema: {}", e);
                warn!("{}", err);
                return GetRecordResponse::internal_server_error(err);
            }
        }
    }

    /// Call `/api/v1/workspaces` with query params to fetch workspaces.
    #[oai(
        path = "/workspaces",
        method = "get",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "fetchWorkspaces"
    )]
    async fn fetch_workspaces(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        page: Query<Option<u64>>,
        page_size: Query<Option<u64>>,
        _token: CustomSecurityScheme,
    ) -> GetRecordsResponse<Workspace> {
        let pool_arc = pool.clone();
        let page = page.0;
        let page_size = page_size.0;
        let username = _token.0.username;

        match RecordResponse::<Workspace>::get_records(
            &pool_arc,
            "biomedgps_workspace",
            &None,
            page,
            page_size,
            Some("created_time ASC"),
            Some(&username),
        )
        .await
        {
            Ok(resp) => {
                if resp.records.is_empty() {
                    // We always want to return at least one workspace for the user.
                    match Workspace::insert_record(
                        &pool_arc,
                        "Default Workspace",
                        None,
                        &username,
                        None,
                    )
                    .await
                    {
                        Ok(workspace) => GetRecordsResponse::ok(RecordResponse {
                            total: 1,
                            records: vec![workspace],
                            page: 1,
                            page_size: 10,
                        }),
                        Err(e) => {
                            let err = format!("Failed to insert default workspace: {}", e);
                            warn!("{}", err);
                            return GetRecordsResponse::internal_server_error(err);
                        }
                    }
                } else {
                    GetRecordsResponse::ok(resp)
                }
            }
            Err(e) => {
                let err = format!("Failed to fetch workspaces: {}", e);
                warn!("{}", err);
                return GetRecordsResponse::bad_request(err);
            }
        }
    }

    /// Call `/api/v1/tasks` with query params to fetch tasks.
    #[oai(
        path = "/tasks",
        method = "get",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "fetchTasks"
    )]
    async fn fetch_tasks(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        page: Query<Option<u64>>,
        page_size: Query<Option<u64>>,
        query_str: Query<Option<String>>,
        _token: CustomSecurityScheme,
    ) -> GetRecordsResponse<Task> {
        let pool_arc = pool.clone();
        let page = page.0;
        let page_size = page_size.0;
        let query_str = query_str.0;
        let username = _token.0.username;

        match PaginationQuery::new(page.clone(), page_size.clone(), query_str.clone()) {
            Ok(_) => {}
            Err(e) => {
                let err = format!("Failed to parse query string: {}", e);
                warn!("{}", err);
                return GetRecordsResponse::bad_request(err);
            }
        }

        let query = match query_str {
            Some(query_str) => match ComposeQuery::from_str(&query_str) {
                Ok(query) => query,
                Err(e) => {
                    let err = format!("Failed to parse query string: {}", e);
                    warn!("{}", err);
                    return GetRecordsResponse::bad_request(err);
                }
            },
            None => None,
        };

        match RecordResponse::<Task>::get_records(
            &pool_arc,
            "biomedgps_task",
            &query,
            page,
            page_size,
            Some("id ASC"),
            Some(&username),
        )
        .await
        {
            Ok(records) => GetRecordsResponse::ok(records),
            Err(e) => {
                let err = format!("Failed to fetch tasks: {}", e);
                warn!("{}", err);
                return GetRecordsResponse::bad_request(err);
            }
        }
    }

    /// Call `/api/v1/tasks/:task_id` with query params to fetch task by task_id.
    #[oai(
        path = "/tasks/:task_id",
        method = "get",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "fetchTaskByTaskId"
    )]
    async fn fetch_task_by_task_id(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        task_id: Path<String>,
        _token: CustomSecurityScheme,
    ) -> GetRecordResponse<ExpandedTask> {
        let pool_arc = pool.clone();
        let task_id = task_id.0;
        let username = _token.0.username;

        let task_root_dir = match std::env::var("TASK_ROOT_DIR") {
            Ok(task_root_dir) => PathBuf::from(task_root_dir),
            Err(e) => {
                let err = format!("The TASK_DIR environment variable is not set: {}", e);
                warn!("{}", err);
                return GetRecordResponse::internal_server_error(err);
            }
        };

        match ExpandedTask::get_records_by_id(&pool_arc, &task_id, &username, &task_root_dir, true)
            .await
        {
            Ok(task) => GetRecordResponse::ok(task),
            Err(e) => {
                let err = format!("Failed to fetch task by task_id: {}", e);
                warn!("{}", err);
                return GetRecordResponse::bad_request(err);
            }
        }
    }

    /// Call `/api/v1/tasks/:task_id/log` with query params to fetch log by task_id.
    #[oai(
        path = "/tasks/:task_id/log",
        method = "get",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "fetchLogByTaskId"
    )]
    async fn fetch_log_by_task_id(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        task_id: Path<String>,
        _token: CustomSecurityScheme,
    ) -> GetRecordResponse<LogMessage> {
        let pool_arc = pool.clone();
        let task_id = task_id.0;
        let username = _token.0.username;

        let task_root_dir = match std::env::var("TASK_ROOT_DIR") {
            Ok(task_root_dir) => PathBuf::from(task_root_dir),
            Err(e) => {
                let err = format!("The TASK_ROOT_DIR environment variable is not set: {}", e);
                warn!("{}", err);
                return GetRecordResponse::internal_server_error(err);
            }
        };

        match ExpandedTask::get_log(&pool_arc, &username, &task_root_dir, &task_id).await {
            Ok(log) => GetRecordResponse::ok(LogMessage::new(log)),
            Err(e) => {
                let err = format!("Failed to fetch log by task_id: {}", e);
                warn!("{}", err);
                return GetRecordResponse::bad_request(err);
            }
        }
    }

    /// Call `/api/v1/tasks/:task_id/file` with query params to fetch file by file_name.
    #[oai(
        path = "/tasks/:task_id/file",
        method = "get",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "fetchFileByFileName"
    )]
    async fn fetch_file_by_file_name(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        task_id: Path<String>,
        file_name: Query<String>,
        _token: CustomSecurityScheme,
    ) -> FileResponse {
        let pool_arc = pool.clone();
        let task_id = task_id.0;
        let file_name = file_name.0;
        let username = _token.0.username;

        let task_root_dir = match std::env::var("TASK_ROOT_DIR") {
            Ok(task_root_dir) => PathBuf::from(task_root_dir),
            Err(e) => {
                let err = format!("The TASK_ROOT_DIR environment variable is not set: {}", e);
                warn!("{}", err);
                return FileResponse::internal_server_error(err);
            }
        };

        match ExpandedTask::get_file(&pool_arc, &username, &task_root_dir, &task_id, &file_name)
            .await
        {
            Ok(file_path) => {
                let mut buffer = Vec::new();
                let mut file = match tokio::fs::File::open(file_path).await {
                    Ok(file) => file,
                    Err(e) => {
                        let err = format!("Failed to open file: {}", e);
                        warn!("{}", err);
                        return FileResponse::internal_server_error(err);
                    }
                };

                if let Err(e) = file.read_to_end(&mut buffer).await {
                    let err = format!("Failed to read file: {}", e);
                    warn!("{}", err);
                    return FileResponse::internal_server_error(err);
                }

                FileResponse::file(buffer)
            }
            Err(e) => {
                warn!("{}", e);
                FileResponse::bad_request(e.to_string())
            }
        }
    }

    /// Call `/api/v1/tasks` with payload to create a task.
    #[oai(
        path = "/tasks",
        method = "post",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "postTask"
    )]
    async fn post_task(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        payload: Json<Task>,
        _token: CustomSecurityScheme,
    ) -> PostResponse<Task> {
        let pool_arc = pool.clone();
        let mut payload = payload.0;
        let username = _token.0.username;
        payload.update_owner(username);

        match payload.validate() {
            Ok(_) => {}
            Err(e) => {
                let err = format!("Failed to validate task: {}", e);
                warn!("{}", err);
                return PostResponse::bad_request(err);
            }
        }

        match payload.insert(&pool_arc).await {
            Ok(task) => PostResponse::created(task),
            Err(e) => {
                let err = format!("Failed to insert task: {}", e);
                warn!("{}", err);
                return PostResponse::bad_request(err);
            }
        }
    }

    /// Call `/api/v1/notifications` with query params to fetch notifications.
    #[oai(
        path = "/notifications",
        method = "get",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "fetchNotifications"
    )]
    async fn fetch_notifications(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        page: Query<Option<u64>>,
        page_size: Query<Option<u64>>,
        query_str: Query<Option<String>>,
        _token: CustomSecurityScheme,
    ) -> GetRecordsResponse<Notification> {
        let pool_arc = pool.clone();
        let page = page.0;
        let page_size = page_size.0;
        let query_str = query_str.0;

        match PaginationQuery::new(page.clone(), page_size.clone(), query_str.clone()) {
            Ok(_) => {}
            Err(e) => {
                let err = format!("Failed to parse query string: {}", e);
                warn!("{}", err);
                return GetRecordsResponse::bad_request(err);
            }
        }

        let query = match query_str {
            Some(query_str) => match ComposeQuery::from_str(&query_str) {
                Ok(query) => query,
                Err(e) => {
                    let err = format!("Failed to parse query string: {}", e);
                    warn!("{}", err);
                    return GetRecordsResponse::bad_request(err);
                }
            },
            None => None,
        };

        match RecordResponse::<Notification>::get_records(
            &pool_arc,
            "biomedgps_notification",
            &query,
            page,
            page_size,
            Some("id ASC"),
            None,
        )
        .await
        {
            Ok(records) => GetRecordsResponse::ok(records),
            Err(e) => {
                let err = format!("Failed to fetch notifications: {}", e);
                warn!("{}", err);
                return GetRecordsResponse::bad_request(err);
            }
        }
    }

    /// Call `/api/v1/configurations` with query params to fetch configurations.
    #[oai(
        path = "/configurations",
        method = "get",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "fetchConfigurations"
    )]
    async fn fetch_configurations(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        page: Query<Option<u64>>,
        page_size: Query<Option<u64>>,
        query_str: Query<Option<String>>,
        _token: CustomSecurityScheme,
    ) -> GetRecordsResponse<Configuration> {
        let pool_arc = pool.clone();
        let page = page.0;
        let page_size = page_size.0;
        let query_str = query_str.0;

        match PaginationQuery::new(page.clone(), page_size.clone(), query_str.clone()) {
            Ok(_) => {}
            Err(e) => {
                let err = format!("Failed to parse query string: {}", e);
                warn!("{}", err);
                return GetRecordsResponse::bad_request(err);
            }
        }

        let query = match query_str {
            Some(query_str) => match ComposeQuery::from_str(&query_str) {
                Ok(query) => query,
                Err(e) => {
                    let err = format!("Failed to parse query string: {}", e);
                    warn!("{}", err);
                    return GetRecordsResponse::bad_request(err);
                }
            },
            None => None,
        };

        match RecordResponse::<Configuration>::get_records(
            &pool_arc,
            "biomedgps_configuration",
            &query,
            page,
            page_size,
            Some("id ASC"),
            None,
        )
        .await
        {
            Ok(records) => GetRecordsResponse::ok(records),
            Err(e) => {
                let err = format!("Failed to fetch configurations: {}", e);
                warn!("{}", err);
                return GetRecordsResponse::bad_request(err);
            }
        }
    }

    /// Call `/api/v1/configurations` with payload to create a configuration.
    #[oai(
        path = "/configurations",
        method = "post",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "postConfiguration"
    )]
    async fn post_configuration(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        payload: Json<Configuration>,
        _token: CustomSecurityScheme,
    ) -> PostResponse<Configuration> {
        let pool_arc = pool.clone();
        let mut payload = payload.0;
        let username = _token.0.username;
        payload.update_owner(&username);

        match payload.validate() {
            Ok(_) => {}
            Err(e) => {
                let err = format!("Failed to validate configuration: {}", e);
                warn!("{}", err);
                return PostResponse::bad_request(err);
            }
        }

        match payload.insert(&pool_arc).await {
            Ok(config) => PostResponse::created(config),
            Err(e) => {
                let err = format!("Failed to insert configuration: {}", e);
                warn!("{}", err);
                return PostResponse::bad_request(err);
            }
        }
    }

    /// Call `/api/v1/configurations/:id` with payload to update a configuration.
    #[oai(
        path = "/configurations/:id",
        method = "put",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "putConfiguration"
    )]
    async fn put_configuration(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        payload: Json<Configuration>,
        id: Path<i64>,
        _token: CustomSecurityScheme,
    ) -> PostResponse<Configuration> {
        let pool_arc = pool.clone();
        let mut payload = payload.0;
        let id = id.0;
        let username = _token.0.username;
        payload.update_owner(&username);

        if id < 0 {
            let err = format!("Invalid id: {}", id);
            warn!("{}", err);
            return PostResponse::bad_request(err);
        }

        match payload.validate() {
            Ok(_) => {}
            Err(e) => {
                let err = format!("Failed to validate configuration: {}", e);
                warn!("{}", err);
                return PostResponse::bad_request(err);
            }
        }

        match payload.update(&pool_arc, id, &username).await {
            Ok(config) => PostResponse::created(config),
            Err(e) => {
                let err = format!("Failed to update configuration: {}", e);
                warn!("{}", err);
                return PostResponse::bad_request(err);
            }
        }
    }

    /// Call `/api/v1/configurations` with payload to delete a configuration.
    #[oai(
        path = "/configurations",
        method = "delete",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "deleteConfiguration"
    )]
    async fn delete_configuration(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        config_name: Query<String>,
        category: Query<String>,
        _token: CustomSecurityScheme,
    ) -> DeleteResponse {
        let pool_arc = pool.clone();
        let config_name = config_name.0;
        let category = category.0;
        let username = _token.0.username;

        if config_name == "" || category == "" {
            let err = format!(
                "Invalid config_name or category: {} {}",
                config_name, category
            );
            warn!("{}", err);
            return DeleteResponse::bad_request(err);
        };

        match Configuration::delete(&pool_arc, &config_name, &category, &username).await {
            Ok(_) => DeleteResponse::no_content(),
            Err(e) => {
                let err = format!("Failed to delete configuration: {}", e);
                warn!("{}", err);
                DeleteResponse::not_found(err)
            }
        }
    }

    /// Call `/api/v1/relations` with query params to fetch relations.
    #[oai(
        path = "/relations",
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
        _token: CustomSecurityScheme,
    ) -> GetRecordsResponse<Relation> {
        let pool_arc = pool.clone();
        let page = page.0;
        let page_size = page_size.0;
        let query_str = query_str.0;

        match PaginationQuery::new(page.clone(), page_size.clone(), query_str.clone()) {
            Ok(_) => {}
            Err(e) => {
                let err = format!("Failed to parse query string: {}", e);
                warn!("{}", err);
                return GetRecordsResponse::bad_request(err);
            }
        };

        let query = match query_str {
            Some(query_str) => match ComposeQuery::from_str(&query_str) {
                Ok(query) => query,
                Err(e) => {
                    let err = format!("Failed to parse query string: {}", e);
                    warn!("{}", err);
                    return GetRecordsResponse::bad_request(err);
                }
            },
            None => None,
        };

        // TODO: We need to add the model name to the query if we allow users to use different model.
        // TODO: We need to ensure the table exists before we use it.
        let table_name = get_kg_score_table_name(DEFAULT_MODEL_NAME);

        match RecordResponse::<Relation>::get_records(
            &pool_arc,
            table_name.as_str(),
            &query,
            page,
            page_size,
            Some("score ASC"),
            None,
        )
        .await
        {
            Ok(records) => GetRecordsResponse::ok(records),
            Err(e) => {
                let err = format!("Failed to fetch relations: {}", e);
                warn!("{}", err);
                return GetRecordsResponse::bad_request(err);
            }
        }
    }

    /// Call `/api/v1/relation-counts` with query params to fetch relation counts.
    #[oai(
        path = "/relation-counts",
        method = "get",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "fetchRelationCounts"
    )]
    async fn fetch_relation_counts(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        query_str: Query<Option<String>>,
        _token: CustomSecurityScheme,
    ) -> GetRelationCountResponse {
        let pool_arc = pool.clone();
        let query_str = query_str.0;

        let query = match query_str {
            Some(query_str) => match ComposeQuery::from_str(&query_str) {
                Ok(query) => query,
                Err(e) => {
                    let err = format!("Failed to parse query string: {}", e);
                    warn!("{}", err);
                    return GetRelationCountResponse::bad_request(err);
                }
            },
            None => None,
        };

        match RelationCount::get_records(&pool_arc, &query).await {
            Ok(entities) => GetRelationCountResponse::ok(entities),
            Err(e) => {
                let err = format!("Failed to fetch relations: {}", e);
                warn!("{}", err);
                return GetRelationCountResponse::bad_request(err);
            }
        }
    }

    /// Call `/api/v1/entity2d` with query params to fetch entity2d.
    #[oai(
        path = "/entity2d",
        method = "get",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "fetchEntity2D"
    )]
    async fn fetch_entity2d(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        page: Query<Option<u64>>,
        page_size: Query<Option<u64>>,
        query_str: Query<Option<String>>,
        _token: CustomSecurityScheme,
    ) -> GetRecordsResponse<Entity2D> {
        let pool_arc = pool.clone();
        let page = page.0;
        let page_size = page_size.0;
        let query_str = query_str.0;

        match PaginationQuery::new(page.clone(), page_size.clone(), query_str.clone()) {
            Ok(_) => {}
            Err(e) => {
                let err = format!("Failed to parse query string: {}", e);
                warn!("{}", err);
                return GetRecordsResponse::bad_request(err);
            }
        }

        let query = match query_str {
            Some(query_str) => match ComposeQuery::from_str(&query_str) {
                Ok(query) => query,
                Err(e) => {
                    let err = format!("Failed to parse query string: {}", e);
                    warn!("{}", err);
                    return GetRecordsResponse::bad_request(err);
                }
            },
            None => None,
        };

        // TODO: Could we compute the 2d embedding on the fly or by the biomedgps-cli tool?
        match RecordResponse::<Entity2D>::get_records(
            &pool_arc,
            "biomedgps_entity2d",
            &query,
            page,
            page_size,
            Some("embedding_id ASC"),
            None,
        )
        .await
        {
            Ok(entities) => GetRecordsResponse::ok(entities),
            Err(e) => {
                let err = format!("Failed to fetch entity2d: {}", e);
                warn!("{}", err);
                return GetRecordsResponse::bad_request(err);
            }
        }
    }

    /// Call `/api/v1/subgraphs` with query params to fetch subgraphs.
    #[oai(
        path = "/subgraphs",
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
        token: CustomSecurityScheme,
    ) -> GetRecordsResponse<Subgraph> {
        let pool_arc = pool.clone();
        let page = page.0;
        let page_size = page_size.0;
        let token = token.0;
        let query_str = query_str.0;

        match PaginationQuery::new(page.clone(), page_size.clone(), query_str.clone()) {
            Ok(_) => {}
            Err(e) => {
                let err = format!("Failed to parse query string: {}", e);
                warn!("{}", err);
                return GetRecordsResponse::bad_request(err);
            }
        }

        let query = match query_str {
            Some(query_str) => match ComposeQuery::from_str(&query_str) {
                Ok(query) => query,
                Err(e) => {
                    let err = format!("Failed to parse query string: {}", e);
                    warn!("{}", err);
                    return GetRecordsResponse::bad_request(err);
                }
            },
            None => None,
        };

        match RecordResponse::<Subgraph>::get_records(
            &pool_arc,
            "biomedgps_subgraph",
            &query,
            page,
            page_size,
            Some("created_time DESC"),
            Some(&token.username),
        )
        .await
        {
            Ok(entities) => GetRecordsResponse::ok(entities),
            Err(e) => {
                let err = format!("Failed to fetch subgraphs: {}", e);
                warn!("{}", err);
                return GetRecordsResponse::bad_request(err);
            }
        }
    }

    /// Call `/api/v1/subgraphs` with payload to create a subgraph.
    #[oai(
        path = "/subgraphs",
        method = "post",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "postSubgraph"
    )]
    async fn post_subgraph(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        payload: Json<Subgraph>,
        _token: CustomSecurityScheme,
    ) -> PostResponse<Subgraph> {
        let pool_arc = pool.clone();
        let mut payload = payload.0;
        let username = _token.0.username.clone();

        // When we enabled auth mode, we need to use the username from an access_token instead.
        if username != USERNAME_PLACEHOLDER.to_string() {
            payload.update_owner(username);
        }

        match payload.validate() {
            Ok(_) => {}
            Err(e) => {
                let err = format!("Failed to validate subgraph: {}", e);
                warn!("{}", err);
                return PostResponse::bad_request(err);
            }
        };

        match payload.insert(&pool_arc).await {
            Ok(kc) => PostResponse::created(kc),
            Err(e) => {
                let err = format!("Failed to insert curated knowledge: {}", e);
                warn!("{}", err);
                return PostResponse::bad_request(err);
            }
        }
    }

    /// Call `/api/v1/subgraphs/:id` with payload to update a subgraph.
    #[oai(
        path = "/subgraphs/:id",
        method = "put",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "putSubgraph"
    )]
    async fn put_subgraph(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        id: Path<String>,
        payload: Json<Subgraph>,
        _token: CustomSecurityScheme,
    ) -> PostResponse<Subgraph> {
        let pool_arc = pool.clone();
        let id = id.0;
        let mut payload = payload.0;
        let username = _token.0.username.clone();

        // When we enabled auth mode, we need to use the username from an access_token instead.
        if username != "user-placeholder".to_string() {
            payload.update_owner(username);
        }

        match SubgraphIdQuery::new(&id) {
            Ok(_) => {}
            Err(e) => {
                let err = format!("Failed to parse subgraph id: {}", e);
                warn!("{}", err);
                return PostResponse::bad_request(err);
            }
        }

        match payload.validate() {
            Ok(_) => {}
            Err(e) => {
                let err = format!("Failed to validate subgraph: {}", e);
                warn!("{}", err);
                return PostResponse::bad_request(err);
            }
        }

        match payload.update(&pool_arc, &id).await {
            Ok(kc) => PostResponse::created(kc),
            Err(e) => {
                let err = format!("Failed to update subgraph: {}", e);
                warn!("{}", err);
                return PostResponse::bad_request(err);
            }
        }
    }

    /// Call `/api/v1/subgraphs/:id` with payload to create subgraph.
    #[oai(
        path = "/subgraphs/:id",
        method = "delete",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "deleteSubgraph"
    )]
    async fn delete_subgraph(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        id: Path<String>,
        _token: CustomSecurityScheme,
    ) -> DeleteResponse {
        let pool_arc = pool.clone();
        let id = id.0;

        match SubgraphIdQuery::new(&id) {
            Ok(_) => {}
            Err(e) => {
                let err = format!("Failed to validate subgraph id: {}", e);
                warn!("{}", err);
                return DeleteResponse::bad_request(err);
            }
        }

        match Subgraph::delete(&pool_arc, &id).await {
            Ok(_) => DeleteResponse::NoContent,
            Err(e) => {
                let err = format!("Failed to delete a subgraph: {}", e);
                warn!("{}", err);
                DeleteResponse::not_found(err)
            }
        }
    }

    /// Call `/api/v1/nodes` with query params to fetch nodes.
    #[oai(
        path = "/nodes",
        method = "get",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "fetchNodes"
    )]
    async fn fetch_nodes(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        node_ids: Query<String>,
        _token: CustomSecurityScheme,
    ) -> GetGraphResponse {
        let pool_arc = pool.clone();
        let node_ids = node_ids.0;

        match NodeIdsQuery::new(&node_ids) {
            Ok(_) => {}
            Err(e) => {
                let err = format!("Failed to validate node ids: {}", e);
                warn!("{}", err);
                return GetGraphResponse::bad_request(err);
            }
        };

        let mut graph = Graph::new();

        if node_ids == "" {
            return GetGraphResponse::ok(graph);
        }

        let node_ids: Vec<&str> = node_ids.split(",").collect();
        match graph.fetch_nodes_by_ids(&pool_arc, &node_ids).await {
            Ok(graph) => GetGraphResponse::ok(graph.to_owned().get_graph(None).unwrap()),
            Err(e) => {
                let err = format!("Failed to fetch nodes: {}", e);
                warn!("{}", err);
                return GetGraphResponse::bad_request(err);
            }
        }
    }

    /// Call `/api/v1/auto-connect-nodes` with query params to fetch edges which connect the input nodes.
    #[oai(
        path = "/auto-connect-nodes",
        method = "get",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "fetchEdgesAutoConnectNodes"
    )]
    async fn fetch_edges_auto_connect_nodes(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        node_ids: Query<String>,
        _token: CustomSecurityScheme,
    ) -> GetGraphResponse {
        let pool_arc = pool.clone();
        let node_ids = node_ids.0;

        match NodeIdsQuery::new(&node_ids) {
            Ok(_) => {}
            Err(e) => {
                let err = format!("Failed to validate node ids: {}", e);
                warn!("{}", err);
                return GetGraphResponse::bad_request(err);
            }
        };

        let mut graph = Graph::new();

        if node_ids == "" {
            return GetGraphResponse::ok(graph);
        }

        let node_ids: Vec<&str> = node_ids.split(",").collect();
        // TODO: we need to get the model_table_prefix from the parameter, so users can get the score from a specific model.
        let model_table_prefix = Some(DEFAULT_MODEL_NAME);
        match graph
            .auto_connect_nodes(&pool_arc, &node_ids, model_table_prefix)
            .await
        {
            Ok(graph) => GetGraphResponse::ok(graph.to_owned().get_graph(None).unwrap()),
            Err(e) => {
                let err = format!("Failed to fetch nodes: {}", e);
                warn!("{}", err);
                return GetGraphResponse::bad_request(err);
            }
        }
    }

    /// Call `/api/v1/one-step-linked-nodes` with query params to fetch linked nodes with one step.
    #[oai(
        path = "/one-step-linked-nodes",
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
        _token: CustomSecurityScheme,
    ) -> GetGraphResponse {
        let pool_arc = pool.clone();
        let page = page.0;
        let page_size = page_size.0;
        let query_str = query_str.0;

        match PaginationQuery::new(page.clone(), page_size.clone(), query_str.clone()) {
            Ok(_) => {}
            Err(e) => {
                let err = format!("Failed to parse query string: {}", e);
                warn!("{}", err);
                return GetGraphResponse::bad_request(err);
            }
        };

        let query = match query_str {
            Some(query_str) => match ComposeQuery::from_str(&query_str) {
                Ok(query) => query,
                Err(e) => {
                    let err = format!("Failed to parse query string: {}", e);
                    warn!("{}", err);
                    return GetGraphResponse::bad_request(err);
                }
            },
            None => None,
        };

        let mut graph = Graph::new();
        // score DESC is the order_by clause for making the engine generate results with scores which computed by the model.
        match graph
            .fetch_linked_nodes(&pool_arc, &query, page, page_size, Some("score DESC"))
            .await
        {
            Ok(graph) => GetGraphResponse::ok(graph.to_owned().get_graph(None).unwrap()),
            Err(e) => {
                let err = format!("Failed to fetch linked nodes: {}", e);
                warn!("{}", err);
                return GetGraphResponse::bad_request(err);
            }
        }
    }

    /// Call `/api/v1/predicted-nodes` with query params to fetch predicted nodes.
    #[oai(
        path = "/predicted-nodes",
        method = "get",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "fetchPredictedNodes"
    )]
    async fn fetch_predicted_nodes(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        node_id: Query<String>,
        relation_type: Query<String>,
        query_str: Query<Option<String>>,
        topk: Query<Option<u64>>,
        model_name: Query<Option<String>>,
        _token: CustomSecurityScheme,
    ) -> GetGraphResponse {
        let pool_arc = pool.clone();
        let query_str = query_str.0;

        match PredictedNodeQuery::new(&node_id.0, &relation_type.0, &query_str, topk.0) {
            Ok(query) => query,
            Err(e) => {
                let err = format!("Failed to parse query string: {}", e);
                warn!("{}", err);
                return GetGraphResponse::bad_request(err);
            }
        };

        let topk = topk.0;

        let query = match query_str {
            Some(query_str) => match ComposeQuery::from_str(&query_str) {
                Ok(query) => query,
                Err(e) => {
                    let err = format!("Failed to parse query string: {}", e);
                    warn!("{}", err);
                    return GetGraphResponse::bad_request(err);
                }
            },
            None => None,
        };

        let mut graph = Graph::new();
        match graph
            .fetch_predicted_nodes(
                &pool_arc,
                &node_id,
                &relation_type,
                &query,
                topk,
                model_name.0,
            )
            .await
        {
            Ok(graph) => GetGraphResponse::ok(graph.to_owned().get_graph(None).unwrap()),
            Err(e) => {
                let err = format!("{}", e);
                warn!("{}", err);
                return GetGraphResponse::bad_request(err);
            }
        }
    }

    /// Call `/api/v1/shared-nodes` with query params to fetch shared nodes.
    #[oai(
        path = "/shared-nodes",
        method = "get",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "fetchSharedNodes"
    )]
    async fn fetch_shared_nodes(
        &self,
        pool: Data<&Arc<neo4rs::Graph>>,
        node_ids: Query<String>,
        target_node_types: Query<Option<String>>,
        start_node_id: Query<Option<String>>,
        topk: Query<Option<u64>>,
        nhops: Query<Option<usize>>,
        nums_shared_by: Query<Option<u64>>,
        _token: CustomSecurityScheme,
    ) -> GetGraphResponse {
        let pool_arc = pool.clone();
        let node_ids = node_ids.0;
        let target_node_types = target_node_types.0;
        let start_node_id = start_node_id.0;

        match NodeIdsQuery::new(&node_ids) {
            Ok(_) => {}
            Err(e) => {
                let err = format!("Failed to validate node ids: {}", e);
                warn!("{}", err);
                return GetGraphResponse::bad_request(err);
            }
        };

        let graph = Graph::new();

        if node_ids == "" {
            return GetGraphResponse::ok(graph);
        }

        let node_ids: Vec<&str> = node_ids.split(",").collect();

        let target_node_type_vec = match &target_node_types {
            Some(t) => {
                // TODO: We need to validate the target_node_types.
                let target_node_types: Vec<&str> = t.split(",").collect();
                if target_node_types.len() == 0 {
                    None
                } else {
                    Some(target_node_types)
                }
            }
            None => None,
        };

        let topk = match topk.0 {
            Some(topk) => topk,
            None => 10,
        };

        let nhops = match nhops.0 {
            Some(nhops) => nhops,
            None => 2,
        };

        let nums_shared_by = match nums_shared_by.0 {
            Some(nums_shared_by) => nums_shared_by,
            None => node_ids.len() as u64,
        };

        let (nodes, edges) = match query_shared_nodes(
            &pool_arc,
            &node_ids,
            target_node_type_vec,
            nhops as usize,
            topk as usize,
            nums_shared_by as usize,
            start_node_id.as_deref(),
        )
        .await
        {
            Ok((nodes, edges)) => (nodes, edges),
            Err(e) => {
                let err = format!("Failed to fetch paths: {}", e);
                warn!("{}", err);
                return GetGraphResponse::bad_request(err);
            }
        };

        if nodes.len() == 0 {
            let err = format!(
                "No shared nodes found between {:?} with {:?} hops and {:?} node types.",
                node_ids, nhops, target_node_types
            );
            warn!("{}", err);
            return GetGraphResponse::bad_request(err);
        };

        let nodes = nodes.iter().collect();
        let edges = edges.iter().collect();
        // TODO: How to get the topk paths based on the scores?
        let graph = Graph::from_data(nodes, edges);
        GetGraphResponse::ok(graph.to_owned().get_graph(None).unwrap())
    }

    /// Call `/api/v1/paths` with query params to fetch paths.
    #[oai(
        path = "/paths",
        method = "get",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "fetchPaths"
    )]
    async fn fetch_paths(
        &self,
        pool: Data<&Arc<neo4rs::Graph>>,
        start_node_id: Query<String>,
        end_node_id: Query<String>,
        nhops: Query<Option<usize>>,
        _token: CustomSecurityScheme,
    ) -> GetGraphResponse {
        let pool_arc = pool.clone();
        let start_node_id = start_node_id.0;
        let end_node_id = end_node_id.0;
        let nhops = match nhops.0 {
            Some(nhops) => nhops,
            None => {
                warn!("nhops is empty.");
                2
            }
        };

        let (nodes, edges) = match query_nhops(&pool_arc, &start_node_id, &end_node_id, nhops).await
        {
            Ok((nodes, edges)) => (nodes, edges),
            Err(e) => {
                let err = format!("Failed to fetch paths: {}", e);
                warn!("{}", err);
                return GetGraphResponse::bad_request(err);
            }
        };

        if nodes.len() == 0 {
            let err = format!(
                "No path found between {} and {} with {} hops.",
                start_node_id, end_node_id, nhops
            );
            warn!("{}", err);
            return GetGraphResponse::bad_request(err);
        };

        let nodes = nodes.iter().collect();
        let edges = edges.iter().collect();
        // TODO: How to get the topk paths based on the scores?
        let graph = Graph::from_data(nodes, edges);
        GetGraphResponse::ok(graph.to_owned().get_graph(None).unwrap())
    }

    /// Call `/api/v1/llm` with query params to get answer from LLM.
    #[oai(
        path = "/llm",
        method = "post",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "askLLM"
    )]
    async fn ask_llm(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        prompt_template_id: Query<String>,
        context: Json<Context>,
        _token: CustomSecurityScheme,
    ) -> PostResponse<LlmResponse> {
        let pool_arc = pool.clone();
        let prompt_template_id = prompt_template_id.0;
        let context = context.0;
        debug!("Prompt template id: {}", prompt_template_id);
        debug!("Context: {:?}", context);

        let openai_api_key = match std::env::var("OPENAI_API_KEY") {
            Ok(openai_api_key) => openai_api_key,
            Err(e) => {
                let err = format!("Failed to get OPENAI_API_KEY: {}", e);
                warn!("{}", err);
                return PostResponse::bad_request(err);
            }
        };

        let chatbot = ChatBot::new("GPT4", &openai_api_key);
        match context
            .answer(&chatbot, &prompt_template_id, Some(&pool_arc))
            .await
        {
            Ok(llm_response) => PostResponse::created(llm_response),
            Err(e) => {
                let err = format!("Failed to get answer from LLM: {}", e);
                warn!("{}", err);
                return PostResponse::bad_request(err);
            }
        }
    }

    /// Call `/api/v1/llm-prompts` with query params to get prompt templates.
    #[oai(
        path = "/llm-prompts",
        method = "get",
        tag = "ApiTags::KnowledgeGraph",
        operation_id = "fetchPrompts"
    )]
    async fn get_prompts(&self, _token: CustomSecurityScheme) -> GetPromptResponse {
        let prompt_lst = PROMPTS.lock().unwrap();
        let total = prompt_lst.len() as u64;
        let page = 1;
        let page_size = total;
        let records = prompt_lst
            .iter()
            .map(|p| {
                p.into_iter()
                    .map(|(k, v)| (String::from(*k), String::from(*v)))
                    .collect()
            })
            .collect();

        GetPromptResponse::ok(PromptList {
            total,
            page,
            page_size,
            records,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{init_logger, kv2urlstr, setup_test_db};
    use log::{debug, error, LevelFilter};
    use poem::middleware::{AddData, AddDataEndpoint};
    use poem::test::TestClient;
    use poem::{
        http::{StatusCode, Uri},
        Endpoint, EndpointExt, Request, Route,
    };
    use poem_openapi::OpenApiService;
    use sqlx::{Pool, Postgres};

    async fn init_app() -> AddDataEndpoint<Route, Arc<Pool<Postgres>>> {
        let _ = init_logger("biomedgps-test", LevelFilter::Debug);
        let pool = setup_test_db().await;

        let arc_pool = Arc::new(pool);
        let shared_rb = AddData::new(arc_pool.clone());
        let service = OpenApiService::new(BiomedgpsApi, "BioMedGPS", "v0.1.0");
        let app = Route::new().nest("/", service).with(shared_rb);
        app
    }

    #[tokio::test]
    async fn test_fetch_entities() {
        let app = init_app().await;
        let cli = TestClient::new(app);

        let resp = cli.get("/api/v1/entities").send().await;
        resp.assert_status_is_ok();

        let json = resp.json().await;
        let entity_records = json.value().deserialize::<RecordResponse<Entity>>();
        assert!(entity_records.records.len() > 0);
        let resp = cli.get("/api/v1/entities?page=1&page_size=10").send().await;
        resp.assert_status_is_ok();

        let json = resp.json().await;
        let entity_records = json.value().deserialize::<RecordResponse<Entity>>();
        assert!(entity_records.records.len() == 10);

        let query_json_str = r#"{"operator": "=", "field": "id", "value": "DOID:2022"}"#;
        let query_str = kv2urlstr("query_str", &query_json_str.to_string());
        debug!("Query string: {}", query_str);

        let resp = cli
            .get(format!(
                "/api/v1/entities?page=1&page_size=10&{}",
                query_str
            ))
            .send()
            .await;
        resp.assert_status_is_ok();

        let json = resp.json().await;
        let entity_records = json.value().deserialize::<RecordResponse<Entity>>();
        assert!(entity_records.records.len() == 1);

        let query_json_str = r#"{
            "operator": "and", "items": [
                {"operator": "=", "field": "id", "value": "DOID:2022"},
                {"operator": "=", "field": "label", "value": "Disease"}
            ]
        }"#;
        let query_str = kv2urlstr("query_str", &query_json_str.to_string());
        debug!("Query string: {}", query_str);

        let resp = cli
            .get(format!(
                "/api/v1/entities?page=1&page_size=10&{}",
                query_str
            ))
            .send()
            .await;
        resp.assert_status_is_ok();

        let json = resp.json().await;
        let entity_records = json.value().deserialize::<RecordResponse<Entity>>();
        assert!(entity_records.records.len() == 1);

        let query_json_str = r#"{
            "operator": "and", "items": [
                {"operator": "=", "field": "id", "value": "NOT-FOUND:2022"},
                {"operator": "=", "field": "label", "value": "NOT-FOUND"}
            ]
        }"#;
        let query_str = kv2urlstr("query_str", &query_json_str.to_string());
        debug!("Query string: {}", query_str);

        let resp = cli
            .get(format!(
                "/api/v1/entities?page=1&page_size=10&{}",
                query_str
            ))
            .send()
            .await;
        resp.assert_status_is_ok();

        let json = resp.json().await;
        let entity_records = json.value().deserialize::<RecordResponse<Entity>>();
        assert!(entity_records.records.len() == 0);
    }

    #[tokio::test]
    async fn test_fetch_predicted_nodes() {
        let app = init_app().await;
        let cli = TestClient::new(app);

        let resp = cli.get("/api/v1/similarity-nodes").send().await;
        resp.assert_status(StatusCode::BAD_REQUEST);

        let resp = cli
            .get("/api/v1/similarity-nodes?node_id=Chemical::MESH:C000601183")
            .send()
            .await;
        let json = resp.json().await;
        let nodes = json.value().object().get("nodes");
        nodes.assert_not_null();

        // TODO: Cannot deserialize Graph, because we cannot rename the field lineWidth to line_width when deserializing.
        // The poem-openapi crate does not support to rename a field when deserializing.
        //
        // let mut records = json.value().deserialize::<Graph>();
        // assert!(records.get_nodes().len() == 10);
    }
}
