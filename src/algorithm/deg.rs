//! Differentially Expressed Genes (DEG) analysis and visualization. This module provides functions to perform DEG analysis and visualize the results. Such as Volcano plot, Clustering, and so on.
use super::theme::npg::PLOTLY_NATURE;
use plotly::{
    common::{Marker, Mode},
    layout::Axis,
    Layout, Plot, Scatter,
};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::io::Write;

pub fn plot2json(plot: Plot) -> String {
    let json = plot.to_json();
    json
}

/// Differentially Expressed Genes (DEG) data structure. We store the p-value, fold change, gene symbol, and entrez id for each gene.
/// The p-value is the probability of observing a fold change as extreme as the one observed, assuming that the null hypothesis is true.
/// The fold change is the ratio of the expression level in the treatment group to the control group.
/// The gene symbol is the human-readable name of the gene.
/// The entrez id is the unique identifier for the gene in the NCBI Entrez Gene database.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct DEG {
    pvalue: f64,
    fold_change: f64,
    entrez_id: String, // Entrez Gene ID, Need to keep it same as in the knowledge graph. Such as ENTREZ:1234
    gene_symbol: String,
}

impl DEG {
    pub fn new(pvalue: f64, fold_change: f64, entrez_id: String, gene_symbol: String) -> DEG {
        DEG {
            pvalue,
            fold_change,
            entrez_id,
            gene_symbol,
        }
    }

    /// Read DEG data from a JSON file. We cache user uploaded DEG data in a JSON file.
    pub fn read_json(file: &str) -> Vec<DEG> {
        let file = File::open(file).expect("File not found");
        let reader = BufReader::new(file);
        let degs: Vec<DEG> = serde_json::from_reader(reader).expect("Error parsing JSON");
        degs
    }

    pub fn volcano_plot(degs: Vec<DEG>, metadata: Metadata) -> Result<Plot, Box<dyn Error>> {
        if degs.is_empty() {
            return Err("No DEG data found".into());
        }

        let upgrated_color = "#e44c37"; // Red
        let downregulated_color = "#59bccb"; // Blue
        let no_change_color = "gray";
        let mut upregulated = Vec::new();
        let mut downregulated = Vec::new();
        let mut no_change = Vec::new();
        let mut upregulated_neg_log10_pvalues = Vec::new();
        let mut downregulated_neg_log10_pvalues = Vec::new();
        let mut no_change_neg_log10_pvalues = Vec::new();
        let mut upregulated_tooltips = Vec::new();
        let mut downregulated_tooltips = Vec::new();
        let mut no_change_tooltips = Vec::new();

        for deg in degs {
            let log2_fold_change = if metadata.is_log2_fold_change {
                deg.fold_change
            } else {
                deg.fold_change.log2()
            };
            let neg_log10_pvalue = -1.0 * deg.pvalue.log10();
            let color = if deg.pvalue < metadata.pvalue_threshold {
                if log2_fold_change > metadata.log2fold_change_threshold {
                    upgrated_color
                } else if log2_fold_change < -metadata.log2fold_change_threshold {
                    downregulated_color
                } else {
                    no_change_color
                }
            } else {
                no_change_color
            };

            if color == upgrated_color {
                upregulated.push((log2_fold_change, neg_log10_pvalue));
                upregulated_tooltips.push(format!(
                    "Gene: {}<br>Entrez ID: {}<br>Log2 Fold Change: {:.3}<br>p-value: {:.3}",
                    deg.gene_symbol, deg.entrez_id, log2_fold_change, deg.pvalue
                ));
                upregulated_neg_log10_pvalues.push(neg_log10_pvalue);
            } else if color == downregulated_color {
                downregulated.push((log2_fold_change, neg_log10_pvalue));
                downregulated_tooltips.push(format!(
                    "Gene: {}<br>Entrez ID: {}<br>Log2 Fold Change: {:.3}<br>p-value: {:.3}",
                    deg.gene_symbol, deg.entrez_id, log2_fold_change, deg.pvalue
                ));
                downregulated_neg_log10_pvalues.push(neg_log10_pvalue);
            } else {
                no_change.push((log2_fold_change, neg_log10_pvalue));
                no_change_tooltips.push(format!(
                    "Gene: {}<br>Entrez ID: {}<br>Log2 Fold Change: {:.3}<br>p-value: {:.3}",
                    deg.gene_symbol, deg.entrez_id, log2_fold_change, deg.pvalue
                ));
                no_change_neg_log10_pvalues.push(neg_log10_pvalue);
            }
        }

        let mut plot = Plot::new();
        if !upregulated.is_empty() {
            let (x, y): (Vec<_>, Vec<_>) = upregulated.iter().cloned().unzip();
            let scatter = Scatter::new(x, y)
                .mode(Mode::Markers)
                .marker(Marker::new().size(5).color(upgrated_color))
                .name("Upregulated")
                .text_array(upregulated_tooltips);
            plot.add_trace(scatter);
        }

        if !downregulated.is_empty() {
            let (x, y): (Vec<_>, Vec<_>) = downregulated.iter().cloned().unzip();
            let scatter = Scatter::new(x, y)
                .mode(Mode::Markers)
                .marker(Marker::new().size(5).color(downregulated_color))
                .name("Downregulated")
                .text_array(downregulated_tooltips);
            plot.add_trace(scatter);
        }

        if !no_change.is_empty() {
            let (x, y): (Vec<_>, Vec<_>) = no_change.iter().cloned().unzip();
            let scatter = Scatter::new(x, y)
                .mode(Mode::Markers)
                .marker(Marker::new().size(5).color(no_change_color))
                .name("No Diff")
                .text_array(no_change_tooltips);
            plot.add_trace(scatter);
        }

        let template = &*PLOTLY_NATURE;
        let layout = Layout::new()
            .template(template)
            .title("Volcano Plot")
            .x_axis(Axis::new().title("Log2 Fold Change"))
            .y_axis(Axis::new().title("-log10(p-value)"))
            .show_legend(true);
        plot.set_layout(layout);

        Ok(plot)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct Metadata {
    disease_name: String,
    disease_id: String, // Mondo Disease ID, same as in the knowledge graph.
    sample_size: usize,
    sample_type: String, // Blood, Tissue, etc.
    species: u64,        // 9606 for human, 10090 for mouse, 10116 for rat, etc.
    is_log2_fold_change: bool,
    pvalue_threshold: f64,
    log2fold_change_threshold: f64, // Always use log2 fold change.
}

impl Metadata {
    pub fn new(
        disease_name: String,
        disease_id: String,
        sample_size: usize,
        sample_type: String,
        species: u64,
        is_log2_fold_change: bool,
        pvalue_threshold: f64,
        log2fold_change_threshold: f64,
    ) -> Metadata {
        Metadata {
            disease_name,
            disease_id,
            sample_size,
            sample_type,
            species,
            is_log2_fold_change,
            pvalue_threshold,
            log2fold_change_threshold,
        }
    }

    pub fn read_json(file: &str) -> Metadata {
        let file = File::open(file).expect("File not found");
        let reader = BufReader::new(file);
        let metadata: Metadata = serde_json::from_reader(reader).unwrap();
        metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deg() {
        let deg = DEG::new(0.01, 2.0, "ENTREZ:1234".to_string(), "GENE1".to_string());
        assert_eq!(deg.pvalue, 0.01);
    }

    #[test]
    fn test_metadata() {
        let metadata = Metadata::new(
            "Disease".to_string(),
            "MONDO:1234".to_string(),
            100,
            "Blood".to_string(),
            9606,
            false,
            0.05,
            1.0,
        );
        assert_eq!(metadata.disease_name, "Disease");
    }

    fn generate_example_data() -> Vec<DEG> {
        let mut degs = Vec::new();
        for i in 1..=100 {
            let fold_change = (i as f64) * 0.01; // Example fold change
            let pvalue = 0.01 + (i as f64) * 0.001; // Example p-value
            let entrez_id = format!("ENTREZ:{}", 1000 + i);
            let gene_symbol = format!("GENE{}", i);
            degs.push(DEG::new(fold_change, pvalue, entrez_id, gene_symbol));
        }
        degs
    }

    #[test]
    fn test_volcano_plot() {
        let degs = generate_example_data();
        let metadata = Metadata::new(
            "Disease".to_string(),
            "MONDO:1234".to_string(),
            100,
            "Blood".to_string(),
            9606,
            false,
            0.05,
            1.0,
        );
        let plot = DEG::volcano_plot(degs, metadata).unwrap();
        let json = plot2json(plot);
        assert_eq!(json.contains("Volcano Plot"), true);
    }

    #[test]
    fn test_read_json() {
        let degs = DEG::read_json("./data/algorithms/deg/all_genes.json");
        let metadata = Metadata::new(
            "Disease".to_string(),
            "MONDO:1234".to_string(),
            100,
            "Blood".to_string(),
            9606,
            false,
            0.05,
            1.0,
        );
        let plot = DEG::volcano_plot(degs, metadata).unwrap();
        let json = plot2json(plot);

        // Save the plot to a file.
        let file = "./data/algorithms/deg/volcano_plot.json";
        let mut file = File::create(file).expect("File not found");
        file.write_all(json.as_bytes())
            .expect("Unable to write data");
    }
}
