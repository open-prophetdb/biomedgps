//! Add metadata to the entity and relationship model.
use crate::model::core::CheckData;
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::File;
use std::io::BufReader;
use std::{error::Error, path::PathBuf};
use validator::Validate;
use log::debug;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Object, sqlx::FromRow, Validate)]
pub struct Category {
    pub category: String,
    #[serde(default = "default_as_empty_string")]
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

fn default_as_empty_string() -> String {
    String::new()
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Object, sqlx::FromRow, Validate)]
pub struct CompoundMetadata {
    #[serde(default = "default_as_empty_string")]
    pub compound_type: String,
    #[serde(default = "default_as_empty_string")]
    pub created: String,
    #[serde(default = "default_as_empty_string")]
    pub updated: String,
    pub drugbank_id: String,
    pub xrefs: Vec<String>,
    pub name: String,
    #[serde(default = "default_as_empty_string")]
    pub description: String,
    #[serde(default = "default_as_empty_string")]
    pub cas_number: String,
    #[serde(default = "default_as_empty_string")]
    pub unii: String,
    #[serde(default = "default_as_empty_string")]
    pub compound_state: String,
    pub groups: Vec<String>,
    // pub general_references: GeneralReference,
    #[serde(default = "default_as_empty_string")]
    pub synthesis_reference: String,
    #[serde(default = "default_as_empty_string")]
    pub indication: String,
    #[serde(default = "default_as_empty_string")]
    pub pharmacodynamics: String,
    #[serde(default = "default_as_empty_string")]
    pub mechanism_of_action: String,
    #[serde(default = "default_as_empty_string")]
    pub toxicity: String,
    #[serde(default = "default_as_empty_string")]
    pub metabolism: String,
    #[serde(default = "default_as_empty_string")]
    pub absorption: String,
    #[serde(default = "default_as_empty_string")]
    pub half_life: String,
    #[serde(default = "default_as_empty_string")]
    pub protein_binding: String,
    #[serde(default = "default_as_empty_string")]
    pub route_of_elimination: String,
    #[serde(default = "default_as_empty_string")]
    pub volume_of_distribution: String,
    #[serde(default = "default_as_empty_string")]
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

impl CheckData for CompoundMetadata {
    fn check_csv_is_valid(filepath: &PathBuf) -> Vec<Box<dyn Error>> {
        Self::check_csv_is_valid_default::<CompoundMetadata>(filepath)
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
            "xrefs".to_string(),
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

impl CompoundMetadata {
    pub async fn sync2db(pool: &sqlx::PgPool, filepath: &PathBuf, clean_table: bool) -> Result<(), Box<dyn Error>> {
        if clean_table {
            let _ = sqlx::query("TRUNCATE TABLE biomedgps_compound_metadata")
                .execute(pool)
                .await?;
        };
        let file = File::open(filepath)?;
        let reader = BufReader::new(file);
        let compounds: Vec<CompoundMetadata> = serde_json::from_reader(reader)?;

        let mut tx = pool.begin().await?;
        let fields_str = CompoundMetadata::fields().join(", ");
        let placeholders_str = CompoundMetadata::fields().iter().enumerate()
            .map(|(i, field)| {
                if field == "categories" || field == "patents" {
                    format!("${}", i + 1)
                } else {
                    format!("${}", i + 1)
                }
            })
            .collect::<Vec<String>>()
            .join(", ");
            
        let sql = format!(
            "INSERT INTO biomedgps_compound_metadata ({}) VALUES ({})",
            fields_str, placeholders_str
        );

        for compound in compounds {
            debug!("Name: {:?}", compound.name);
            debug!("Groups: {:?}", compound.groups);
            debug!("Synonyms: {:?}", compound.synonyms);

            let result = sqlx::query(&sql)
                .bind(&compound.compound_type)
                .bind(&compound.created)
                .bind(&compound.updated)
                .bind(&compound.drugbank_id)
                .bind(&compound.xrefs)
                .bind(&compound.name)
                .bind(&compound.description)
                .bind(&compound.cas_number)
                .bind(&compound.unii)
                .bind(&compound.compound_state)
                .bind(&compound.groups)
                .bind(&compound.synthesis_reference)
                .bind(&compound.indication)
                .bind(&compound.pharmacodynamics)
                .bind(&compound.mechanism_of_action)
                .bind(&compound.toxicity)
                .bind(&compound.metabolism)
                .bind(&compound.absorption)
                .bind(&compound.half_life)
                .bind(&compound.protein_binding)
                .bind(&compound.route_of_elimination)
                .bind(&compound.volume_of_distribution)
                .bind(&compound.clearance)
                .bind(&serde_json::to_value(compound.categories)?)
                .bind(&serde_json::to_value(compound.patents)?)
                .bind(&compound.synonyms)
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
