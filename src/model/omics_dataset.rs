//! The database schema for the omics datasets. These are the models that will be used to interact with the omics datasets using the duckdb.

use lazy_static::lazy_static;
use log::{info, error};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::Mutex;
use validator::{Validate, ValidateArgs, ValidationError};

lazy_static! {
    static ref OmicsDatasetRootDir: Mutex<PathBuf> = Mutex::new(PathBuf::new());
}

pub fn merge_json_files(
    metadata_files: &Vec<PathBuf>,
    output_file: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut output = fs::File::create(output_file)?;
    output.write_all(b"[")?;

    for (i, metadata_file) in metadata_files.iter().enumerate() {
        let content = fs::read_to_string(metadata_file)?;
        if i > 0 {
            output.write_all(b",")?;
        }
        output.write_all(content.as_bytes())?;
    }

    output.write_all(b"]")?;
    Ok(())
}

/// Read, parse and return the metadata of the omics datasets.
pub fn merge_json_files_safe(metadata_files: Vec<PathBuf>) -> Vec<OmicsDatasetMetadata> {
    let mut metadata_list = Vec::new();
    for metadata_file in metadata_files {
        match fs::read_to_string(&metadata_file) {
            Ok(content) => match serde_json::from_str::<OmicsDatasetMetadata>(&content) {
                Ok(metadata) => match metadata.validate() {
                    Ok(_) => {
                        metadata_list.push(metadata);
                        info!(
                            "Successfully processed metadata: {}",
                            metadata_file.display()
                        );
                    }
                    Err(e) => {
                        error!(
                            "Failed to validate metadata: {}",
                            e.to_string()
                        );
                    }
                },
                Err(e) => {
                    error!(
                        "Failed to parse metadata file '{}': {}",
                        metadata_file.display(),
                        e
                    );
                }
            },
            Err(e) => {
                error!(
                    "Failed to read metadata file '{}': {}",
                    metadata_file.display(),
                    e
                );
            }
        }
    }

    metadata_list
}

/// Initialize the omics dataset root directory, it should be called when the server starts.
///
/// Args:
///   - root_dir: The root directory of the omics datasets.
///   - safe_mode: If true, the merge function will check each field in the metadata.json file first. [TODO: more checking logic will be added later]
///
/// Returns:
///   - Result<(), Box<dyn std::error::Error>>: The result of the operation.
pub fn init_omics_dataset_root_dir(
    root_dir: &PathBuf,
    safe_mode: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut omics_dataset_root_dir = OmicsDatasetRootDir.lock().unwrap();
    *omics_dataset_root_dir = root_dir.clone();
    let metadata_file = omics_dataset_root_dir.join("metadata.json");

    // We must release the lock here, otherwise the validate_* function will not be able to access the omics_dataset_root_dir.
    drop(omics_dataset_root_dir);

    // List all the subdirectories in the root directory and get the metadata.json file in each subdirectory.
    let metadata_files = std::fs::read_dir(root_dir)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            if entry.file_type().ok()?.is_dir() {
                Some(entry.path().join("metadata.json"))
            } else {
                None
            }
        })
        .collect::<Vec<PathBuf>>();

    // Merge all the metadata.json files into a single metadata.json file.
    if !safe_mode {
        merge_json_files(
            &metadata_files,
            &metadata_file,
        )?;
    } else {
        let metadata_list = merge_json_files_safe(metadata_files);
        let metadata_str = serde_json::to_string(&metadata_list)?;
        fs::write(&metadata_file, metadata_str)?;
    }

    Ok(())
}

/// Get the omics dataset root directory.
pub fn get_omics_dataset_root_dir() -> PathBuf {
    OmicsDatasetRootDir.lock().unwrap().clone()
}

fn validate_case_group(case_group: &CaseGroup) -> Result<(), ValidationError> {
    if case_group.case_group_id.is_empty() || case_group.case_group_name.is_empty() {
        let mut error = ValidationError::new("Case group ID and name cannot be empty");
        error.add_param("case_group_id".into(), &case_group.case_group_id);
        error.add_param("case_group_name".into(), &case_group.case_group_name);
        return Err(error);
    }

    if case_group.case_group_caselist.is_empty() {
        let mut error = ValidationError::new("Case group caselist cannot be empty");
        error.add_param("case_group_caselist".into(), &case_group.case_group_caselist.join(","));
        return Err(error);
    }

    // TODO:Check if all the samples in the case group caselist exist in the omics dataset.

    if case_group.control_group_id.is_empty() || case_group.control_group_name.is_empty() {
        let mut error = ValidationError::new("Control group ID and name cannot be empty");
        error.add_param("control_group_id".into(), &case_group.control_group_id);
        error.add_param("control_group_name".into(), &case_group.control_group_name);
        return Err(error);
    }

    if case_group.control_group_caselist.is_empty() {
        let mut error = ValidationError::new("Control group caselist cannot be empty");
        error.add_param("control_group_caselist".into(), &case_group.control_group_caselist.join(","));
        return Err(error);
    }

    // TODO:Check if all the samples in the control group caselist exist in the omics dataset.

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Validate)]
#[validate(schema(function = "validate_case_group", skip_on_field_errors = false))]
pub struct CaseGroup {
    pub case_group_id: String,
    pub case_group_name: String,
    pub case_group_description: String,
    pub case_group_caselist: Vec<String>,
    pub control_group_id: String,
    pub control_group_name: String,
    pub control_group_description: String,
    pub control_group_caselist: Vec<String>,
}

fn validate_omics_dataset(omics_dataset: &OmicsDataset) -> Result<(), ValidationError> {
    if omics_dataset.dataset_id.is_empty() || omics_dataset.dataset_name.is_empty() {
        let mut error = ValidationError::new("Dataset ID and name cannot be empty");
        error.add_param("dataset_id".into(), &omics_dataset.dataset_id);
        error.add_param("dataset_name".into(), &omics_dataset.dataset_name);
        return Err(error);
    }

    if omics_dataset.dataset_type.is_empty() {
        let mut error = ValidationError::new("Dataset type cannot be empty");
        error.add_param("dataset_type".into(), &omics_dataset.dataset_type);
        return Err(error);
    }

    if omics_dataset.dataset_filepath.is_empty() {
        let mut error = ValidationError::new("Dataset filepath cannot be empty");
        error.add_param("dataset_filepath".into(), &omics_dataset.dataset_filepath);
        return Err(error);
    }

    let root_dir = get_omics_dataset_root_dir();
    info!("Root dir: {}", root_dir.display());
    if !root_dir.join(&omics_dataset.dataset_filepath).exists() {
        let mut error = ValidationError::new("Dataset filepath does not exist");
        error.add_param("dataset_filepath".into(), &omics_dataset.dataset_filepath);
        return Err(error);
    }

    if omics_dataset.status.is_empty() {
        let mut error = ValidationError::new("Status cannot be empty");
        error.add_param("status".into(), &omics_dataset.status);
        return Err(error);
    }

    if omics_dataset.tech_type.is_empty() {
        let mut error = ValidationError::new("Tech type cannot be empty");
        error.add_param("tech_type".into(), &omics_dataset.tech_type);
        return Err(error);
    }

    if omics_dataset.num_samples == 0 {
        let mut error = ValidationError::new("Number of samples cannot be 0");
        error.add_param("num_samples".into(), &omics_dataset.num_samples.to_string());
        return Err(error);
    }

    if omics_dataset.caselist.is_empty() {
        let mut error = ValidationError::new("Caselist cannot be empty");
        error.add_param("caselist".into(), &omics_dataset.caselist.join(","));
        return Err(error);
    }

    for case_group in omics_dataset.predefined_groups.iter() {
        if let Err(e) = validate_case_group(case_group) {
            return Err(e);
        }
    }

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Validate)]
#[validate(schema(function = "validate_omics_dataset", skip_on_field_errors = false))]
pub struct OmicsDataset {
    pub dataset_id: String,
    pub dataset_name: String,
    pub dataset_type: String,
    pub dataset_description: String,
    pub dataset_filepath: String,
    pub tech_type: String,
    pub status: String,
    pub datatype: String,
    pub num_samples: u32,
    pub caselist: Vec<String>,
    pub predefined_groups: Vec<CaseGroup>,
}

fn validate_omics_dataset_metadata(
    omics_dataset_metadata: &OmicsDatasetMetadata,
) -> Result<(), ValidationError> {
    if omics_dataset_metadata.id.is_empty() || omics_dataset_metadata.name.is_empty() {
        let mut error = ValidationError::new("Dataset ID and name cannot be empty");
        error.add_param("id".into(), &omics_dataset_metadata.id);
        error.add_param("name".into(), &omics_dataset_metadata.name);
        return Err(error);
    }

    if omics_dataset_metadata.disease_ontology_id.is_empty() {
        let mut error = ValidationError::new("Disease ontology ID cannot be empty");
        error.add_param("disease_ontology_id".into(), &omics_dataset_metadata.disease_ontology_id);
        return Err(error);
    }

    if omics_dataset_metadata.disease_name.is_empty() {
        let mut error = ValidationError::new("Disease name cannot be empty");
        error.add_param("disease_name".into(), &omics_dataset_metadata.disease_name);
        return Err(error);
    }

    if omics_dataset_metadata.categories.is_empty() {
        let mut error = ValidationError::new("Categories cannot be empty");
        error.add_param("categories".into(), &omics_dataset_metadata.categories.join(","));
        return Err(error);
    }

    let regex_pattern = Regex::new(r"^[0-9,]+$").unwrap();
    if !regex_pattern.is_match(&omics_dataset_metadata.pmid) {
        let mut error = ValidationError::new(
            "PMID must be a number or a comma separated list of numbers",
        );
        error.add_param("pmid".into(), &omics_dataset_metadata.pmid);
        return Err(error);
    }

    if omics_dataset_metadata.status.is_empty() {
        let mut error = ValidationError::new("Status cannot be empty");
        error.add_param("status".into(), &omics_dataset_metadata.status);
        return Err(error);
    }

    if omics_dataset_metadata.num_samples == 0 {
        let mut error = ValidationError::new("Number of samples cannot be 0");
        error.add_param("num_samples".into(), &omics_dataset_metadata.num_samples.to_string());
        return Err(error);
    }

    // Check if the url is a valid url
    let regex_pattern = Regex::new(r"^https?://[^\s/$.?#].[^\s,]*$").unwrap();
    for url in omics_dataset_metadata.url.split(',') {
        if !regex_pattern.is_match(url) {
            let mut error = ValidationError::new("URL is not a valid url, it should be a url or a comma separated list of urls.");
            error.add_param("url".into(), &url);
            return Err(error);
        }
    }

    for omics_dataset in omics_dataset_metadata.omics_datasets.iter() {
        info!("Validating omics dataset: {}", omics_dataset.dataset_id);
        if let Err(e) = validate_omics_dataset(omics_dataset) {
            return Err(e);
        }
    }

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Validate)]
#[validate(schema(
    function = "validate_omics_dataset_metadata",
    skip_on_field_errors = false
))]
pub struct OmicsDatasetMetadata {
    pub id: String,
    pub name: String,
    pub description: String,
    pub disease_ontology_id: String,
    pub disease_name: String,
    pub categories: Vec<String>,
    pub citation: String,
    pub pmid: String,
    pub status: String,
    pub num_samples: u32,
    pub url: String,
    pub tags: Vec<String>,
    pub omics_datasets: Vec<OmicsDataset>,
}

impl OmicsDatasetMetadata {
    pub fn load_datasets(root_dir: PathBuf) -> Result<Vec<Self>, Box<dyn std::error::Error>> {
        if !root_dir.exists() {
            return Err("Omics dataset root directory does not exist".into());
        }

        if !root_dir.join("metadata.json").exists() {
            return Err("No metadata.json file found in the root directory, it might not contain any omics datasets.".into());
        }

        let datasets_metadata = serde_json::from_str::<Vec<OmicsDatasetMetadata>>(
            &std::fs::read_to_string(root_dir.join("metadata.json")).unwrap(),
        )
        .unwrap();
        Ok(datasets_metadata)
    }
}

/// Omics Data Point
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OmicsDataFeature {
    pub id: String,
    pub dataset_id: String,
    pub data: HashMap<String, serde_json::Value>, // Key is sample name, value is a dynamic type
}

/// Clinical Data Point
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClinicalData {
    pub participant_id: String,
    pub data: HashMap<String, serde_json::Value>, // Key is the clinical data type, value is the data value
}

/// Data dictionary for the clinical data.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClinicalDataDictionary {
    pub field_title: String,
    pub field_name: String,
    pub field_type: String,
    pub field_choices: Option<HashMap<String, String>>, // Optional mapping for choice fields
    pub field_description: String,
    pub field_notes: Option<String>,
    pub field_group: Option<String>,
}

/// Data dictionary for the omics data.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OmicsDataDictionary {
    pub id: String,
    pub name: String,
    pub label: String,    // Protein, Gene, Metabolite, Compound, etc.
    pub resource: String, // Which file it comes from
    pub description: Option<String>,
    pub synonyms: Option<Vec<String>>, // List of synonyms
    pub xrefs: Option<Vec<String>>,    // Cross-references
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::init_logger;
    use log::{debug, error, LevelFilter};

    #[test]
    fn test_init_omics_dataset_root_dir() {
        let _ = init_logger("biomedgps-test", LevelFilter::Debug);
        let root_dir = PathBuf::from("/Users/jy006/Downloads/BioMedGPS_Datasets");
        match init_omics_dataset_root_dir(&root_dir, true) {
            Ok(_) => {
                info!("Omics dataset root directory initialized successfully");
            }
            Err(e) => {
                error!("Failed to initialize omics dataset root directory: {}", e);
            }
        }

        assert!(get_omics_dataset_root_dir().exists());
        assert!(get_omics_dataset_root_dir().join("metadata.json").exists());
        assert!(get_omics_dataset_root_dir() == root_dir);
    }
}
