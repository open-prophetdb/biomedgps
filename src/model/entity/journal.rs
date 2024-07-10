use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use validator::Validate;
use poem_openapi::Object;

use crate::model::core::CheckData;

pub const DEFAULT_MIN_LENGTH: u64 = 1;
pub const NAME_MAX_LENGTH: u64 = 255;
pub const ISSN_MAX_LENGTH: u64 = 32;
pub const CATEGORY_MAX_LENGTH: u64 = 32;
pub const JCR_QUARTILE_MAX_LENGTH: u64 = 8;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object, sqlx::FromRow, Validate)]
pub struct JournalMetadata {
    #[validate(length(
        max = "NAME_MAX_LENGTH",
        min = "DEFAULT_MIN_LENGTH",
        message = "The length of id should be between 1 and 64."
    ))]
    pub journal_name: String,
    #[validate(length(
        max = "NAME_MAX_LENGTH",
        min = "DEFAULT_MIN_LENGTH",
        message = "The length of id should be between 1 and 64."
    ))]
    pub abbr_name: String,
    #[validate(length(
        max = "ISSN_MAX_LENGTH",
        min = "DEFAULT_MIN_LENGTH",
        message = "The length of id should be between 1 and 64."
    ))]
    pub issn: String,
    #[validate(length(
        max = "ISSN_MAX_LENGTH",
        min = "DEFAULT_MIN_LENGTH",
        message = "The length of id should be between 1 and 64."
    ))]
    pub eissn: String,
    pub impact_factor: Option<f64>,
    pub impact_factor_5_year: Option<f64>,
    #[validate(length(
        max = "CATEGORY_MAX_LENGTH",
        min = "DEFAULT_MIN_LENGTH",
        message = "The length of id should be between 1 and 64."
    ))]
    pub category: String,
    #[validate(length(
        max = "JCR_QUARTILE_MAX_LENGTH",
        min = "DEFAULT_MIN_LENGTH",
        message = "The length of id should be between 1 and 64."
    ))]
    pub jcr_quartile: String,
    pub rank: Option<i32>,
    pub total_num_of_journals: Option<i32>,
}

impl CheckData for JournalMetadata {
    fn check_csv_is_valid(filepath: &PathBuf) -> Vec<Box<dyn Error>> {
        Self::check_csv_is_valid_default::<JournalMetadata>(filepath)
    }

    fn unique_fields() -> Vec<String> {
        vec!["name".to_string()]
    }

    fn fields() -> Vec<String> {
        vec![
            "journal_name".to_string(),
            "abbr_name".to_string(),
            "issn".to_string(),
            "eissn".to_string(),
            "impact_factor".to_string(),
            "impact_factor_5_year".to_string(),
            "category".to_string(),
            "jcr_quartile".to_string(),
            "rank".to_string(),
            "total_num_of_journals".to_string(),
        ]
    }
}

impl JournalMetadata {
    /// Get the average impact factor of the journals
    /// 
    /// # Arguments
    /// * `pool` - The database connection pool
    /// * `journal_names` - The list of journal names
    /// * `method` - The method to calculate the average impact factor, either "mean" or "median"
    /// 
    /// # Returns
    /// The average impact factor
    /// 
    /// # Errors
    /// Return an error if the method is invalid or the database query fails
    pub async fn average_impact_factor(
        pool: &sqlx::PgPool,
        journal_names: Vec<String>,
        method: Option<&str>,
    ) -> Result<f64, Box<dyn Error>> {
        let average_sql_str = if method.is_none() {
            "AVG(impact_factor)"
        } else if method == Some("mean") {
            "AVG(impact_factor)"
        } else if method == Some("median") {
            "PERCENTILE_CONT(0.5) WITHIN GROUP (ORDER BY impact_factor)"
        } else {
            return Err("Invalid method".into());
        };

        let sql_str = format!(
            "SELECT {} FROM biomedgps_journal_metadata WHERE {}",
            average_sql_str,
            journal_names
                .iter()
                .enumerate()
                .map(|(i, _)| format!("LOWER(journal_name) ILIKE LOWER(${})", i + 1))
                .collect::<Vec<String>>()
                .join(" OR ")
        );

        let mut query = sqlx::query_as::<_, (f64,)>(sql_str.as_str());
        for (i, name) in journal_names.iter().enumerate() {
            query = query.bind(name);
        }

        let row: (f64,) = query.fetch_one(pool).await?;
        Ok(row.0)
    }

    pub async fn sync2db(
        pool: &sqlx::PgPool,
        filepath: &PathBuf,
        clean_table: bool,
    ) -> Result<(), Box<dyn Error>> {
        if clean_table {
            let _ = sqlx::query("TRUNCATE TABLE biomedgps_journal_metadata")
                .execute(pool)
                .await?;
        };
        let file = File::open(filepath)?;
        let reader = BufReader::new(file);
        let journals: Vec<JournalMetadata> = serde_json::from_reader(reader)?;

        let mut tx = pool.begin().await?;
        let fields_str = JournalMetadata::fields().join(", ");
        let placeholders_str = JournalMetadata::fields()
            .iter()
            .enumerate()
            .map(|(i, _)| format!("${}", i + 1))
            .collect::<Vec<String>>()
            .join(", ");

        let sql = format!(
            "INSERT INTO biomedgps_journal_metadata ({}) VALUES ({})",
            fields_str, placeholders_str
        );

        for journal in journals {
            let result = sqlx::query(&sql)
                .bind(&journal.journal_name)
                .bind(&journal.abbr_name)
                .bind(&journal.issn)
                .bind(&journal.eissn)
                .bind(&journal.impact_factor)
                .bind(&journal.impact_factor_5_year)
                .bind(&journal.category)
                .bind(&journal.jcr_quartile)
                .bind(&journal.rank)
                .bind(&journal.total_num_of_journals)
                .execute(&mut tx)
                .await;

            if let Err(e) = result {
                tx.rollback().await?;
                return Err(Box::new(e));
            }
        }

        tx.commit().await?;

        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::{init_logger, setup_test_db};
    use crate::model::core::RecordResponse;
    use std::path::Path;

    #[tokio::test]
    async fn test_sync2db() {
        let pool = setup_test_db().await;
        let filepath = Path::new("data/entity_attr/impact_factor_2024.tsv").to_path_buf();
        let _ = JournalMetadata::sync2db(&pool, &filepath, true).await.unwrap();

        let journal_names = vec!["Nature".to_string(), "Science".to_string()];
        let journals = RecordResponse::<JournalMetadata>::get_records(&pool, "biomedgps_journal_metadata", &None, Some(1), Some(10), None, None).await.unwrap();
        assert_ne!(journals.total, 0);
    }

    #[tokio::test]
    async fn test_average_impact_factor() {
        let pool = setup_test_db().await;
        let filepath = Path::new("data/entity_attr/impact_factor_2024.tsv").to_path_buf();
        let _ = JournalMetadata::sync2db(&pool, &filepath, true).await.unwrap();
        let journal_names = vec!["Nature".to_string(), "Science".to_string()];
        let mean = JournalMetadata::average_impact_factor(&pool, journal_names.clone(), Some("mean"))
            .await
            .unwrap();
        let median = JournalMetadata::average_impact_factor(&pool, journal_names.clone(), Some("median"))
            .await
            .unwrap();
        assert_eq!(mean, 42.0);
        assert_eq!(median, 42.0);
    }
}