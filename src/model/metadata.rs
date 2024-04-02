//! Add metadata to the entity and relationship model.
use log::debug;
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use serde_json;
use std::{error::Error, path::PathBuf};
use validator::{Validate, ValidationErrors};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Object, sqlx::FromRow, Validate)]
pub struct Category {
    pub category: String,
    pub mesh_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Object, sqlx::FromRow, Validate)]
pub struct Patent {
    pub number: String,
    pub country: String,
    pub approved: String,
    pub expires: String,
    pub pediatric_extension: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Object, sqlx::FromRow, Validate)]
pub struct CompoundMetadata {
    pub compound_type: String,
    pub created: String,
    pub updated: String,
    pub drugbank_id: Vec<String>,
    pub name: String,
    pub description: String,
    pub cas_number: String,
    pub unii: String,
    pub compound_state: String,
    pub groups: Vec<String>,
    // pub general_references: GeneralReference,
    pub synthesis_reference: String,
    pub indication: String,
    pub pharmacodynamics: String,
    pub mechanism_of_action: String,
    pub toxicity: String,
    pub metabolism: String,
    pub absorption: String,
    pub half_life: String,
    pub protein_binding: String,
    pub route_of_elimination: String,
    pub volume_of_distribution: String,
    pub clearance: String,
    // pub classification: Classification,
    // pub salts: Vec<String>,
    pub synonyms: Vec<String>,
    // pub products: Vec<Product>,
    // pub international_brands: Vec<InternationalBrand>,
    // pub mixtures: Vec<Mixture>,
    // pub packagers: Vec<Packager>,
    // pub manufacturers: Vec<Manufacturer>,
    // pub prices: Vec<Price>,
    pub categories: Vec<Category>,
    // pub affected_organisms: Vec<String>,
    // pub dosages: Vec<Dosage>,
    // pub atc_codes: Vec<AtcCode>,
    // pub ahfs_codes: Vec<AhfsCode>,
    // pub pdb_entries: Vec<PdbEntry>,
    pub patents: Vec<Patent>,
    // pub food_interactions: Vec<String>,
    // pub drug_interactions: Vec<DrugInteraction>,
    // pub sequences: Vec<Sequence>,
    // pub experimental_properties: ExperimentalProperty,
    // pub external_identifiers: Vec<ExternalIdentifier>,
    // pub external_links: Vec<ExternalLink>,
    // pub pathways: Vec<Pathway>,
    // pub reactions: Vec<Reaction>,
    // pub snp_effects: Vec<SnpEffect>,
    // pub snp_adverse_drug_reactions: Vec<SnpAdverseDrugReaction>,
    // pub targets: Vec<Target>,
    // pub enzymes: Vec<Enzyme>,
    // pub carriers: Vec<Carrier>,
    // pub transporters: Vec<Transporter>,
}

impl CompoundMetadata {
    pub async fn sync2db(pool: &sqlx::PgPool, filepath: &PathBuf) -> Result<(), Box<dyn Error>> {
        match sqlx::query("DROP TABLE IF EXISTS staging")
            .execute(pool)
            .await
        {
            Ok(_) => debug!("Drop table staging successfully."),
            Err(e) => debug!("Drop table staging failed: {:?}", e),
        }

        let mut tx = pool.begin().await?;
        sqlx::query(
            "CREATE TEMPORARY TABLE staging (LIKE biomedgps_compound_metadata INCLUDING DEFAULTS)",
        )
        .execute(&mut tx)
        .await?;

        let columns = Self::fields().join(", ");
        let query_str = format!(
            "COPY staging ({}) FROM {} WITH (FORMAT JSON)",
            columns,
            filepath.display()
        );

        debug!("Start to copy data to the staging table.");
        sqlx::query(&query_str).execute(&mut tx).await?;

        let where_clause = Self::unique_fields()
            .iter()
            .map(|c| format!("biomedgps_compound_metadata.{} = staging.{}", c, c))
            .collect::<Vec<String>>()
            .join(" AND ");

        sqlx::query(&format!(
            "INSERT INTO biomedgps_compound_metadata ({})
             SELECT {} FROM staging
             WHERE NOT EXISTS (SELECT 1 FROM biomedgps_compound_metadata WHERE {})
             ON CONFLICT DO NOTHING",
            columns, columns, where_clause
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
}

pub trait CheckMetadata {
    fn check_json_is_valid(filepath: &PathBuf) -> Vec<Box<ValidationErrors>>;

    // Implement the check function
    fn check_json_is_valid_default<
        S: for<'de> serde::Deserialize<'de> + Validate + std::fmt::Debug,
    >(
        filepath: &PathBuf,
    ) -> Vec<Box<ValidationErrors>> {
        let file = std::fs::File::open(filepath).unwrap();
        let reader = std::io::BufReader::new(file);
        let data: Vec<S> = serde_json::from_reader(reader).unwrap();
        let mut errors: Vec<Box<ValidationErrors>> = Vec::new();
        for d in data.iter() {
            match d.validate() {
                Ok(_) => {}
                Err(e) => {
                    errors.push(Box::new(e));
                }
            }
        }
        errors
    }

    fn fields() -> Vec<String>;

    fn unique_fields() -> Vec<String>;

    fn get_error_msg<S: for<'de> serde::Deserialize<'de> + Validate + std::fmt::Debug>(
        r: Result<Vec<S>, Box<ValidationErrors>>,
    ) -> String {
        match r {
            Ok(_) => "".to_string(),
            Err(e) => {
                return e.to_string();
            }
        }
    }
}

impl CheckMetadata for CompoundMetadata {
    fn check_json_is_valid(filepath: &PathBuf) -> Vec<Box<ValidationErrors>> {
        Self::check_json_is_valid_default::<CompoundMetadata>(filepath)
    }

    fn unique_fields() -> Vec<String> {
        vec!["name".to_string()]
    }

    fn fields() -> Vec<String> {
        vec![
            "compound_type".to_string(),
            "created".to_string(),
            "updated".to_string(),
            "drugbank_id".to_string(),
            "name".to_string(),
            "description".to_string(),
            "cas_number".to_string(),
            "unii".to_string(),
            "compound_state".to_string(),
            "groups".to_string(),
            "synthesis_reference".to_string(),
            "indication".to_string(),
            "pharmacodynamics".to_string(),
            "mechanism_of_action".to_string(),
            "toxicity".to_string(),
            "metabolism".to_string(),
            "absorption".to_string(),
            "half_life".to_string(),
            "protein_binding".to_string(),
            "route_of_elimination".to_string(),
            "volume_of_distribution".to_string(),
            "clearance".to_string(),
            "categories".to_string(),
            "patents".to_string(),
            "synonyms".to_string(),
        ]
    }
}
