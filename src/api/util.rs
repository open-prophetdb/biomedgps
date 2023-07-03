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
