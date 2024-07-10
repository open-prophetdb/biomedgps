//! Add metadata to the entity and relationship model.
use crate::model::core::CheckData;
use log::debug;
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use serde_json;
use sqlx::{FromRow, Row};
use std::fs::File;
use std::io::BufReader;
use std::{error::Error, path::PathBuf};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Object, FromRow, Validate)]
pub struct Category {
    pub category: String,
    #[serde(default = "default_as_empty_string")]
    pub mesh_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Object, FromRow, Validate)]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Object, FromRow, Validate)]
pub struct Article {
    pub ref_id: String,
    pub pubmed_id: String,
    pub citation: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Object, FromRow, Validate)]
pub struct Link {
    pub ref_id: String,
    pub title: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Object, FromRow, Validate)]
pub struct GeneralReferences {
    pub articles: Vec<Article>,
    pub links: Vec<Link>,
    // pub textbooks: Vec<Textbook>,
    // pub attachments: Vec<Attachment>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Object, FromRow, Validate)]
pub struct Classification {
    pub description: String,
    pub direct_parent: String,
    pub kingdom: String,
    pub superclass: String,
    pub class: String,
    pub subclass: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Object, FromRow, Validate)]
pub struct Product {
    pub name: String,
    pub labeller: String,
    pub ndc_id: String,
    pub ndc_product_code: String,
    pub dpd_id: String,
    pub ema_product_code: String,
    pub ema_ma_number: String,
    pub started_marketing_on: String,
    pub ended_marketing_on: String,
    pub dosage_form: String,
    pub strength: String,
    pub route: String,
    pub fda_application_number: String,
    pub generic: String,
    pub over_the_counter: String,
    pub approved: String,
    pub country: String,
    pub source: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Object, FromRow, Validate)]
pub struct Packager {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Object, FromRow, Validate)]
pub struct Manufacturer {
    pub text: String,
    pub generic: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Object, FromRow, Validate)]
pub struct Cost {
    pub currency: String,
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Object, FromRow, Validate)]
pub struct Price {
    pub description: String,
    pub cost: Cost,
    pub unit: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Object, FromRow, Validate)]
pub struct Dosage {
    pub form: String,
    pub route: String,
    pub strength: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Object, FromRow, Validate)]
pub struct AtcCodeLevel {
    pub code: String,
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Object, FromRow, Validate)]
pub struct AtcCode {
    pub code: String,
    pub level: Vec<AtcCodeLevel>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Object, FromRow, Validate)]
pub struct Sequence {
    #[serde(default)]
    pub text: String,
    pub format: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Object, FromRow, Validate)]
pub struct ExternalIdentifier {
    pub resource: String,
    pub identifier: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Object, FromRow, Validate)]
pub struct ExternalLink {
    pub resource: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Object, FromRow, Validate)]
pub struct Property {
    pub kind: String,
    pub value: String,
    pub source: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Object, FromRow, Validate)]
pub struct ExperimentalProperty {
    pub property: Vec<Property>,
}

fn default_as_experimental_property() -> ExperimentalProperty {
    ExperimentalProperty { property: vec![] }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Object, FromRow, Validate)]
pub struct Target {
    pub position: String,
    pub id: String,
    pub name: String,
    pub organism: String,
    pub actions: Vec<String>,
    pub references: GeneralReferences,
    pub known_action: String,
    pub polypeptide: Option<Vec<Polypeptide>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Object, FromRow, Validate)]
pub struct Organism {
    pub text: String,
    pub ncbi_taxonomy_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Object, FromRow, Validate)]
pub struct Pfam {
    pub identifier: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Object, FromRow, Validate)]
pub struct GoClassifier {
    pub category: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Object, FromRow, Validate)]
pub struct Polypeptide {
    pub id: String,
    pub source: String,
    pub name: String,
    pub general_function: String,
    pub specific_function: String,
    pub gene_name: String,
    pub locus: String,
    pub cellular_location: String,
    pub transmembrane_regions: String,
    pub signal_regions: String,
    pub theoretical_pi: String,
    pub molecular_weight: String,
    pub chromosome_location: String,
    pub organism: Organism,
    pub external_identifiers: Vec<ExternalIdentifier>,
    pub synonyms: Vec<String>,
    pub amino_acid_sequence: Sequence,
    pub gene_sequence: Sequence,
    pub pfams: Vec<Pfam>,
    pub go_classifiers: Vec<GoClassifier>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Object, FromRow, Validate)]
pub struct CompoundAttr {
    #[serde(default = "default_as_empty_string")]
    pub compound_type: String,
    #[serde(default = "default_as_empty_string")]
    pub created: String,
    #[serde(default = "default_as_empty_string")]
    pub updated: String,
    pub drugbank_id: String,
    #[serde(default)]
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
    #[serde(default)]
    pub groups: Vec<String>,
    pub general_references: Option<GeneralReferences>,
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
    pub classification: Option<Classification>,
    // pub salts: Vec<String>,
    #[serde(default)]
    pub synonyms: Vec<String>,
    #[serde(default)]
    pub products: Vec<Product>,
    // pub international_brands: Vec<InternationalBrand>,
    // pub mixtures: Vec<Mixture>,
    #[serde(default)]
    pub packagers: Vec<Packager>,
    #[serde(default)]
    pub manufacturers: Vec<Manufacturer>,
    #[serde(default)]
    pub prices: Vec<Price>,
    #[serde(default)]
    pub categories: Vec<Category>,
    #[serde(default)]
    pub affected_organisms: Vec<String>,
    #[serde(default)]
    pub dosages: Vec<Dosage>,
    #[serde(default)]
    pub atc_codes: Vec<AtcCode>,
    // pub ahfs_codes: Vec<AhfsCode>,
    // pub pdb_entries: Vec<PdbEntry>,
    #[serde(default)]
    pub patents: Vec<Patent>,
    #[serde(default)]
    pub food_interactions: Vec<String>,
    // pub drug_interactions: Vec<DrugInteraction>,
    #[serde(default)]
    pub sequences: Vec<Sequence>,
    #[serde(default)]
    pub experimental_properties: Option<ExperimentalProperty>,
    #[serde(default)]
    pub external_identifiers: Vec<ExternalIdentifier>,
    #[serde(default)]
    pub external_links: Vec<ExternalLink>,
    // pub pathways: Vec<Pathway>,
    // pub reactions: Vec<Reaction>,
    // pub snp_effects: Vec<SnpEffect>,
    // pub snp_adverse_drug_reactions: Vec<SnpAdverseDrugReaction>,
    #[serde(default)]
    pub targets: Vec<Target>,
    // pub enzymes: Vec<Enzyme>,
    // pub carriers: Vec<Carrier>,
    // pub transporters: Vec<Transporter>,
}

impl CheckData for CompoundAttr {
    fn check_csv_is_valid(filepath: &PathBuf) -> Vec<Box<dyn Error>> {
        Self::check_csv_is_valid_default::<CompoundAttr>(filepath)
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
            "general_references".to_string(),
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
            "classification".to_string(),
            "synonyms".to_string(),
            "products".to_string(),
            "packagers".to_string(),
            "manufacturers".to_string(),
            "prices".to_string(),
            "categories".to_string(),
            "affected_organisms".to_string(),
            "dosages".to_string(),
            "atc_codes".to_string(),
            "patents".to_string(),
            "food_interactions".to_string(),
            "sequences".to_string(),
            "experimental_properties".to_string(),
            "external_identifiers".to_string(),
            "external_links".to_string(),
            "targets".to_string(),
        ]
    }
}

impl CompoundAttr {
    pub async fn sync2db(
        pool: &sqlx::PgPool,
        filepath: &PathBuf,
        clean_table: bool,
    ) -> Result<(), Box<dyn Error>> {
        if clean_table {
            let _ = sqlx::query("TRUNCATE TABLE biomedgps_compound_metadata")
                .execute(pool)
                .await?;
        };
        let file = File::open(filepath)?;
        let reader = BufReader::new(file);
        let compounds: Vec<CompoundAttr> = serde_json::from_reader(reader)?;

        let mut tx = pool.begin().await?;
        let fields_str = CompoundAttr::fields().join(", ");
        let placeholders_str = CompoundAttr::fields()
            .iter()
            .enumerate()
            .map(|(i, field)| format!("${}", i + 1))
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
                .bind(&serde_json::to_value(compound.general_references)?)
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
                .bind(&serde_json::to_value(compound.classification)?)
                .bind(&compound.synonyms)
                .bind(&serde_json::to_value(compound.products)?)
                .bind(&serde_json::to_value(compound.packagers)?)
                .bind(&serde_json::to_value(compound.manufacturers)?)
                .bind(&serde_json::to_value(compound.prices)?)
                .bind(&serde_json::to_value(compound.categories)?)
                .bind(&compound.affected_organisms)
                .bind(&serde_json::to_value(compound.dosages)?)
                .bind(&serde_json::to_value(compound.atc_codes)?)
                .bind(&serde_json::to_value(compound.patents)?)
                .bind(&compound.food_interactions)
                .bind(&serde_json::to_value(compound.sequences)?)
                .bind(&serde_json::to_value(compound.experimental_properties)?)
                .bind(&serde_json::to_value(compound.external_identifiers)?)
                .bind(&serde_json::to_value(compound.external_links)?)
                .bind(&serde_json::to_value(compound.targets)?)
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
