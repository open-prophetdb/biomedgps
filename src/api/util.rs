use std::path::Path;

pub fn check_bedgraphs(datadir: &Path, sample_names: &Vec<String>) -> Result<(), Vec<String>> {
    let mut errors = vec![];
    let extensions = vec!["-Forward.bedgraph".to_string(), "-Reverse.bedgraph".to_string()];
    for sample_name in sample_names {
        for extension in &extensions {
            let filename = format!("{}{}", sample_name, extension);
            let path = datadir.join(&filename);

            if !path.exists() {
                errors.push(format!(
                    "Bedgraph file {} does not exist in {}",
                    filename,
                    datadir.display()
                ));
            }
        }
    }

    if errors.len() > 0 {
        return Err(errors);
    }
    Ok(())
}

pub fn check_bigwigs(datadir: &Path, sample_names: &Vec<String>) -> Result<(), Vec<String>> {
    let mut errors = vec![];
    let extensions = vec!["-Forward.bigwig".to_string(), "-Reverse.bigwig".to_string()];
    for sample_name in sample_names {
        for extension in &extensions {
            let filename = format!("{}{}", sample_name, extension);
            let path = datadir.join(&filename);

            if !path.exists() {
                errors.push(format!(
                    "Bigwig file {} does not exist in {}",
                    filename,
                    datadir.display()
                ));
            }
        }
    }

    if errors.len() > 0 {
        return Err(errors);
    }
    Ok(())
}

pub fn check_ref_genomes(datadir: &Path, genomes: &Vec<String>) -> Result<(), Vec<String>> {
    let mut errors = vec![];
    let extensions = vec![
        ".fa.gz".to_string(),
        ".fa.gz.fai".to_string(),
        ".fa.gz.gzi".to_string(),
        ".gff3.gz".to_string(),
        ".gff3.gz.tbi".to_string(),
        ".gff3.gz.ix".to_string(),
        ".gff3.gz.ixx".to_string(),
        // ".gff3.gz_meta.json".to_string(),
    ];
    for genome in genomes {
        for extension in &extensions {
            let genome_filename = format!("{}{}", genome, extension);
            let genome_path = datadir.join(genome).join(&genome_filename);
            if !genome_path.exists() {
                errors.push(format!(
                    "Genome file {} does not exist in {}",
                    &genome_filename,
                    datadir.display()
                ));
            }
        }
    }

    if errors.len() > 0 {
        return Err(errors);
    }
    Ok(())
}
