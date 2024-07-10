//! Add metadata to the entity and relationship model.
use super::entity::compound::CompoundAttr;
use crate::model::core::CheckData;
use crate::query_builder::sql_builder::ComposeQuery;
use anyhow::Ok as AnyOk;
use log::debug;
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use serde_json;
use sqlx::Row;
use std::error::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Object)]
pub struct EntityAttrRecordResponse<S>
where
    S: Serialize
        + std::fmt::Debug
        + std::marker::Unpin
        + Send
        + Sync
        + poem_openapi::types::Type
        + poem_openapi::types::ParseFromJSON
        + poem_openapi::types::ToJSON,
{
    /// data
    pub records: Vec<S>,
    /// total num
    pub total: u64,
    /// current page index
    pub page: u64,
    /// default 10
    pub page_size: u64,
}

/// The context is used to store the context for the LLM. The context can be an entity, an expanded relation, or treatments with disease context.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object)]
pub struct EntityAttr {
    pub compounds: Option<EntityAttrRecordResponse<CompoundAttr>>,
}

impl EntityAttrRecordResponse<CompoundAttr> {
    pub async fn fetch_records(
        pool: &sqlx::PgPool,
        query: &Option<ComposeQuery>,
        page: Option<u64>,
        page_size: Option<u64>,
        order_by: Option<&str>,
    ) -> Result<EntityAttrRecordResponse<CompoundAttr>, anyhow::Error> {
        let mut query_str = match query {
            Some(ComposeQuery::QueryItem(item)) => item.format(),
            Some(ComposeQuery::ComposeQueryItem(item)) => item.format(),
            None => "".to_string(),
        };

        if query_str.is_empty() {
            query_str = "1=1".to_string();
        };

        let order_by_str = if order_by.is_none() {
            "".to_string()
        } else {
            format!("ORDER BY {}", order_by.unwrap())
        };

        let pagination_str = if page.is_none() && page_size.is_none() {
            "LIMIT 10 OFFSET 0".to_string()
        } else {
            let page = match page {
                Some(page) => page,
                None => 1,
            };

            let page_size = match page_size {
                Some(page_size) => page_size,
                None => 10,
            };

            let limit = page_size;
            let offset = (page - 1) * page_size;

            format!("LIMIT {} OFFSET {}", limit, offset)
        };

        let sql_str = format!(
            "SELECT {} FROM {} WHERE {} {} {}",
            CompoundAttr::fields().join(", "),
            "biomedgps_compound_metadata",
            query_str,
            order_by_str,
            pagination_str
        );

        let records = sqlx::query(sql_str.as_str()).fetch_all(pool).await?;

        let sql_str = format!(
            "SELECT COUNT(*) FROM biomedgps_compound_metadata WHERE {}",
            query_str
        );

        let total = sqlx::query_as::<_, (i64,)>(sql_str.as_str())
            .fetch_one(pool)
            .await?;

        let mut new_records = Vec::<CompoundAttr>::new();
        for record in records {
            new_records.push(CompoundAttr {
                compound_type: record.try_get("compound_type")?,
                created: record.try_get("created")?,
                updated: record.try_get("updated")?,
                drugbank_id: record.try_get("drugbank_id")?,
                xrefs: record.try_get("xrefs")?,
                name: record.try_get("name")?,
                description: record.try_get("description")?,
                cas_number: record.try_get("cas_number")?,
                unii: record.try_get("unii")?,
                compound_state: record.try_get("compound_state")?,
                groups: record.try_get("groups")?,
                general_references: serde_json::from_value(record.get("general_references"))?,
                synthesis_reference: record.try_get("synthesis_reference")?,
                indication: record.try_get("indication")?,
                pharmacodynamics: record.try_get("pharmacodynamics")?,
                mechanism_of_action: record.try_get("mechanism_of_action")?,
                toxicity: record.try_get("toxicity")?,
                metabolism: record.try_get("metabolism")?,
                absorption: record.try_get("absorption")?,
                half_life: record.try_get("half_life")?,
                protein_binding: record.try_get("protein_binding")?,
                route_of_elimination: record.try_get("route_of_elimination")?,
                volume_of_distribution: record.try_get("volume_of_distribution")?,
                clearance: record.try_get("clearance")?,
                classification: serde_json::from_value(record.get("classification"))?,
                synonyms: record.try_get("synonyms")?,
                products: serde_json::from_value(record.get("products"))?,
                packagers: serde_json::from_value(record.get("packagers"))?,
                manufacturers: serde_json::from_value(record.get("manufacturers"))?,
                prices: serde_json::from_value(record.get("prices"))?,
                categories: serde_json::from_value(record.get("categories"))?,
                affected_organisms: record.try_get("affected_organisms")?,
                dosages: serde_json::from_value(record.get("dosages"))?,
                atc_codes: serde_json::from_value(record.get("atc_codes"))?,
                patents: serde_json::from_value(record.get("patents"))?,
                food_interactions: record.try_get("food_interactions")?,
                sequences: serde_json::from_value(record.get("sequences"))?,
                experimental_properties: serde_json::from_value(
                    record.get("experimental_properties"),
                )?,
                external_identifiers: serde_json::from_value(record.get("external_identifiers"))?,
                external_links: serde_json::from_value(record.get("external_links"))?,
                targets: serde_json::from_value(record.get("targets"))?,
            });
        }

        AnyOk(EntityAttrRecordResponse {
            records: new_records,
            total: total.0 as u64,
            page: page.unwrap_or(1),
            page_size: page_size.unwrap_or(10),
        })
    }
}
