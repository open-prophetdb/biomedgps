use log::{debug, info};
use std::{error::Error, path::PathBuf};

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

pub async fn import_file_in_loop(
    pool: &sqlx::PgPool,
    filepath: &PathBuf,
    table_name: &str,
    expected_columns: &Vec<String>,
    unique_columns: &Vec<String>,
    delimiter: u8,
) -> Result<(), Box<dyn Error>> {
    match sqlx::query("DROP TABLE staging").execute(pool).await {
        Ok(_) => {}
        Err(_) => {}
    }

    let mut tx = pool.begin().await?;
    // Here we replace '{}' and {} placeholders with the appropriate values.
    sqlx::query(&format!(
        "CREATE TEMPORARY TABLE staging (LIKE {} INCLUDING ALL)",
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
         WHERE NOT EXISTS (SELECT 1 FROM {} WHERE {})",
        table_name, columns, columns, table_name, where_clause
    ))
    .execute(&mut tx)
    .await?;

    tx.commit().await?;

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

pub async fn update_relation_metadata(
    pool: &sqlx::PgPool,
    drop: bool,
) -> Result<(), Box<dyn Error>> {
    let table_name = "biomedgps_relation_metadata";
    if drop {
        drop_table(&pool, table_name).await;
    };

    info!("Update relation metadata from relation table.");

    let query_str = format!("
        INSERT INTO {} (relation_type, start_entity_type, end_entity_type, relation_count, resource)
        SELECT relation_type, source_type as start_entity_type, target_type as end_entity_type, count(*) as relation_count, resource
        FROM biomedgps_relation
        GROUP BY relation_type, source_type, target_type, resource;
    ", table_name);

    sqlx::query(&query_str)
        .execute(pool)
        .await
        .expect("Failed to update data.");
    info!("{} updated.", table_name);

    Ok(())
}
