use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use chrono::{DateTime, Utc, Duration};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object)]
pub struct TopCurator {
    pub curator: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object)]
pub struct TopRelationType {
    pub relation_type: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object)]
pub struct TopEntityType {
    pub entity_type: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object, sqlx::FromRow, Validate)]
pub struct MonthlyTrend {
    pub month: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object)]
pub struct CuratorActivity {
    pub curator: String,
    pub knowledges: i64,
    pub entities: i64,
    pub sentences: i64,
    pub publications: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object)]
pub struct MonthlyCuratorTrend {
    pub month: String,
    pub total: i64,
    pub per_curator: HashMap<String, i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object)]
pub struct CurationStatistics {
    pub total_knowledges: i64,
    pub total_entities: i64,
    pub total_key_sentences: i64,
    pub total_curators: i64,
    pub total_publications: i64,
    pub recent_activity_30_days: i64,
    pub recent_activity_60_days: i64,
    pub recent_activity_90_days: i64,
    pub recent_activity_180_days: i64,
    pub top_curators: Vec<TopCurator>,
    pub top_relation_types: Vec<TopRelationType>,
    pub top_entity_types: Vec<TopEntityType>,
    pub monthly_trend: Vec<MonthlyTrend>,
    pub curator_activity: Vec<CuratorActivity>,
    pub monthly_curator_trend: Vec<MonthlyCuratorTrend>,
}

// Cache structure to store statistics with expiration time
#[derive(Debug, Clone)]
struct CachedStatistics {
    data: CurationStatistics,
    expires_at: DateTime<Utc>,
}

// Global cache using once_cell
static STATISTICS_CACHE: Lazy<Mutex<Option<CachedStatistics>>> = Lazy::new(|| Mutex::new(None));

impl CurationStatistics {
    pub async fn fetch_statistics(pool: &sqlx::PgPool) -> Result<Self, anyhow::Error> {
        // Check cache first
        {
            let cache = STATISTICS_CACHE.lock().unwrap();
            if let Some(cached) = &*cache {
                if Utc::now() < cached.expires_at {
                    return Ok(cached.data.clone());
                }
            }
        }

        // Cache miss or expired, fetch from database
        let stats = Self::fetch_statistics_from_db(pool).await?;

        // Update cache with 24-hour expiration
        {
            let mut cache = STATISTICS_CACHE.lock().unwrap();
            *cache = Some(CachedStatistics {
                data: stats.clone(),
                expires_at: Utc::now() + Duration::hours(24),
            });
        }

        Ok(stats)
    }

    async fn fetch_statistics_from_db(pool: &sqlx::PgPool) -> Result<Self, anyhow::Error> {
        // Fetch basic counts
        let total_knowledges: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM biomedgps_knowledge_curation"
        )
        .fetch_one(pool)
        .await?;

        let total_entities: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM biomedgps_entity_curation"
        )
        .fetch_one(pool)
        .await?;

        let total_key_sentences: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM biomedgps_key_sentence_curation"
        )
        .fetch_one(pool)
        .await?;

        let total_curators: i64 = sqlx::query_scalar(
            "SELECT COUNT(DISTINCT curator) FROM (
                SELECT curator FROM biomedgps_knowledge_curation
                UNION
                SELECT curator FROM biomedgps_entity_curation
                UNION
                SELECT curator FROM biomedgps_key_sentence_curation
            ) AS all_curators"
        )
        .fetch_one(pool)
        .await?;

        let total_publications: i64 = sqlx::query_scalar(
            "SELECT COUNT(DISTINCT fingerprint) FROM biomedgps_knowledge_curation"
        )
        .fetch_one(pool)
        .await?;

        // Fetch recent activity counts
        let recent_activity_30_days: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM (
                SELECT created_at FROM biomedgps_knowledge_curation WHERE created_at >= NOW() - INTERVAL '30 days'
                UNION ALL
                SELECT created_at FROM biomedgps_entity_curation WHERE created_at >= NOW() - INTERVAL '30 days'
                UNION ALL
                SELECT created_at FROM biomedgps_key_sentence_curation WHERE created_at >= NOW() - INTERVAL '30 days'
            ) AS recent_30"
        )
        .fetch_one(pool)
        .await?;

        let recent_activity_60_days: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM (
                SELECT created_at FROM biomedgps_knowledge_curation WHERE created_at >= NOW() - INTERVAL '60 days'
                UNION ALL
                SELECT created_at FROM biomedgps_entity_curation WHERE created_at >= NOW() - INTERVAL '60 days'
                UNION ALL
                SELECT created_at FROM biomedgps_key_sentence_curation WHERE created_at >= NOW() - INTERVAL '60 days'
            ) AS recent_60"
        )
        .fetch_one(pool)
        .await?;

        let recent_activity_90_days: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM (
                SELECT created_at FROM biomedgps_knowledge_curation WHERE created_at >= NOW() - INTERVAL '90 days'
                UNION ALL
                SELECT created_at FROM biomedgps_entity_curation WHERE created_at >= NOW() - INTERVAL '90 days'
                UNION ALL
                SELECT created_at FROM biomedgps_key_sentence_curation WHERE created_at >= NOW() - INTERVAL '90 days'
            ) AS recent_90"
        )
        .fetch_one(pool)
        .await?;

        let recent_activity_180_days: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM (
                SELECT created_at FROM biomedgps_knowledge_curation WHERE created_at >= NOW() - INTERVAL '180 days'
                UNION ALL
                SELECT created_at FROM biomedgps_entity_curation WHERE created_at >= NOW() - INTERVAL '180 days'
                UNION ALL
                SELECT created_at FROM biomedgps_key_sentence_curation WHERE created_at >= NOW() - INTERVAL '180 days'
            ) AS recent_180"
        )
            .fetch_one(pool)
            .await?;

        // Fetch top curators
        let top_curators_rows: Vec<(String, i64)> = sqlx::query_as(
            "SELECT curator, COUNT(*)::bigint as count FROM (
                SELECT curator FROM biomedgps_knowledge_curation
                UNION ALL
                SELECT curator FROM biomedgps_entity_curation
                UNION ALL
                SELECT curator FROM biomedgps_key_sentence_curation
            ) AS all_curations
            GROUP BY curator
            ORDER BY count DESC
            LIMIT 10"
        )
        .fetch_all(pool)
        .await?;

        let top_curators: Vec<TopCurator> = top_curators_rows
            .into_iter()
            .map(|(curator, count)| TopCurator { curator, count })
            .collect();

        // Fetch top relation types from knowledge curation
        let top_relation_types_rows: Vec<(String, i64)> = sqlx::query_as(
            "SELECT relation_type, COUNT(*)::bigint as count 
             FROM biomedgps_knowledge_curation 
             GROUP BY relation_type 
             ORDER BY count DESC 
             LIMIT 10"
        )
        .fetch_all(pool)
        .await?;

        let top_relation_types: Vec<TopRelationType> = top_relation_types_rows
            .into_iter()
            .map(|(relation_type, count)| TopRelationType { relation_type, count })
            .collect();

        // Fetch top entity types from entity curation
        let top_entity_types_rows: Vec<(String, i64)> = sqlx::query_as(
            "SELECT entity_type, COUNT(*)::bigint as count FROM (
                SELECT source_type as entity_type FROM biomedgps_knowledge_curation
                UNION ALL
                SELECT target_type as entity_type FROM biomedgps_knowledge_curation
                UNION ALL
                SELECT entity_type FROM biomedgps_entity_curation
            ) AS all_entity_types
            GROUP BY entity_type
            ORDER BY count DESC
            LIMIT 10"
        )
        .fetch_all(pool)
        .await?;

        let top_entity_types: Vec<TopEntityType> = top_entity_types_rows
            .into_iter()
            .map(|(entity_type, count)| TopEntityType { entity_type, count })
            .collect();

        // Fetch monthly trend (last 12 months)
        let monthly_trend: Vec<MonthlyTrend> = sqlx::query_as(
            "SELECT 
                TO_CHAR(month_date, 'YYYY-MM') as month,
                COALESCE(monthly_counts.count, 0)::bigint as count
             FROM generate_series(
                date_trunc('month', NOW() - INTERVAL '11 months'),
                date_trunc('month', NOW()),
                '1 month'::interval
             ) AS month_date
             LEFT JOIN (
                SELECT 
                    date_trunc('month', all_curations.created_at) as month,
                    COUNT(*)::bigint as count
                FROM (
                    SELECT created_at FROM biomedgps_knowledge_curation
                    UNION ALL
                    SELECT created_at FROM biomedgps_entity_curation
                    UNION ALL
                    SELECT created_at FROM biomedgps_key_sentence_curation
                ) AS all_curations
                WHERE all_curations.created_at >= date_trunc('month', NOW() - INTERVAL '11 months')
                GROUP BY date_trunc('month', all_curations.created_at)
             ) AS monthly_counts ON month_date = monthly_counts.month
             ORDER BY month_date"
        )
        .fetch_all(pool)
        .await?;

        // Fetch curator activity
        let curator_activity_rows: Vec<(String, i64, i64, i64, i64)> = sqlx::query_as(
            "SELECT 
                curators.curator,
                COALESCE(knowledge_stats.knowledge_count, 0)::bigint as knowledges,
                COALESCE(entity_stats.entity_count, 0)::bigint as entities,
                COALESCE(sentence_stats.sentence_count, 0)::bigint as sentences,
                COALESCE(publication_stats.publication_count, 0)::bigint as publications
             FROM (
                SELECT DISTINCT curator FROM (
                    SELECT curator FROM biomedgps_knowledge_curation
                    UNION
                    SELECT curator FROM biomedgps_entity_curation
                    UNION
                    SELECT curator FROM biomedgps_key_sentence_curation
                ) AS all_curators_list
             ) AS curators
             LEFT JOIN (
                SELECT curator, COUNT(*)::bigint as knowledge_count
                FROM biomedgps_knowledge_curation
                GROUP BY curator
             ) AS knowledge_stats ON curators.curator = knowledge_stats.curator
             LEFT JOIN (
                SELECT curator, COUNT(*)::bigint as entity_count
                FROM biomedgps_entity_curation
                GROUP BY curator
             ) AS entity_stats ON curators.curator = entity_stats.curator
             LEFT JOIN (
                SELECT curator, COUNT(*)::bigint as sentence_count
                FROM biomedgps_key_sentence_curation
                GROUP BY curator
             ) AS sentence_stats ON curators.curator = sentence_stats.curator
             LEFT JOIN (
                SELECT curator, COUNT(DISTINCT fingerprint)::bigint as publication_count
                FROM biomedgps_knowledge_curation
                GROUP BY curator
             ) AS publication_stats ON curators.curator = publication_stats.curator
             ORDER BY (COALESCE(knowledge_stats.knowledge_count, 0) + COALESCE(entity_stats.entity_count, 0) + COALESCE(sentence_stats.sentence_count, 0)) DESC"
        )
        .fetch_all(pool)
        .await?;

        let curator_activity: Vec<CuratorActivity> = curator_activity_rows
            .into_iter()
            .map(|(curator, knowledges, entities, sentences, publications)| CuratorActivity {
                curator,
                knowledges,
                entities,
                sentences,
                publications
            })
            .collect();

        // Fetch monthly curator trend (last 12 months)
        let monthly_curator_trend_rows: Vec<(String, i64, String, i64)> = sqlx::query_as(
            "SELECT 
                TO_CHAR(month_date, 'YYYY-MM') as month,
                COALESCE(monthly_curator_counts.total_count, 0)::bigint as total,
                COALESCE(monthly_curator_counts.curator, '') as curator,
                COALESCE(monthly_curator_counts.curator_count, 0)::bigint as curator_count
             FROM generate_series(
                date_trunc('month', NOW() - INTERVAL '11 months'),
                date_trunc('month', NOW()),
                '1 month'::interval
             ) AS month_date
             LEFT JOIN (
                SELECT 
                    date_trunc('month', all_curations.created_at) as month,
                    all_curations.curator,
                    COUNT(*)::bigint as curator_count,
                    SUM(COUNT(*)) OVER (PARTITION BY date_trunc('month', all_curations.created_at))::bigint as total_count
                FROM (
                    SELECT created_at, curator FROM biomedgps_knowledge_curation
                    UNION ALL
                    SELECT created_at, curator FROM biomedgps_entity_curation
                    UNION ALL
                    SELECT created_at, curator FROM biomedgps_key_sentence_curation
                ) AS all_curations
                WHERE all_curations.created_at >= date_trunc('month', NOW() - INTERVAL '11 months')
                GROUP BY date_trunc('month', all_curations.created_at), all_curations.curator
             ) AS monthly_curator_counts ON month_date = monthly_curator_counts.month
             ORDER BY month_date, monthly_curator_counts.curator"
        )
        .fetch_all(pool)
        .await?;

        // Group monthly curator trend data
        let mut monthly_curator_trend: Vec<MonthlyCuratorTrend> = Vec::new();
        let mut current_month = String::new();
        let mut current_total = 0i64;
        let mut current_per_curator: HashMap<String, i64> = HashMap::new();

        for (month, total, curator, curator_count) in monthly_curator_trend_rows {
            if month != current_month {
                if !current_month.is_empty() {
                    monthly_curator_trend.push(MonthlyCuratorTrend {
                        month: current_month.clone(),
                        total: current_total,
                        per_curator: current_per_curator.clone(),
                    });
                }
                current_month = month;
                current_total = total;
                current_per_curator = HashMap::new();
            }
            
            if !curator.is_empty() {
                current_per_curator.insert(curator, curator_count);
            }
        }

        // Don't forget the last month
        if !current_month.is_empty() {
            monthly_curator_trend.push(MonthlyCuratorTrend {
                month: current_month,
                total: current_total,
                per_curator: current_per_curator,
            });
        }

        Ok(CurationStatistics {
            total_knowledges,
            total_entities,
            total_key_sentences,
            total_curators,
            total_publications,
            recent_activity_30_days,
            recent_activity_60_days,
            recent_activity_90_days,
            recent_activity_180_days,
            top_curators,
            top_relation_types,
            top_entity_types,
            monthly_trend,
            curator_activity,
            monthly_curator_trend,
        })
    }

    /// Force refresh the cache by clearing it
    pub fn clear_cache() {
        let mut cache = STATISTICS_CACHE.lock().unwrap();
        *cache = None;
    }

    /// Check if cache exists and is valid
    pub fn is_cache_valid() -> bool {
        let cache = STATISTICS_CACHE.lock().unwrap();
        if let Some(cached) = &*cache {
            Utc::now() < cached.expires_at
        } else {
            false
        }
    }
}