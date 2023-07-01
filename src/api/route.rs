use super::model::{
    BackgroundFrequencyResponse, BackgroundFrequencyStat, Chromosome, CountResponse, CountStat,
    Dataset, DatasetPageResponse, SpeciesGenomePairs, GeneDataResponse
};
use crate::query::query_builder::{ComposeQuery, QueryItem};
use log::{debug, warn};
use poem::web::Data;
use poem_openapi::Object;
use poem_openapi::{param::Path, param::Query, payload::Json, ApiResponse, OpenApi, Tags};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Tags)]
enum ApiTags {
    Datasets,
    Dataset,
    BackgroundFrequencies,
    Counts,
    SpeciesGenomePairs,
    CountStat,
    BackgroundFrequencyStat,
    Chromosomes,
    Genes,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Object)]
struct ErrorMessage {
    msg: String,
}

#[derive(ApiResponse)]
enum GetChromosomesResponse {
    #[oai(status = 200)]
    Ok(Json<Vec<Chromosome>>),

    #[oai(status = 400)]
    BadRequest(Json<ErrorMessage>),

    #[oai(status = 404)]
    NotFound(Json<ErrorMessage>),
}

#[derive(ApiResponse)]
enum GetGenesResponse {
    #[oai(status = 200)]
    Ok(Json<GeneDataResponse>),

    #[oai(status = 400)]
    BadRequest(Json<ErrorMessage>),

    #[oai(status = 404)]
    NotFound(Json<ErrorMessage>),
}

#[derive(ApiResponse)]
enum GetDatasetsResponse {
    #[oai(status = 200)]
    Ok(Json<DatasetPageResponse>),

    #[oai(status = 400)]
    BadRequest(Json<ErrorMessage>),

    #[oai(status = 404)]
    NotFound(Json<ErrorMessage>),
}

#[derive(ApiResponse)]
enum GetDatasetResponse {
    #[oai(status = 200)]
    Ok(Json<Dataset>),

    #[oai(status = 400)]
    BadRequest(Json<ErrorMessage>),

    #[oai(status = 404)]
    NotFound(Json<ErrorMessage>),

    #[oai(status = 500)]
    InternalError(Json<ErrorMessage>),
}

#[derive(ApiResponse)]
enum GetBackgroundFrequenciesResponse {
    #[oai(status = 200)]
    Ok(Json<BackgroundFrequencyResponse>),

    #[oai(status = 400)]
    BadRequest(Json<ErrorMessage>),

    #[oai(status = 404)]
    NotFound(Json<ErrorMessage>),
}

#[derive(ApiResponse)]
enum GetCountsResponse {
    #[oai(status = 200)]
    Ok(Json<CountResponse>),

    #[oai(status = 400)]
    BadRequest(Json<ErrorMessage>),

    #[oai(status = 404)]
    NotFound(Json<ErrorMessage>),
}

#[derive(ApiResponse)]
enum GetSpeciesGenomePairsResponse {
    #[oai(status = 200)]
    Ok(Json<SpeciesGenomePairs>),

    #[oai(status = 400)]
    BadRequest(Json<ErrorMessage>),

    #[oai(status = 404)]
    NotFound(Json<ErrorMessage>),
}

#[derive(ApiResponse)]
enum GetCountStatResponse {
    #[oai(status = 200)]
    Ok(Json<Vec<CountStat>>),

    #[oai(status = 400)]
    BadRequest(Json<ErrorMessage>),

    #[oai(status = 404)]
    NotFound(Json<ErrorMessage>),
}

#[derive(ApiResponse)]
enum GetBackgroundFrequencyStatResponse {
    #[oai(status = 200)]
    Ok(Json<Vec<BackgroundFrequencyStat>>),

    #[oai(status = 400)]
    BadRequest(Json<ErrorMessage>),

    #[oai(status = 404)]
    NotFound(Json<ErrorMessage>),
}

pub struct RnmpdbApi;

#[OpenApi]
impl RnmpdbApi {
    /// Call `/api/v1/chromosomes` with query params to fetch chromosomes.
    #[oai(
        path = "/api/v1/chromosomes",
        method = "get",
        tag = "ApiTags::Chromosomes",
        operation_id = "fetchChromosomes"
    )]
    async fn fetch_chromosomes(&self, pool: Data<&Arc<sqlx::PgPool>>) -> GetChromosomesResponse {
        let pool_arc = pool.clone();

        match Chromosome::get_chromosomes(&pool_arc).await {
            Ok(chromosomes) => GetChromosomesResponse::Ok(Json(chromosomes)),
            Err(e) => {
                let err = format!("Failed to fetch chromosomes: {}", e);
                warn!("{}", err);
                return GetChromosomesResponse::BadRequest(Json(ErrorMessage { msg: err }));
            }
        }
    }

    /// Call `/api/v1/genes` with query params to fetch genes.
    #[oai(
        path = "/api/v1/genes",
        method = "get",
        tag = "ApiTags::Genes",
        operation_id = "fetchGenes"
    )]
    async fn fetch_genes(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        ref_genome: Query<String>,
        query_str: Query<Option<String>>,
        page: Query<Option<u64>>,
        page_size: Query<Option<u64>>,
    ) -> GetGenesResponse {
        let page = page.unwrap_or_else(|| 1);
        let page_size = page_size.unwrap_or_else(|| 10);
        let query_str = query_str.0;
        let pool_arc = pool.clone();
        let ref_genome = ref_genome.as_str();

        match GeneDataResponse::get_genes(&pool_arc, &ref_genome, query_str, page, page_size).await {
            Ok(genes) => GetGenesResponse::Ok(Json(genes)),
            Err(e) => {
                let err = format!("Failed to fetch genes: {}", e);
                warn!("{}", err);
                return GetGenesResponse::BadRequest(Json(ErrorMessage { msg: err }));
            }
        }
    }

    /// Call `/api/v1/datasets` with query params to fetch datasets.
    #[oai(
        path = "/api/v1/datasets",
        method = "get",
        tag = "ApiTags::Datasets",
        operation_id = "fetchDatasets"
    )]
    async fn fetch_datasets(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        page: Query<Option<u64>>,
        page_size: Query<Option<u64>>,
        query_str: Query<Option<String>>,
    ) -> GetDatasetsResponse {
        let pool_arc = pool.clone();
        let page = page.unwrap_or_else(|| 1);
        let page_size = page_size.unwrap_or_else(|| 10);

        let query_str = match query_str.0 {
            Some(query_str) => query_str,
            None => {
                warn!("Query string is empty.");
                "".to_string()
            }
        };

        let query = if query_str == "" {
            ComposeQuery::QueryItem(QueryItem::default())
        } else {
            debug!("Query string: {}", &query_str);
            // Parse query string as json
            match serde_json::from_str(&query_str) {
                Ok(query) => query,
                Err(e) => {
                    let err = format!("Failed to parse query string: {}", e);
                    warn!("{}", err);
                    return GetDatasetsResponse::BadRequest(Json(ErrorMessage { msg: err }));
                }
            }
        };

        match DatasetPageResponse::get_datasets(&pool_arc, &query, page, page_size).await {
            Ok(datasets) => GetDatasetsResponse::Ok(Json(datasets)),
            Err(e) => {
                let err = format!("Failed to fetch datasets: {}", e);
                warn!("{}", err);
                return GetDatasetsResponse::BadRequest(Json(ErrorMessage { msg: err }));
            }
        }
    }

    /// Call `/api/v1/datasets/:id` to fetch the dataset.
    #[oai(
        path = "/api/v1/datasets/:id",
        method = "get",
        tag = "ApiTags::Dataset",
        operation_id = "fetchDataset"
    )]
    async fn fetch_dataset(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        id: Path<u32>,
    ) -> GetDatasetResponse {
        let pool_arc = pool.clone();
        let id = id.0;

        match DatasetPageResponse::get_dataset(&pool_arc, id as i32).await {
            Ok(dataset) => GetDatasetResponse::Ok(Json(dataset)),
            Err(e) => {
                let err = format!("Failed to fetch datasets: {}", e);
                warn!("{}", err);
                return GetDatasetResponse::BadRequest(Json(ErrorMessage { msg: err }));
            }
        }
    }

    /// Call `/api/v1/species-genome-pairs` to fetch the species-genome pairs.
    #[oai(
        path = "/api/v1/species-genome-pairs",
        method = "get",
        tag = "ApiTags::SpeciesGenomePairs",
        operation_id = "fetchSpeciesGenomePairs"
    )]
    async fn fetch_species_genome_pairs(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
    ) -> GetSpeciesGenomePairsResponse {
        let pool_arc = pool.clone();

        match DatasetPageResponse::get_species_genomes(&pool_arc).await {
            Ok(pairs) => {
                let species_genome_pairs = SpeciesGenomePairs { data: pairs };
                return GetSpeciesGenomePairsResponse::Ok(Json(species_genome_pairs));
            }
            Err(e) => {
                let err = format!("Failed to fetch species-genome pairs: {}", e);
                warn!("{}", err);
                return GetSpeciesGenomePairsResponse::BadRequest(Json(ErrorMessage { msg: err }));
            }
        }
    }

    /// Call `/api/v1/background-frequencies` with query params to fetch background frequencies.
    #[oai(
        path = "/api/v1/background-frequencies",
        method = "get",
        tag = "ApiTags::BackgroundFrequencies",
        operation_id = "fetchBackgroundFrequencies"
    )]
    async fn fetch_background_frequencies(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        page: Query<Option<u64>>,
        page_size: Query<Option<u64>>,
        query_str: Query<Option<String>>,
    ) -> GetBackgroundFrequenciesResponse {
        let pool_arc = pool.clone();
        let page = page.unwrap_or_else(|| 1);
        let page_size = page_size.unwrap_or_else(|| 10);

        let query_str = match query_str.0 {
            Some(query_str) => query_str,
            None => {
                warn!("Query string is empty.");
                "".to_string()
            }
        };

        let query = if query_str == "" {
            ComposeQuery::QueryItem(QueryItem::default())
        } else {
            debug!("Query string: {}", &query_str);
            // Parse query string as json
            match serde_json::from_str(&query_str) {
                Ok(query) => query,
                Err(e) => {
                    let err = format!("Failed to parse query string: {}", e);
                    warn!("{}", err);
                    return GetBackgroundFrequenciesResponse::BadRequest(Json(ErrorMessage {
                        msg: err,
                    }));
                }
            }
        };

        match BackgroundFrequencyResponse::get_background_frequency(
            &pool_arc, &query, page, page_size,
        )
        .await
        {
            Ok(records) => GetBackgroundFrequenciesResponse::Ok(Json(records)),
            Err(e) => {
                let err = format!("Failed to fetch records: {}", e);
                warn!("{}", err);
                return GetBackgroundFrequenciesResponse::BadRequest(Json(ErrorMessage {
                    msg: err,
                }));
            }
        }
    }

    /// Call `/api/v1/counts` with query params to fetch counts.
    #[oai(
        path = "/api/v1/counts",
        method = "get",
        tag = "ApiTags::Counts",
        operation_id = "fetchCounts"
    )]
    async fn fetch_counts(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        page: Query<Option<u64>>,
        page_size: Query<Option<u64>>,
        query_str: Query<Option<String>>,
    ) -> GetCountsResponse {
        let pool_arc = pool.clone();
        let page = page.unwrap_or_else(|| 1);
        let page_size = page_size.unwrap_or_else(|| 10);

        let query_str = match query_str.0 {
            Some(query_str) => query_str,
            None => {
                warn!("Query string is empty.");
                "".to_string()
            }
        };

        let query = if query_str == "" {
            ComposeQuery::QueryItem(QueryItem::default())
        } else {
            debug!("Query string: {}", &query_str);
            // Parse query string as json
            match serde_json::from_str(&query_str) {
                Ok(query) => query,
                Err(e) => {
                    let err = format!("Failed to parse query string: {}", e);
                    warn!("{}", err);
                    return GetCountsResponse::BadRequest(Json(ErrorMessage { msg: err }));
                }
            }
        };

        match CountResponse::get_counts(&pool_arc, &query, page, page_size).await {
            Ok(records) => GetCountsResponse::Ok(Json(records)),
            Err(e) => {
                let err = format!("Failed to fetch records: {}", e);
                warn!("{}", err);
                return GetCountsResponse::BadRequest(Json(ErrorMessage { msg: err }));
            }
        }
    }

    /// Call `/api/v1/count-stat` with query params to fetch count-stat.
    #[oai(
        path = "/api/v1/count-stat",
        method = "get",
        tag = "ApiTags::CountStat",
        operation_id = "fetchCountStat"
    )]
    async fn fetch_count_stat(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        query_str: Query<Option<String>>,
    ) -> GetCountStatResponse {
        let pool_arc = pool.clone();

        let query_str = match query_str.0 {
            Some(query_str) => query_str,
            None => {
                warn!("Query string is empty.");
                "".to_string()
            }
        };

        let query = if query_str == "" {
            ComposeQuery::QueryItem(QueryItem::default())
        } else {
            debug!("Query string: {}", &query_str);
            // Parse query string as json
            match serde_json::from_str(&query_str) {
                Ok(query) => query,
                Err(e) => {
                    let err = format!("Failed to parse query string: {}", e);
                    warn!("{}", err);
                    return GetCountStatResponse::BadRequest(Json(ErrorMessage { msg: err }));
                }
            }
        };

        match CountStat::get_count_stat(&pool_arc, &query, vec![]).await {
            Ok(records) => GetCountStatResponse::Ok(Json(records)),
            Err(e) => {
                let err = format!("Failed to fetch records: {}", e);
                warn!("{}", err);
                return GetCountStatResponse::BadRequest(Json(ErrorMessage { msg: err }));
            }
        }
    }

    /// Call `/api/v1/background-freq-stat` with query params to fetch background-freq-stat.
    #[oai(
        path = "/api/v1/background-freq-stat",
        method = "get",
        tag = "ApiTags::BackgroundFrequencyStat",
        operation_id = "fetchBackgroundFrequencyStat"
    )]
    async fn fetch_background_freq_stat(
        &self,
        pool: Data<&Arc<sqlx::PgPool>>,
        query_str: Query<Option<String>>,
    ) -> GetBackgroundFrequencyStatResponse {
        let pool_arc = pool.clone();

        let query_str = match query_str.0 {
            Some(query_str) => query_str,
            None => {
                warn!("Query string is empty.");
                "".to_string()
            }
        };

        let query = if query_str == "" {
            ComposeQuery::QueryItem(QueryItem::default())
        } else {
            debug!("Query string: {}", &query_str);
            // Parse query string as json
            match serde_json::from_str(&query_str) {
                Ok(query) => query,
                Err(e) => {
                    let err = format!("Failed to parse query string: {}", e);
                    warn!("{}", err);
                    return GetBackgroundFrequencyStatResponse::BadRequest(Json(ErrorMessage {
                        msg: err,
                    }));
                }
            }
        };

        match BackgroundFrequencyStat::get_background_freq_stat(&pool_arc, &query, vec![]).await {
            Ok(records) => GetBackgroundFrequencyStatResponse::Ok(Json(records)),
            Err(e) => {
                let err = format!("Failed to fetch records: {}", e);
                warn!("{}", err);
                return GetBackgroundFrequencyStatResponse::BadRequest(Json(ErrorMessage {
                    msg: err,
                }));
            }
        }
    }
}
