//! Utility functions for the model module. Contains functions to import data from CSV files into the database, and to update the metadata tables.

use itertools::Itertools;
use lazy_static::lazy_static;
use log::{debug, error, info, warn};
use polars::prelude::IntoVec;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Mutex;
use std::{error::Error, fmt, path::PathBuf};

lazy_static! {
    static ref EXISTING_COLORS: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

/// Custom Error type for the graph module
#[derive(Debug)]
pub struct ValidationError {
    pub details: String,
    pub data: Vec<String>,
}

impl ValidationError {
    pub fn new(msg: &str, data: Vec<String>) -> ValidationError {
        ValidationError {
            details: msg.to_string(),
            data,
        }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for ValidationError {
    fn description(&self) -> &str {
        &self.details
    }

    fn cause(&self) -> Option<&dyn Error> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

/// A color map for the node labels.
/// More details on https://colorbrewer2.org/#type=qualitative&scheme=Paired&n=12
/// Don't change the order of the colors. It is important to keep the colors consistent.
/// In future, we may specify a color for each node label when we can know all the node labels.
const NODE_COLORS: [&str; 12] = [
    "#ffff99", "#6a3d9a", "#ff7f00", "#b2df8a", "#a6cee3", "#e31a1c", "#fdbf6f", "#fb9a99",
    "#cab2d6", "#33a02c", "#b15928", "#1f78b4",
];

const DEFAULT_COLOR: &str = "#000000";

/// We have a set of colors and we want to match a color to a node label in a deterministic way.
/// Examples: { "SideEffect": "0", "Pathway": "2", "Symptom": "2", "MolecularFunction": "3", "Metabolite": "4", "Gene": "5", "PharmacologicClass": "6", "Disease": "7", "CellularComponent": "8", "Compound": "9", "BiologicalProcess": "10", "Anatomy": "11" }
/// { "Gene": ["#e31a1c", "Red"], "Compound": ["#33a02c", "Green"], "Disease": ["#fb9a99", "Light Pink"], "Pathway": ["#6a3d9a", "Purple"], "Anatomy": ["#1f78b4", "Dark Blue"], "BiologicalProcess": ["#b15928", "Brown"], "CellularComponent": ["#cab2d6", "Lavender"], "Metabolite": ["#a6cee3", "Light Blue"], "MolecularFunction": ["#b2df8a", "Light Green"], "PharmacologicClass": ["#fdbf6f", "Peach"], "SideEffect": ["#ffff99", "Yellow"], "Symptom": ["#ff7f00", "Orange"] }
/// TODO: We might get the same color for different node labels. We need to handle this case. For example, we can load a user-defined color map from a settings file.
pub fn match_color(entity_type: &str) -> String {
    let mut hasher = DefaultHasher::new();
    entity_type.hash(&mut hasher);
    let mut hash = hasher.finish();

    let mut existing_colors = EXISTING_COLORS.lock().unwrap();

    if let Some(color) = existing_colors.get(entity_type) {
        return color.to_string();
    }

    let mut attempts = 0;
    while attempts < NODE_COLORS.len() {
        let index = hash % NODE_COLORS.len() as u64;
        let color = NODE_COLORS[index as usize];

        if !existing_colors.values().any(|v| v == color) {
            existing_colors.insert(entity_type.to_string(), color.to_string());
            return color.to_string();
        }

        hash += 1;
        attempts += 1;
    }

    DEFAULT_COLOR.to_string()
}

pub fn read_annotation_file(filepath: &PathBuf) -> Result<HashMap<String, String>, Box<dyn Error>> {
    let delimiter = get_delimiter(filepath).unwrap();

    // Read the annotation file into a hashmap.
    let mut relation_type_mappings = std::collections::HashMap::new();
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .delimiter(delimiter)
        .from_path(filepath)
        .unwrap();

    // Check the annotation file format.
    let headers = reader.headers().unwrap();
    for col in ["relation_type", "formatted_relation_type"].iter() {
        if !headers.into_vec().contains(&col.to_string()) {
            return Err(format!("The annotation file should have two columns: relation_type and formatted_relation_type. But we found that the annotation file has no {} column.", col).into());
        }
    }

    let relation_type_index = headers.iter().position(|h| h == "relation_type").unwrap();
    let formatted_relation_type_index = headers
        .iter()
        .position(|h| h == "formatted_relation_type")
        .unwrap();

    // Only get the relation_type and formatted_relation_type columns from the annotation file.
    for result in reader.records() {
        let record = result.unwrap();
        let relation_type = record.get(relation_type_index).unwrap().to_string();
        let formatted_relation_type = record
            .get(formatted_relation_type_index)
            .unwrap()
            .to_string();
        relation_type_mappings.insert(relation_type, formatted_relation_type);
    }

    Ok(relation_type_mappings)
}

/// Update the existing colors with the new entity types.
pub fn update_existing_colors(entity_types: &Vec<String>) {
    // Order the entity types by their names.
    let mut entity_types = entity_types.clone();
    entity_types.sort();

    // Assign colors to the entity types.
    for (i, entity_type) in entity_types.iter().enumerate() {
        // Find the color for the entity type and update the existing colors.
        match_color(entity_type);
    }
}

pub fn get_delimiter(filepath: &PathBuf) -> Result<u8, Box<dyn Error>> {
    let suffix = match filepath.extension() {
        Some(suffix) => suffix.to_str().unwrap(),
        None => return Err("File has no extension".into()),
    };

    if suffix == "csv" {
        Ok(b',')
    } else if suffix == "tsv" {
        Ok(b'\t')
    } else if suffix == "txt" {
        Ok(b' ')
    } else {
        Err(format!("Unsupported file type: {}", suffix).into())
    }
}

pub async fn drop_table(pool: &sqlx::PgPool, table: &str) {
    debug!("Dropping table {}...", table);
    sqlx::query(&format!(
        "
        DO $$ BEGIN
        IF EXISTS (SELECT FROM information_schema.tables 
                    WHERE  table_schema = 'public' 
                    AND    table_name   = '{}')
        THEN
            DELETE FROM {};
        END IF;
        END $$;
        ",
        table, table
    ))
    .execute(pool)
    .await
    .unwrap();
}

pub async fn drop_records(pool: &sqlx::PgPool, table: &str, colname: &str, colvalue: &str) {
    debug!("Dropping records from table {}...", table);
    sqlx::query(&format!(
        "
        DELETE FROM {} WHERE {} = '{}';
        ",
        table, colname, colvalue
    ))
    .execute(pool)
    .await
    .unwrap();
}

pub async fn import_file_in_loop(
    pool: &sqlx::PgPool,
    filepath: &PathBuf,
    table_name: &str,
    expected_columns: &Vec<String>,
    unique_columns: &Vec<String>,
    delimiter: u8,
) -> Result<(), Box<dyn Error>> {
    match sqlx::query("DROP TABLE IF EXISTS staging")
        .execute(pool)
        .await
    {
        Ok(_) => {}
        Err(_) => {}
    }

    let mut tx = pool.begin().await?;
    // Here we replace '{}' and {} placeholders with the appropriate values.
    sqlx::query(&format!(
        "CREATE TEMPORARY TABLE staging (LIKE {} INCLUDING DEFAULTS)",
        table_name
    ))
    .execute(&mut tx)
    .await?;

    let columns = expected_columns.join(",");
    let query_str = format!(
        "COPY staging ({}) FROM '{}' DELIMITER E'{}' CSV HEADER",
        columns,
        filepath.display(),
        delimiter as char
    );

    debug!("Importing query string: {}", query_str);

    sqlx::query(&query_str).execute(&mut tx).await?;

    let where_clause = unique_columns
        .iter()
        .map(|c| format!("{}.{} = staging.{}", table_name, c, c))
        .collect::<Vec<String>>()
        .join(" AND ");

    sqlx::query(&format!(
        "INSERT INTO {} ({})
         SELECT {} FROM staging
         WHERE NOT EXISTS (SELECT 1 FROM {} WHERE {})
         ON CONFLICT DO NOTHING",
        table_name, columns, columns, table_name, where_clause
    ))
    .execute(&mut tx)
    .await?;

    tx.commit().await?;

    match sqlx::query("DROP TABLE IF EXISTS staging")
        .execute(pool)
        .await
    {
        Ok(_) => {}
        Err(_) => {}
    };

    Ok(())
}

pub async fn import_file(
    pool: &sqlx::PgPool,
    filepath: &PathBuf,
    table_name: &str,
    expected_columns: &Vec<String>,
    delimiter: u8,
    drop: bool,
) -> Result<(), Box<dyn Error>> {
    if drop {
        drop_table(&pool, table_name).await;
    };

    let columns = expected_columns.join(", ");

    let stmt = format!(
        r#"COPY {} ({}) FROM '{}' DELIMITER E'{}' CSV HEADER;"#,
        table_name,
        columns,
        filepath.display(),
        delimiter as char
    );

    debug!("Importing query string: {}", stmt);

    sqlx::query(&stmt)
        .execute(pool)
        .await
        .expect("Failed to import data.");
    info!("{} imported.", filepath.display());

    Ok(())
}

pub async fn update_entity_metadata(pool: &sqlx::PgPool, drop: bool) -> Result<(), Box<dyn Error>> {
    let table_name = "biomedgps_entity_metadata";
    if drop {
        drop_table(&pool, table_name).await;
    };

    info!("Update entity metadata from entity table.");

    let query_str = format!(
        "
        INSERT INTO {} (resource, entity_type, entity_count)
        SELECT resource, label as entity_type, count(*) as entity_count
        FROM biomedgps_entity
        GROUP BY resource, label;
    ",
        table_name
    );

    sqlx::query(&query_str)
        .execute(pool)
        .await
        .expect("Failed to update data.");
    info!("{} updated.", table_name);

    Ok(())
}

#[derive(Debug, serde::Deserialize)]
struct RelationMetadata {
    relation_type: String,
    description: String,
}

pub async fn update_relation_metadata(
    pool: &sqlx::PgPool,
    metadata_filepath: &PathBuf,
    drop: bool,
) -> Result<(), Box<dyn Error>> {
    let table_name = "biomedgps_relation_metadata";
    if drop {
        drop_table(&pool, table_name).await;
    };

    info!("Load relation metadata from an annotation file.");

    let delimiter = get_delimiter(metadata_filepath)?;
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .from_path(metadata_filepath)?;

    let headers = reader.headers().unwrap();
    for col in ["relation_type", "description"].iter() {
        if !headers.into_iter().contains(col) {
            return Err(format!(
                "Column {} not found in the {} file. You should specify a file with the columns 'relation_type' and 'description' for annotating the relation types in the relation table.",
                col,
                metadata_filepath.display()
            )
            .into());
        }
    }

    let mut records = Vec::new();
    for result in reader.deserialize::<RelationMetadata>() {
        let record: RelationMetadata = result?;
        records.push(record);
    }

    info!("Update relation metadata from relation table.");

    let query_str = format!("
        INSERT INTO {} (relation_type, formatted_relation_type, start_entity_type, end_entity_type, relation_count, resource, dataset)
        SELECT relation_type, formatted_relation_type, source_type as start_entity_type, target_type as end_entity_type, count(*) as relation_count, resource, dataset
        FROM biomedgps_relation
        GROUP BY relation_type, formatted_relation_type, source_type, target_type, resource, dataset;
    ", table_name);

    sqlx::query(&query_str)
        .execute(pool)
        .await
        .expect("Failed to update data.");

    // Update the description of the relation types.
    let mut tx = pool.begin().await?;
    for record in records {
        sqlx::query(
            "
            UPDATE biomedgps_relation_metadata
            SET description = $1
            WHERE relation_type = $2;
        ",
        )
        .bind(record.description)
        .bind(record.relation_type)
        .execute(&mut tx)
        .await?;
    }
    tx.commit().await?;

    info!("{} updated.", table_name);

    Ok(())
}

pub fn parse_csv_error(e: &csv::Error) -> String {
    match *e.kind() {
        csv::ErrorKind::Deserialize {
            pos: Some(ref pos),
            ref err,
            ..
        } => {
            format!(
                "Failed to deserialize the data, line: {}, column: {}, details: ({})",
                pos.line(),
                pos.record() + 1,
                err.kind()
            )
        }
        _ => {
            format!("Failed to parse CSV: ({})", e)
        }
    }
}

pub fn show_errors(errors: &Vec<Box<dyn std::error::Error>>, show_all_errors: bool) {
    if !show_all_errors {
        let total = errors.len();
        let num = if total > 3 { 3 } else { total };
        warn!("Found {} errors, only show the {} validation errors, if you want to see all errors, use --show-all-errors.", total, num);
        for e in errors.iter().take(3) {
            error!("{}", e);
        }

        if total == num {
            return;
        } else {
            warn!("Hide {} validation errors.", errors.len() - num);
        }
    } else {
        for e in errors {
            error!("{}", e);
        }
    }
}
