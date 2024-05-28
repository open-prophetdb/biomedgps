use rustworkx_core::centrality::{
    betweenness_centrality, closeness_centrality, eigenvector_centrality, katz_centrality,
};
use rustworkx_core::petgraph::graph::EdgeReference;
use rustworkx_core::petgraph::graph::NodeIndex;
use rustworkx_core::petgraph::graph::{DiGraph, UnGraph};
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::{from_value, to_value};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Relation {
    reltype: String,
    source_name: String,
    source_type: String,
    source_id: String,
    target_name: String,
    target_type: String,
    target_id: String,
    score: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct CentralityResult {
    entity_id: String,
    entity_type: String,
    entity_name: String,
    betweenness_score: f64,
    degree_score: f64,
    closeness_score: f64,
    eigenvector_score: f64,
    pagerank_score: f64,
}

fn calculate_centralities(relations: Vec<Relation>) -> Vec<CentralityResult> {
    // let mut graph = DiGraph::<String, f64>::new();
    let mut graph = UnGraph::<String, f64>::new_undirected();
    let mut node_indices: HashMap<String, NodeIndex> = HashMap::new();

    for rel in &relations {
        let source_id = &rel.source_id;
        let target_id = &rel.target_id;
        if !node_indices.contains_key(source_id) {
            let node_index = graph.add_node(source_id.clone());
            node_indices.insert(source_id.clone(), node_index);
        }
        if !node_indices.contains_key(target_id) {
            let node_index = graph.add_node(target_id.clone());
            node_indices.insert(target_id.clone(), node_index);
        }
        let source_index = node_indices[source_id];
        let target_index = node_indices[target_id];
        graph.add_edge(source_index, target_index, rel.score);
    }

    let betweenness_scores = betweenness_centrality(&graph, false, true, 100);
    let closeness_scores = closeness_centrality(&graph, false);
    // More details on the parameters for eigenvector_centrality can be found at https://docs.rs/rustworkx-core/latest/rustworkx_core/centrality/fn.eigenvector_centrality.html
    let eigenvector_scores = match eigenvector_centrality(
        &graph,
        |edge: EdgeReference<f64>| -> Result<f64, ()> { Ok(*edge.weight()) },
        None,
        None,
    ) {
        Ok(scores) => scores.unwrap_or(vec![]),
        Err(_) => vec![],
    };

    // let degree_scores = degree_centrality(&graph);
    // let pagerank_scores = pagerank(&graph, 0.85, 0.0001).unwrap();

    let mut results = Vec::new();
    for (node, index) in node_indices.iter() {
        let entity_id = node.clone();
        let entity_info = relations
            .iter()
            .find(|r| &r.source_id == node || &r.target_id == node);

        let (entity_type, entity_name) = match entity_info {
            Some(relation) => {
                if &relation.source_id == node {
                    (relation.source_type.clone(), relation.source_name.clone())
                } else {
                    (relation.target_type.clone(), relation.target_name.clone())
                }
            }
            None => ("Unknown".to_string(), "Unknown".to_string()),
        };
        let betweenness_score = match *betweenness_scores.get(index.index()).unwrap_or(&None) {
            Some(score) => score,
            None => 0.0,
        };
        let closeness_score = match *closeness_scores.get(index.index()).unwrap_or(&None) {
            Some(score) => score,
            None => 0.0,
        };
        let eigenvector_score = *eigenvector_scores.get(index.index()).unwrap_or(&0.0);

        let degree_score = 0.0;
        let pagerank_score = 0.0;

        // let degree_score = *degree_scores.get(index.index()).unwrap_or(&0.0);
        // let pagerank_score = *pagerank_scores.get(index.index()).unwrap_or(&0.0);

        results.push(CentralityResult {
            entity_id,
            entity_type,
            entity_name,
            betweenness_score,
            closeness_score,
            eigenvector_score,
            degree_score,
            pagerank_score,
        });
    }

    results
}

#[wasm_bindgen]
pub fn calculate_centrality(relations: JsValue) -> JsValue {
    let relations: Vec<Relation> = from_value(relations).unwrap();
    let results = calculate_centralities(relations);
    to_value(&results).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_centralities() {
        let relations = vec![
            Relation {
                reltype: "related".to_string(),
                source_name: "A".to_string(),
                source_type: "Person".to_string(),
                source_id: "A".to_string(),
                target_name: "B".to_string(),
                target_type: "Person".to_string(),
                target_id: "B".to_string(),
                score: 0.5,
            },
            Relation {
                reltype: "related".to_string(),
                source_name: "A".to_string(),
                source_type: "Person".to_string(),
                source_id: "A".to_string(),
                target_name: "C".to_string(),
                target_type: "Person".to_string(),
                target_id: "C".to_string(),
                score: 0.5,
            },
            Relation {
                reltype: "related".to_string(),
                source_name: "B".to_string(),
                source_type: "Person".to_string(),
                source_id: "B".to_string(),
                target_name: "C".to_string(),
                target_type: "Person".to_string(),
                target_id: "C".to_string(),
                score: 0.5,
            },
        ];

        let results = calculate_centralities(relations);
        for result in &results {
            println!(
                "entity_id: {}, entity_type: {}, betweenness_score: {}, closeness_score: {}, eigenvector_score: {}, degree_score: {}, pagerank_score: {}",
                result.entity_id,
                result.entity_type,
                result.betweenness_score,
                result.closeness_score,
                result.eigenvector_score,
                result.degree_score,
                result.pagerank_score,
            );
        }

        assert_eq!(results.len(), 3);

        let a_node = results.iter().find(|r| r.entity_id == "A").unwrap();
        assert_eq!(a_node.entity_type, "Person");
        assert_eq!(a_node.betweenness_score, 2.0);
        assert_eq!(a_node.closeness_score, 0.0);
        assert_eq!(a_node.eigenvector_score, 0.0);
        assert_eq!(a_node.degree_score, 0.0);
        assert_eq!(a_node.pagerank_score, 0.0);
    }
}
