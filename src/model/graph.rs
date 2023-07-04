//! Graph module is used to define the graph data structure and its related functions. You can use it to fetch the graph data from the postgresql database or neo4j graph database and convert it to the graph data structure which can be used by the frontend.
//!

use crate::model::core::{Entity, Relation};
use lazy_static::lazy_static;
use log::{debug, error};
use poem_openapi::Object;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::{error::Error, fmt};

lazy_static! {
    static ref COMPOSED_ENTITY_REGEX: Regex =
        Regex::new(r"^[A-Za-z]+::[A-Za-z0-9\-]+:[a-z0-9A-Z\.\-_]+$").unwrap();
}

/// Custom Error type for the graph module
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

/// A color map for the node labels.
/// More details on https://colorbrewer2.org/#type=qualitative&scheme=Paired&n=12
/// Don't change the order of the colors. It is important to keep the colors consistent.
/// In future, we may specify a color for each node label when we can know all the node labels.
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

    /// Get the edges in the graph and check if the related nodes are in the graph if the strict_mode is true. It will return the missed nodes here instead of fetching the missed nodes in the get_nodes function.
    ///
    /// # Arguments
    pub fn get_edges(&mut self, strict_mode: Option<bool>) -> Result<&Vec<Edge>, ValidationError> {
        // Dedup the edges
        self.edges.sort_by(|a, b| a.relid.cmp(&b.relid));
        self.edges.dedup_by(|a, b| a.relid == b.relid);

        if strict_mode.is_some() {
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
        } else {
            Ok(&self.edges)
        }
    }

    // Add a node to the graph
    // TODO: we need to check if the node already exists in the graph?
    fn add_node(&mut self, node: Node) {
        self.nodes.push(node);
    }

    /// Add an edge to the graph, we will check if the edge and the related nodes already exists in the get_edges function
    ///
    /// # Arguments
    ///
    /// * `edge` - An edge struct.
    ///
    /// # Example
    ///
    /// ```
    /// use biomedgps::model::graph::{Graph, Edge};
    /// use biomedgps::model::core::Relation;
    ///
    /// let relation = Relation {
    ///     id: 1,
    ///     relation_type: "TREATS".to_string(),
    ///     source_id: "MESH:D0001".to_string(),
    ///     source_type: "Compound".to_string(),
    ///     target_id: "MESH:D0002".to_string(),
    ///     target_type: "Disease".to_string(),
    ///     score: Some(0.9),
    ///     key_sentence: Some("The compound treats the disease".to_string()),
    ///     resource: "CORD19".to_string(),
    /// };
    ///
    /// let mut graph = Graph::new();
    /// let edge = Edge::new(&relation);
    /// graph.add_edge(edge);
    ///
    /// let edges = graph.get_edges(None).unwrap();
    /// assert_eq!(edges.len(), 1);
    /// ```
    ///
    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.push(edge);
    }

    /// Generate a query string to get the nodes from the database
    ///
    /// # Arguments
    ///
    /// * `node_ids` - A vector of node ids
    ///
    /// # Returns
    ///
    /// A query string
    ///
    /// # Example
    ///
    /// ```
    /// use regex::Regex;
    /// use biomedgps::model::graph::Graph;
    ///
    /// let node_ids = vec!["Compound::MESH:D0001", "Compound::MESH:D0002"];
    /// let query = Graph::gen_entity_query_from_node_ids(&node_ids);
    /// let re = Regex::new(r"\s+").unwrap();
    /// let query = re.replace_all(&query, " ");
    /// let expected_query = "SELECT * FROM biomedgps_entity WHERE COALESCE(label, '') || '::' || COALESCE(id, '') in ('Compound::MESH:D0001', 'Compound::MESH:D0002');";
    /// assert_eq!(query, expected_query);
    /// ```
    pub fn gen_entity_query_from_node_ids(node_ids: &Vec<&str>) -> String {
        // SELECT *
        // FROM (SELECT *, COALESCE(label, '') || '::' || COALESCE(id, '') AS full_name FROM biomedgps_entity) as T
        // WHERE full_name in ('Compound::MESH:002', 'Compound::MESH:D0001');

        debug!("Raw node_ids: {:?}", node_ids);

        // Remove invalid node ids
        let filtered_node_ids: Vec<&str> = node_ids
            .iter()
            .filter(|node_id| COMPOSED_ENTITY_REGEX.is_match(node_id))
            .map(|&node_id| node_id)
            .collect();

        debug!("Filtered node_ids: {:?}", node_ids);
        debug!(
            "There are {} invalid node ids.",
            node_ids.len() - filtered_node_ids.len()
        );

        if filtered_node_ids.len() == 0 {
            return "".to_string();
        } else {
            let query_str = format!(
                "SELECT * FROM biomedgps_entity WHERE COALESCE(label, '') || '::' || COALESCE(id, '') in ('{}');",
                filtered_node_ids.join("', '")
            );

            query_str
        }
    }

    /// Fetch the nodes from the database
    ///
    /// # Arguments
    ///
    /// * `pool` - The database connection pool
    /// * `node_ids` - The node ids, which are composed of node type and node id. For example, "Compound::MESH:D0001"
    ///
    /// # Returns
    ///
    /// * `Result<&Self, anyhow::Error>` - The result of fetching the nodes from the database
    ///
    /// # Examples
    ///
    /// ```
    /// use sqlx::postgres::PgPool;
    /// use biomedgps::model::graph::Graph;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let database_url = "postgres://postgres:password@localhost:5432/test_biomedgps";
    ///     let pool = PgPool::connect(database_url).await.unwrap();
    ///     let mut graph = Graph::new();
    ///     let node_ids = vec!["Compound::MESH:D0001", "Compound::MESH:D0002"];
    ///
    ///     assert!(graph.fetch_nodes_from_db(&pool, &node_ids).await.is_ok());
    /// }
    /// ```
    ///
    pub async fn fetch_nodes_from_db(
        &mut self,
        pool: &sqlx::PgPool,
        node_ids: &Vec<&str>,
    ) -> Result<&Self, anyhow::Error> {
        let query_str = Self::gen_entity_query_from_node_ids(node_ids);

        debug!("query_str: {}", query_str);

        match sqlx::query_as::<_, Entity>(query_str.as_str())
            .fetch_all(pool)
            .await
        {
            Ok(records) => {
                for record in records {
                    let node = Node::new(&record);
                    self.add_node(node);
                }

                Ok(self)
            }
            Err(e) => {
                error!("Error in auto_connect_nodes: {}", e);
                Err(e.into())
            }
        }
    }

    /// Parse the composed node id to get the node type and node id
    ///
    /// # Example
    ///
    /// ```
    /// use biomedgps::model::graph::Graph;
    ///
    /// let composed_node_id = "Compound::MESH:D0001";
    /// let (node_type, node_id) = Graph::parse_composed_node_ids(composed_node_id).unwrap();
    /// assert_eq!(node_type, "Compound");
    /// assert_eq!(node_id, "MESH:D0001");
    /// ```
    ///
    /// # Errors
    ///
    /// If the composed node id is not valid, it will return an error.
    ///
    /// ```
    /// use biomedgps::model::graph::Graph;
    ///
    /// let composed_node_id = "Compound::MESH:D0001::";
    /// let result = Graph::parse_composed_node_ids(composed_node_id);
    /// assert!(result.is_err());
    /// ```
    ///
    /// # Arguments
    ///
    /// * `composed_node_id` - The composed node id, like `Compound::MESH:D0001`
    ///
    /// # Returns
    ///
    /// * `Ok((node_type, node_id))` - The node type and node id
    /// * `Err(ValidationError)` - The error message and the invalid node ids
    ///
    pub fn parse_composed_node_ids(
        composed_node_id: &str,
    ) -> Result<(String, String), ValidationError> {
        let node_ids: Vec<&str> = composed_node_id.split("::").collect();
        if node_ids.len() == 2 {
            let node_type = node_ids[0].to_string();
            let node_id = node_ids[1].to_string();
            Ok((node_type, node_id))
        } else {
            Err(ValidationError::new(
                &format!("The composed node id is not valid: {}", composed_node_id),
                vec![composed_node_id.to_string()],
            ))
        }
    }

    /// Generate the query string to fetch the relations from the database
    /// The query string is like:
    /// SELECT *
    /// FROM biomedgps_relation)
    /// WHERE COALESCE(source_type, '') || '::' || COALESCE(source_id, '') in ('Compound::MESH:D001', 'Compound::MESH:D002');
    ///
    /// # Examples:
    ///
    /// ```
    /// use regex::Regex;
    /// use biomedgps::model::graph::Graph;
    ///
    /// let node_ids = vec!["Compound::MESH:D001", "Compound::MESH:D002"];
    /// let query_str = Graph::gen_relation_query_from_node_ids(&node_ids);
    /// let re = Regex::new(r"\s+").unwrap();
    /// let query_str = re.replace_all(query_str.as_str(), " ");
    /// assert_eq!(query_str, "SELECT * FROM biomedgps_relation WHERE COALESCE(source_type, '') || '::' || COALESCE(source_id, '') in ('Compound::MESH:D001', 'Compound::MESH:D002') AND COALESCE(target_type, '') || '::' || COALESCE(target_id, '') in ('Compound::MESH:D001', 'Compound::MESH:D002');");
    /// ```
    ///  
    /// # Arguments
    ///
    /// * `node_ids` - a list of composed node ids, such as ['Compound::MESH:D001', 'Compound::MESH:D002']
    ///
    /// # Returns
    ///
    /// Returns a query string.
    ///
    pub fn gen_relation_query_from_node_ids(node_ids: &Vec<&str>) -> String {
        debug!("Raw node_ids: {:?}", node_ids);

        // Remove invalid node ids
        let filtered_node_ids: Vec<&str> = node_ids
            .iter()
            .filter(|node_id| COMPOSED_ENTITY_REGEX.is_match(node_id))
            .map(|&node_id| node_id)
            .collect();

        debug!("Filtered node_ids: {:?}", node_ids);
        debug!(
            "There are {} invalid node ids.",
            node_ids.len() - filtered_node_ids.len()
        );

        if filtered_node_ids.len() == 0 {
            return "".to_string();
        } else {
            let query_str = format!(
                "SELECT * 
                 FROM biomedgps_relation
                 WHERE COALESCE(source_type, '') || '::' || COALESCE(source_id, '') in ('{}') AND 
                       COALESCE(target_type, '') || '::' || COALESCE(target_id, '') in ('{}');",
                filtered_node_ids.join("', '"),
                filtered_node_ids.join("', '"),
            );

            query_str
        }
    }

    /// Try to connect the nodes in the graph and return the edges and nodes that are in the graph.
    ///
    /// # Examples
    ///
    /// ```
    /// use biomedgps::model::graph::Graph;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut graph = Graph::new();
    ///
    ///     // Please use your own database url
    ///     let database_url = "postgres://postgres:password@localhost:5432/test_biomedgps";
    ///     let pool = sqlx::PgPool::connect(&database_url).await.unwrap();
    ///
    ///     let node_ids = vec![
    ///         "Chemical::MESH:C000601183",
    ///         "Metabolite::HMDB:HMDB0108363",
    ///         "Gene::ENTREZ:108715297",
    ///     ];
    ///
    ///     graph.auto_connect_nodes(&pool, &node_ids).await.unwrap();
    ///
    ///     println!("graph: {:?}", graph);
    ///     assert_eq!(graph.get_nodes().len(), 3);
    ///     assert_eq!(graph.get_edges(None).unwrap().len(), 3);
    /// }
    /// ```
    ///
    /// # Arguments
    ///
    /// * `pool` - The database connection pool
    /// * `node_ids` - The node ids, like `["Compound::MESH:D0001", "Compound::MESH:D0002"]`
    ///
    /// # Returns
    ///
    /// * `Ok(&Self)` - The graph
    /// * `Err(anyhow::Error)` - The error message
    ///
    pub async fn auto_connect_nodes(
        &mut self,
        pool: &sqlx::PgPool,
        node_ids: &Vec<&str>,
    ) -> Result<&Self, anyhow::Error> {
        let query_str = Self::gen_relation_query_from_node_ids(node_ids);

        debug!("query_str: {}", query_str);

        let mut error_msg = "".to_string();
        match sqlx::query_as::<_, Relation>(query_str.as_str())
            .fetch_all(pool)
            .await
        {
            Ok(records) => {
                for record in records {
                    let edge = Edge::new(&record);
                    self.add_edge(edge);
                }
            }
            Err(e) => {
                error_msg = format!("Error in auto_connect_nodes: {}", e);
            }
        };

        match self.fetch_nodes_from_db(pool, node_ids).await {
            Ok(_) => {}
            Err(e) => {
                error_msg = format!(
                    "{}\n{}",
                    error_msg,
                    format!("Error in fetch_nodes_from_db: {}", e)
                );
            }
        };

        if error_msg.len() > 0 {
            Err(anyhow::Error::msg(error_msg))
        } else {
            Ok(self)
        }
    }

    // Fetch the linked nodes with some relation types or other conditions
    pub async fn fetch_linked_nodes() {}

    // Fetch the linked nodes within n steps with some relation types or other conditions
    pub async fn fetch_linked_nodes_within_steps() {}
}

#[cfg(test)]
mod tests {
    extern crate log;
    extern crate stderrlog;
    use super::*;
    use crate::{import_data, run_migrations, init_log};
    use regex::Regex;

    // Setup the test database
    async fn setup_test_db() -> sqlx::PgPool {
        // Get the database url from the environment variable
        let database_url = match std::env::var("DATABASE_URL") {
            Ok(v) => v,
            Err(_) => {
                println!("{}", "DATABASE_URL is not set.");
                std::process::exit(1);
            }
        };
        let pool = sqlx::PgPool::connect(&database_url).await.unwrap();

        // Run the migrations
        run_migrations(&database_url).await.unwrap();

        // Import data file in examples folder into the database
        let entity_data_file = "examples/entity.tsv";

        import_data(&database_url, entity_data_file, "entity", true, true).await;

        let relation_data_file = "examples/relation.tsv";

        import_data(&database_url, relation_data_file, "relation", true, true).await;

        return pool;
    }

    #[test]
    fn test_parse_composed_node_ids() {
        init_log();
        let composed_node_id = "Gene::ENTREZ:1";
        let (node_type, node_id) = Graph::parse_composed_node_ids(composed_node_id).unwrap();
        assert_eq!(node_type, "Gene");
        assert_eq!(node_id, "ENTREZ:1");

        let composed_node_id = "ENTREZ:1";
        match Graph::parse_composed_node_ids(composed_node_id) {
            Ok(_) => assert!(false),
            Err(e) => assert_eq!(e.details, "The composed node id is not valid: ENTREZ:1"),
        };
    }

    #[test]
    fn test_gen_entity_query_from_node_ids() {
        init_log();
        let node_ids = vec!["Gene::ENTREZ:1", "Gene::ENTREZ:2", "Gene::ENTREZ:3"];
        let query_str = Graph::gen_entity_query_from_node_ids(&node_ids);

        // Remove the newlines and unnecessary spaces by using regex
        let re = Regex::new(r"\s+").unwrap();
        let query_str = re.replace_all(query_str.as_str(), " ");

        assert_eq!(query_str, "SELECT * FROM biomedgps_entity WHERE COALESCE(label, '') || '::' || COALESCE(id, '') in ('Gene::ENTREZ:1', 'Gene::ENTREZ:2', 'Gene::ENTREZ:3');")
    }

    #[test]
    fn test_gen_relation_query_from_node_ids() {
        init_log();
        let node_ids = vec!["Gene::ENTREZ:1", "Gene::ENTREZ:2", "Gene::ENTREZ:3"];
        let query_str = Graph::gen_relation_query_from_node_ids(&node_ids);

        // Remove the newlines and unnecessary spaces by using regex
        let re = Regex::new(r"\s+").unwrap();
        let query_str = re.replace_all(query_str.as_str(), " ");

        assert_eq!(query_str, "SELECT * FROM biomedgps_relation WHERE COALESCE(source_type, '') || '::' || COALESCE(source_id, '') in ('Gene::ENTREZ:1', 'Gene::ENTREZ:2', 'Gene::ENTREZ:3') AND COALESCE(target_type, '') || '::' || COALESCE(target_id, '') in ('Gene::ENTREZ:1', 'Gene::ENTREZ:2', 'Gene::ENTREZ:3');".to_string());

        let invalid_node_ids = vec!["Gene:ENTREZ::001", "Gene:ENTREZ::002", "Gene::ENTREZ::003"];
        let query_str = Graph::gen_relation_query_from_node_ids(&invalid_node_ids);

        // Remove the newlines and unnecessary spaces by using regex
        let re = Regex::new(r"\s+").unwrap();
        let query_str = re.replace_all(query_str.as_str(), " ");

        assert_eq!(query_str, "".to_string());
    }

    #[tokio::test]
    async fn test_auto_connect_nodes() {
        init_log();
        let mut graph = Graph::new();

        let pool = setup_test_db().await;

        let node_ids = vec![
            "Chemical::MESH:C000601183",
            "Metabolite::HMDB:HMDB0108363",
            "Gene::ENTREZ:108715297",
        ];

        graph.auto_connect_nodes(&pool, &node_ids).await.unwrap();

        println!("graph: {:?}", graph);
        assert_eq!(graph.nodes.len(), 3);
        assert_eq!(graph.edges.len(), 3);
    }
}
