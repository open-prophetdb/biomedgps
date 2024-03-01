//! This module defines the routes of the API.

use crate::api::auth::{CustomSecurityScheme, USERNAME_PLACEHOLDER};
use crate::api::schema::{
    ApiTags, DeleteResponse, GetEntityColorMapResponse, GetGraphResponse, GetRecordsResponse,
    GetRelationCountResponse, GetStatisticsResponse, GetWholeTableResponse, NodeIdsQuery,
    Pagination, PaginationQuery, PostResponse, PredictedNodeQuery, SubgraphIdQuery,
};
use crate::model::core::{
    Entity, Entity2D, EntityMetadata, KnowledgeCuration, RecordResponse, Relation, RelationCount,
    RelationMetadata, Statistics, Subgraph,
};
use crate::model::graph::Graph;
use crate::model::init_db::get_kg_score_table_name;
use crate::model::kge::DEFAULT_MODEL_NAME;
use crate::model::llm::{ChatBot, Context, LlmResponse};
use crate::model::util::match_color;
use crate::query_builder::cypher_builder::{query_nhops, query_shared_nodes};
use crate::query_builder::sql_builder::{get_all_field_pairs, make_order_clause_by_pairs};
use log::{debug, info, warn};
use poem::web::Data;
use poem_openapi::{param::Path, param::Query, payload::Json, OpenApi};
use std::sync::Arc;
use validator::Validate;

pub struct BiomedgpsApi;

#[OpenApi(prefix_path = "/api/v1")]
impl BiomedgpsApi {
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
    ) -> GetStatisticsResponse {
        info!("Username: {}", _token.0.username);
        let pool_arc = pool.clone();

        let entity_metadata = match EntityMetadata::get_entity_metadata(&pool_arc).await {
            Ok(entity_metadata) => entity_metadata,
            Err(e) => {
                let err = format!("Failed to fetch entity metadata: {}", e);
                warn!("{}", err);
                return GetStatisticsResponse::bad_request(err);
            }
        };

        let relation_metadata = match RelationMetadata::get_relation_metadata(&pool_arc).await {
            Ok(relation_metadata) => relation_metadata,
            Err(e) => {
                let err = format!("Failed to fetch relation metadata: {}", e);
                warn!("{}", err);
                return GetStatisticsResponse::bad_request(err);
            }
        };

        let statistics = Statistics::new(entity_metadata, relation_metadata);

        GetStatisticsResponse::ok(statistics)
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
                    return GetRecordsResponse::bad_request(err);
                }
            }
        };

        let order_by_clause = match query.clone() {
            Some(q) => {
                let pairs = get_all_field_pairs(&q);
                if pairs.len() == 0 {
                    "id ASC".to_string()
                } else {
                    // More fields will cause bad performance
                    make_order_clause_by_pairs(pairs, 2)
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
        curator: Query<String>,
        project_id: Query<Option<String>>,
        organization_id: Query<Option<String>>,
        page: Query<Option<u64>>,
        page_size: Query<Option<u64>>,
        strict_mode: Query<bool>,
        _token: CustomSecurityScheme,
    ) -> GetGraphResponse {
        let pool_arc = pool.clone();
        let curator = curator.0;

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
        curator: Query<String>,
        project_id: Query<Option<String>>,
        organization_id: Query<Option<String>>,
        page: Query<Option<u64>>,
        page_size: Query<Option<u64>>,
        // We need to confirm the token is valid and contains all projects and organizations which the user has access to.
        _token: CustomSecurityScheme,
    ) -> GetRecordsResponse<KnowledgeCuration> {
        let pool_arc = pool.clone();
        let curator = curator.0;

        if curator != _token.0.username {
            let err = format!(
                "You cannot query curated knowledges from other users. You are {} and you are querying {}'s curated knowledges.",
                _token.0.username, curator
            );
            warn!("{}", err);
            return GetRecordsResponse::bad_request(err);
        }

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

        match KnowledgeCuration::get_records_by_owner(
            &pool_arc,
            &curator,
            project_id,
            organization_id,
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

        match PaginationQuery::new(page.clone(), page_size.clone(), query_str.0.clone()) {
            Ok(_) => {}
            Err(e) => {
                let err = format!("Failed to parse query string: {}", e);
                warn!("{}", err);
                return GetRecordsResponse::bad_request(err);
            }
        }

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
                    return GetRecordsResponse::bad_request(err);
                }
            }
        };

        match RecordResponse::<KnowledgeCuration>::get_records(
            &pool_arc,
            "biomedgps_knowledge_curation",
            &query,
            page,
            page_size,
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
        let payload = payload.0;

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
        let payload = payload.0;
        let id = id.0;

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

        match KnowledgeCuration::delete(&pool_arc, id).await {
            Ok(_) => DeleteResponse::no_content(),
            Err(e) => {
                let err = format!("Failed to delete curated knowledge: {}", e);
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

        match PaginationQuery::new(page.clone(), page_size.clone(), query_str.0.clone()) {
            Ok(_) => {}
            Err(e) => {
                let err = format!("Failed to parse query string: {}", e);
                warn!("{}", err);
                return GetRecordsResponse::bad_request(err);
            }
        };

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
                    return GetRecordsResponse::bad_request(err);
                }
            }
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
        )
        .await
        {
            Ok(entities) => GetRecordsResponse::ok(entities),
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
                    return GetRelationCountResponse::bad_request(err);
                }
            }
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
        operation_id = "fetchEntity2d"
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

        match PaginationQuery::new(page.clone(), page_size.clone(), query_str.0.clone()) {
            Ok(_) => {}
            Err(e) => {
                let err = format!("Failed to parse query string: {}", e);
                warn!("{}", err);
                return GetRecordsResponse::bad_request(err);
            }
        }

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
                    return GetRecordsResponse::bad_request(err);
                }
            }
        };

        // TODO: Could we compute the 2d embedding on the fly or by the biomedgps-cli tool?
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
        _token: CustomSecurityScheme,
    ) -> GetRecordsResponse<Subgraph> {
        let pool_arc = pool.clone();
        let page = page.0;
        let page_size = page_size.0;

        match PaginationQuery::new(page.clone(), page_size.clone(), query_str.0.clone()) {
            Ok(_) => {}
            Err(e) => {
                let err = format!("Failed to parse query string: {}", e);
                warn!("{}", err);
                return GetRecordsResponse::bad_request(err);
            }
        }

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
                    return GetRecordsResponse::bad_request(err);
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

        match PaginationQuery::new(page.clone(), page_size.clone(), query_str.0.clone()) {
            Ok(_) => {}
            Err(e) => {
                let err = format!("Failed to parse query string: {}", e);
                warn!("{}", err);
                return GetGraphResponse::bad_request(err);
            }
        };

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
                    return GetGraphResponse::bad_request(err);
                }
            }
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

        match PredictedNodeQuery::new(&node_id.0, &relation_type.0, &query_str.0, topk.0) {
            Ok(query) => query,
            Err(e) => {
                let err = format!("Failed to parse query string: {}", e);
                warn!("{}", err);
                return GetGraphResponse::bad_request(err);
            }
        };

        let query_str = match query_str.0 {
            Some(query_str) => query_str,
            None => {
                warn!("Query string is empty.");
                "".to_string()
            }
        };

        let topk = topk.0;

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
                    return GetGraphResponse::bad_request(err);
                }
            }
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
        topk: Query<Option<u64>>,
        nhops: Query<Option<usize>>,
        nums_shared_by: Query<Option<u64>>,
        _token: CustomSecurityScheme,
    ) -> GetGraphResponse {
        let pool_arc = pool.clone();
        let node_ids = node_ids.0;
        let target_node_types = target_node_types.0;

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
