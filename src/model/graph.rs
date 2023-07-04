use crate::model::core::{Entity, RecordResponse, Relation};
use crate::query::sql_builder::{ComposeQuery, ComposeQueryItem, QueryItem, Value};
use log::error;
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::{error::Error, fmt};

#[derive(Debug)]
pub struct ValidationError {
    details: String,
    data: Vec<String>,
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

// More details on https://colorbrewer2.org/#type=qualitative&scheme=Paired&n=12
// Don't change the order of the colors. It is important to keep the colors consistent.
// In future, we may specify a color for each node label when we can know all the node labels.
const NODE_COLORS: [&str; 12] = [
    "#a6cee3", "#1f78b4", "#b2df8a", "#33a02c", "#fb9a99", "#e31a1c", "#fdbf6f", "#ff7f00",
    "#cab2d6", "#6a3d9a", "#ffff99", "#b15928",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object)]
pub struct NodeKeyShape {
    pub fill: String,
    pub stroke: String,
    pub opacity: f64,
    #[serde(rename = "fillOpacity")]
    pub fill_opacity: f64,
}

impl NodeKeyShape {
    pub fn new(node_label: &str) -> Self {
        let color = Self::match_color(node_label);

        NodeKeyShape {
            fill: color.clone(),
            stroke: color,
            opacity: 0.95,
            fill_opacity: 0.95,
        }
    }

    // We have a set of colors and we want to match a color to a node label in a deterministic way.
    fn match_color(node_label: &str) -> String {
        let mut hasher = DefaultHasher::new();
        node_label.hash(&mut hasher);
        let hash = hasher.finish();
        let index = hash % NODE_COLORS.len() as u64;
        NODE_COLORS[index as usize].to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object)]
pub struct Icon {
    pub r#type: String,
    pub value: String,
    pub fill: String,
    pub size: i32,
    pub color: String,
}

impl Icon {
    pub fn new(node_label: &str) -> Self {
        // Get the first character of the node label and convert it to a uppercase letter.
        // We use this letter as the icon value.
        let first_char = node_label
            .chars()
            .next()
            .unwrap()
            .to_uppercase()
            .to_string();

        Icon {
            r#type: "text".to_string(),
            value: first_char,
            fill: "#000".to_string(),
            size: 15,
            color: "#000".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object)]
pub struct NodeStyle {
    pub label: String,
    pub keyshape: NodeKeyShape,
    pub icon: Icon,
}

impl NodeStyle {
    pub fn new(node_label: &str) -> Self {
        NodeStyle {
            label: node_label.to_string(),
            keyshape: NodeKeyShape::new(node_label),
            icon: Icon::new(node_label),
        }
    }
}

fn convert_null_to_empty_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.or_else(|| Some("".to_string())))
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object)]
pub struct NodeData {
    pub identity: String,
    pub id: String,
    pub label: String,
    pub name: String,
    #[serde(deserialize_with = "convert_null_to_empty_string")]
    pub description: Option<String>,
    pub resource: String,
    // In future, we can add more fields here after we add additional fields for the Entity struct
}

impl NodeData {
    pub fn new(entity: &Entity) -> Self {
        NodeData {
            identity: entity.id.clone(),
            id: entity.id.clone(),
            label: entity.label.clone(),
            name: entity.name.clone(),
            description: entity.description.clone(),
            resource: entity.resource.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object)]
pub struct Node {
    // Please follow the Graphin format to define the fields. more details on https://graphin.antv.vision/graphin/render/data
    #[serde(rename = "comboId")]
    pub combo_id: Option<String>,
    pub id: String,
    pub label: String,
    pub nlabel: String,
    pub degree: Option<i32>, // Map degree to node size
    pub style: NodeStyle,
    pub category: String, // node or edge
    pub cluster: Option<String>,
    pub r#type: String, // "graphin-circle"
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub data: NodeData,
}

impl Node {
    pub fn new(entity: &Entity) -> Self {
        let identity = Self::format_id(&entity.label, &entity.id);
        Node {
            // We don't use the combo feature in the current stage, so we set the combo_id to None. It's just compatible with the Graphin format.
            combo_id: None,
            // Different labels can have the same id, so we need to add the label to the id
            id: identity.clone(),
            label: identity,
            nlabel: entity.label.clone(),
            degree: None,
            style: NodeStyle::new(&entity.label),
            category: "node".to_string(),
            // In the current stage, we can use the label as the cluster to group the nodes. In future, maybe we can find other better ways to group the nodes. In that case, we can use the update_cluster method to update the cluster information.
            cluster: Some(entity.label.clone()),
            r#type: "graphin-circle".to_string(),
            x: None,
            y: None,
            data: NodeData::new(entity),
        }
    }

    pub fn parse_id(id: &str) -> (String, String) {
        let parts: Vec<&str> = id.split('-').collect();
        (parts[0].to_string(), parts[1].to_string())
    }

    pub fn format_id(label: &str, id: &str) -> String {
        format!("{}-{}", label, id)
    }

    // Update the node position
    // We will use the tsne coordinates to update the node position, so we need to set the method to update the node position
    pub fn update_position(&mut self, x: f64, y: f64) {
        self.x = Some(x);
        self.y = Some(y);
    }

    // Update the node degree
    // TODO: We need to find a value as the degree of the node
    pub fn update_degree(&mut self, degree: i32) {
        self.degree = Some(degree);
    }

    // Update the node cluster
    // Some layout algorithms will use the cluster information to group the nodes.
    pub fn update_cluster(&mut self, cluster: String) {
        self.cluster = Some(cluster);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object)]
pub struct EdgeLabel {
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object)]
pub struct EdgeKeyShape {
    #[serde(rename = "lineDash")]
    pub line_dash: [i32; 2],
    pub stroke: String,
    #[serde(rename = "lineWidth")]
    pub line_width: i32,
}

impl EdgeKeyShape {
    // In the current stage, we use the default value for the edge key shape. In future, we can add more fields to the EdgeKeyShape struct to customize the edge key shape.
    pub fn new() -> Self {
        EdgeKeyShape {
            line_dash: [5, 5],
            stroke: "#ccc".to_string(),
            line_width: 2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object)]
pub struct EdgeStyle {
    pub label: EdgeLabel,
    pub keyshape: EdgeKeyShape,
}

impl EdgeStyle {
    pub fn new(relation_type: &str) -> Self {
        EdgeStyle {
            label: EdgeLabel {
                value: relation_type.to_string(),
            },
            keyshape: EdgeKeyShape::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object)]
pub struct EdgeData {
    pub relation_type: String,
    pub source_id: String,
    pub source_type: String,
    pub target_id: String,
    pub target_type: String,
    pub score: f64,
    pub key_sentence: String,
    pub resource: String,
    // In future, we can add more fields here after we add additional fields for the Entity struct
}

impl EdgeData {
    pub fn new(relation: &Relation) -> Self {
        EdgeData {
            relation_type: relation.relation_type.clone(),
            source_id: relation.source_id.clone(),
            source_type: relation.source_type.clone(),
            target_id: relation.target_id.clone(),
            target_type: relation.target_type.clone(),
            score: relation.score.unwrap_or(0.0),
            key_sentence: relation.key_sentence.clone().unwrap_or("".to_string()),
            resource: relation.resource.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object)]
pub struct Edge {
    pub relid: String,
    // The source and target fields are the id of the node, not the label or entity id of the node. It must be the same as the id field of the Node struct. Otherwise, the edge will not be connected to the node.
    pub source: String,
    pub category: String,
    pub target: String,
    pub reltype: String,
    pub style: EdgeStyle,
    pub data: EdgeData,
}

impl Edge {
    pub fn new(relation: &Relation) -> Self {
        let relid = format!(
            "{}-{}-{}",
            relation.source_id, relation.relation_type, relation.target_id
        );
        Edge {
            relid: relid.clone(),
            source: Node::format_id(&relation.source_type, &relation.source_id),
            category: "edge".to_string(),
            target: Node::format_id(&relation.target_type, &relation.target_id),
            reltype: relation.relation_type.clone(),
            style: EdgeStyle::new(&relation.relation_type),
            data: EdgeData::new(relation),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object)]
pub struct Graph {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

impl Graph {
    pub fn new() -> Self {
        Graph {
            nodes: vec![],
            edges: vec![],
        }
    }

    // Get the nodes in the graph
    pub fn get_nodes(&mut self) -> &Vec<Node> {
        // Dedup the nodes
        self.nodes.sort_by(|a, b| a.id.cmp(&b.id));
        self.nodes.dedup_by(|a, b| a.id == b.id);
        &self.nodes
    }

    // Get the edges in the graph and check if the related nodes are in the graph
    // We cannot get the missed nodes in the get_nodes function, so we return the missed nodes here instead of fetching the missed nodes in the get_nodes function.
    pub fn get_edges(&mut self) -> Result<&Vec<Edge>, ValidationError> {
        // Dedup the edges
        self.edges.sort_by(|a, b| a.relid.cmp(&b.relid));
        self.edges.dedup_by(|a, b| a.relid == b.relid);
        // Ensure the related nodes are in the graph
        let mut node_ids: Vec<String> = vec![];
        for edge in &self.edges {
            node_ids.push(edge.source.clone());
            node_ids.push(edge.target.clone());
        }
        node_ids.sort();
        node_ids.dedup();

        let all_node_ids = self
            .nodes
            .iter()
            .map(|node| &node.id)
            .collect::<Vec<&String>>();
        let missed_node_ids = node_ids
            .iter()
            .filter(|node_id| !all_node_ids.contains(node_id))
            .collect::<Vec<&String>>();
        let missed_node_ids = if missed_node_ids.len() > 0 {
            // TODO: we need to handle the error here
            missed_node_ids
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<String>>()
        } else {
            vec![]
        };

        if missed_node_ids.len() > 0 {
            Err(ValidationError::new(
                "The related nodes of the edges are not in the graph",
                missed_node_ids,
            ))
        } else {
            Ok(&self.edges)
        }
    }

    // Add a node to the graph
    // TODO: we need to check if the node already exists in the graph?
    fn add_node(&mut self, node: Node) {
        self.nodes.push(node);
    }

    // Add an edge to the graph
    // TODO: we need to check if the edge and the related nodes already exists in the graph?
    fn add_edge(&mut self, edge: Edge) {
        self.edges.push(edge);
    }

    pub async fn fetch_nodes_from_db(&mut self, pool: &sqlx::PgPool, node_ids: Vec<&str>) -> Result<(), anyhow::Error> {
        let page = None;
        let page_size = None;
        let node_id_arr = Value::ArrayString(node_ids.iter().map(|id| id.to_string()).collect());
        let query_item = QueryItem::new("id".to_string(), node_id_arr, "in".to_string());
        let query = Some(ComposeQuery::QueryItem(query_item));
        match RecordResponse::<Entity>::get_records(
            pool,
            "biomedgps_entity",
            &query,
            page,
            page_size,
            Some("id ASC"),
        )
        .await
        {
            Ok(records) => {
                for record in records.records {
                    let node = Node::new(&record);
                    self.add_node(node);
                };

                Ok(())
            }
            Err(e) => {
                Err(e)
            }
        }
    }

    fn parse_composed_node_ids(composed_node_id: &str) -> Result<(String, String), ValidationError> {
        let node_ids: Vec<&str> = composed_node_id.split("::").collect();
        if node_ids.len() == 2 {
            let node_type = node_ids[0].to_string();
            let node_id = node_ids[1].to_string();
            Ok((node_type, node_id))
        } else {
            Err(ValidationError::new(&format!("The composed node id is not valid: {}", composed_node_id), vec![]))
        }
    }

    fn gen_relation_query_from_node_ids(node_ids: Vec<&str>) -> ComposeQuery {
        let parsed_node_ids = node_ids
            .iter()
            .map(|node_id| {
                match Graph::parse_composed_node_ids(node_id) {
                    Ok((node_type, node_id)) => Some((node_type, node_id)),
                    Err(e) => {
                        None
                    }
                }
            })
            .collect::<Vec<Option<(String, String)>>>();

        // Filter out the invalid node ids
        let parsed_node_ids = parsed_node_ids
            .iter()
            .filter(|node_id| node_id.is_some())
            .map(|node_id| node_id.clone().unwrap())
            .collect::<Vec<(String, String)>>();

        // Group the node ids by node type
        let mut node_ids_by_type: HashMap<String, Vec<String>> = HashMap::new();
        for (node_type, node_id) in parsed_node_ids {
            if node_ids_by_type.contains_key(&node_type) {
                let node_ids = node_ids_by_type.get_mut(&node_type).unwrap();
                node_ids.push(node_id);
            } else {
                node_ids_by_type.insert(node_type, vec![node_id]);
            }
        }

        // Generate query item for each node type
        let mut query_items: Vec<ComposeQueryItem> = vec![];
        for (node_type, node_ids) in node_ids_by_type {
            let source_node_type = Value::String(node_type.clone());
            let target_node_type = Value::String(node_type.clone());

            let node_id_arr =
                Value::ArrayString(node_ids.iter().map(|id| id.to_string()).collect());

            let source_query_item1 = ComposeQuery::QueryItem(QueryItem::new(
                "source_id".to_string(),
                node_id_arr.clone(),
                "in".to_string(),
            ));
            let source_query_item2 = ComposeQuery::QueryItem(QueryItem::new(
                "source_type".to_string(),
                source_node_type,
                "=".to_string(),
            ));

            let mut source_query = ComposeQueryItem::new("and");
            source_query.add_item(source_query_item1);
            source_query.add_item(source_query_item2);

            let target_query_item1 = ComposeQuery::QueryItem(QueryItem::new(
                "target_id".to_string(),
                node_id_arr,
                "in".to_string(),
            ));
            let target_query_item2 = ComposeQuery::QueryItem(QueryItem::new(
                "target_type".to_string(),
                target_node_type,
                "=".to_string(),
            ));

            let mut target_query = ComposeQueryItem::new("and");
            target_query.add_item(target_query_item1);
            target_query.add_item(target_query_item2);

            let mut query_item = ComposeQueryItem::new("and");
            query_item.add_item(ComposeQuery::ComposeQueryItem(source_query));
            query_item.add_item(ComposeQuery::ComposeQueryItem(target_query));

            query_items.push(query_item);
        }

        let mut query = ComposeQueryItem::new("or");
        for query_item in query_items {
            query.add_item(ComposeQuery::ComposeQueryItem(query_item));
        }

        ComposeQuery::ComposeQueryItem(query)
    }

    pub async fn auto_connect_nodes(&mut self, pool: &sqlx::PgPool, node_ids: Vec<&str>) -> Result<&Self, anyhow::Error> {
        let page = None;
        let page_size = None;

        let query = Self::gen_relation_query_from_node_ids(node_ids);

        match RecordResponse::<Relation>::get_records(
            pool,
            "biomedgps_relation",
            &Some(query),
            page,
            page_size,
            Some("id ASC"),
        )
        .await
        {
            Ok(records) => {
                for record in records.records {
                    let edge = Edge::new(&record);
                    self.add_edge(edge);
                }

                Ok(self)
            }
            Err(e) => {
                error!("Error in auto_connect_nodes: {}", e);
                Err(e)
            }
        }
    }

    // Fetch the linked nodes with some relation types or other conditions
    pub async fn fetch_linked_nodes() {}

    // Fetch the linked nodes within n steps with some relation types or other conditions
    pub async fn fetch_linked_nodes_within_steps() {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_composed_node_ids() {
        let composed_node_id = "Gene::ENTREZ001";
        let (node_type, node_id) = Graph::parse_composed_node_ids(composed_node_id).unwrap();
        assert_eq!(node_type, "Gene");
        assert_eq!(node_id, "ENTREZ001");

        let composed_node_id = "ENTREZ001";
        match Graph::parse_composed_node_ids(composed_node_id) {
            Ok(_) => assert!(false),
            Err(e) => assert_eq!(e.details, "The composed node id is not valid: ENTREZ001"),
        }
    }

    #[test]
    fn test_gen_relation_query_from_node_ids() {
        let node_ids = vec!["Gene::ENTREZ001", "Gene::ENTREZ002", "Gene::ENTREZ003"];
        let query = Graph::gen_relation_query_from_node_ids(node_ids);
        let query_str = match query {
            ComposeQuery::ComposeQueryItem(query_item) => query_item.format(),
            _ => "".to_string(),
        };

        assert_eq!(
            query_str, 
            "((source_id in ('ENTREZ001','ENTREZ002','ENTREZ003') and source_type = 'Gene') and (target_id in ('ENTREZ001','ENTREZ002','ENTREZ003') and target_type = 'Gene'))".to_string()
        );

        let node_ids = vec!["ENTREZ001", "Gene::ENTREZ002", "Gene::ENTREZ003", "Disease::DOID001"];
        let query = Graph::gen_relation_query_from_node_ids(node_ids);
        let query_str = match query {
            ComposeQuery::ComposeQueryItem(query_item) => query_item.format(),
            _ => "".to_string(),
        };

        assert_eq!(query_str, "((source_id in ('DOID001') and source_type = 'Disease') and (target_id in ('DOID001') and target_type = 'Disease')) or ((source_id in ('ENTREZ002','ENTREZ003') and source_type = 'Gene') and (target_id in ('ENTREZ002','ENTREZ003') and target_type = 'Gene'))".to_string());
    }
}
