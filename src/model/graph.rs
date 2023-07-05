//! Graph module is used to define the graph data structure and its related functions.
//!
//! NOTICE:
//! - The graph data structure is different from the Entity struct. The Entity struct is used to represent the entity data in the database. The graph data structure is used to represent the graph data which can be used by the frontend to render the graph.
//! - The module is used to fetch the graph data from the postgresql database or neo4j graph database and convert it to the graph data structure which can be used by the frontend.
//!

use crate::model::core::{Entity, RecordResponse, Relation};
use crate::query_builder::sql_builder::ComposeQuery;
use lazy_static::lazy_static;
use log::{debug, error};
use poem_openapi::Object;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::vec;
use std::{error::Error, fmt};

lazy_static! {
    static ref COMPOSED_ENTITY_REGEX: Regex =
        Regex::new(r"^[A-Za-z]+::[A-Za-z0-9\-]+:[a-z0-9A-Z\.\-_]+$").unwrap();
}

const COMPOSED_ENTITY_DELIMETER: &str = "::";

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

/// A NodeKeyShape struct for the node rendering.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object)]
pub struct NodeKeyShape {
    pub fill: String,
    pub stroke: String,
    pub opacity: f64,
    #[serde(rename = "fillOpacity")]
    pub fill_opacity: f64,
}

impl NodeKeyShape {
    /// Create a NodeKeyShape according to the node label.
    pub fn new(node_label: &str) -> Self {
        let color = Self::match_color(node_label);

        NodeKeyShape {
            fill: color.clone(),
            stroke: color,
            opacity: 0.95,
            fill_opacity: 0.95,
        }
    }

    /// We have a set of colors and we want to match a color to a node label in a deterministic way.
    fn match_color(node_label: &str) -> String {
        let mut hasher = DefaultHasher::new();
        node_label.hash(&mut hasher);
        let hash = hasher.finish();
        let index = hash % NODE_COLORS.len() as u64;
        NODE_COLORS[index as usize].to_string()
    }
}

/// A icon struct for the node rendering.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object)]
pub struct Icon {
    pub r#type: String,
    pub value: String,
    pub fill: String,
    pub size: i32,
    pub color: String,
}

impl Icon {
    /// Get the first character of the node label and convert it to a uppercase letter.
    /// We use this letter as the icon value.
    pub fn new(node_label: &str) -> Self {
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

/// A style struct for the node rendering.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object)]
pub struct NodeStyle {
    pub label: String,
    pub keyshape: NodeKeyShape,
    pub icon: Icon,
}

impl NodeStyle {
    /// Create a NodeStyle according to the node label.
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

/// A node struct. It is same with Entity struct but for the frontend to render.
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
    /// Create a NodeData from an Entity.
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

/// A node struct for the frontend to render.
///
/// NOTICE: Node - frontend, Entity - backend
///
/// Please follow the Graphin format to define the fields. more details on https://graphin.antv.vision/graphin/render/data
///
/// * `comboId` - We don't use the combo feature in the current stage, so we set the combo_id to None. It's just compatible with the Graphin format.
/// * `id` - The id of the node. It's a combination of the node label and the node id. For example, "Disease::MESH:D0001". It must match the COMPOSED_ENTITY_REGEX regex. Different label can have the same entity id, so we need to add the label to the entity id and make a composed id for uniqueness.
/// * `label` - The label of the node. It is same with the node id. For example, "Disease::MESH:D0001".
/// * `nlabel` - The label of the entity. For example, "Disease".
/// * `degree` - The degree of the node. It is used to determine the node size. For example, 10. In the current stage, we don't use this field.
/// * `style` - The style of the node. It contains the label, keyshape and icon. The label is the node label. The keyshape is the node shape. The icon is the node icon. For example, {"label": "Disease", "keyshape": {"fill": "#a6cee3", "stroke": "#a6cee3", "opacity": 0.95, "fill_opacity": 0.95}, "icon": {"type": "text", "value": "D", "fill": "#000", "size": 15, "color": "#000"}}.
/// * `category` - The category of the node. It must be "node".
/// * `cluster` - In the current stage, we can use the label as the cluster to group the nodes. In future, maybe we can find other better ways to group the nodes. In that case, we can use the update_cluster method to update the cluster information.
/// * `type` - The type of the node. It must be "graphin-circle".
/// * `x` - The x coordinate of the node. It is used to determine the node position. For example, 100. In the currect stage, we use the tsne algorithm to calculate the node position. If you want to set x and y, you need to use the update_position method.
/// * `y` - Same with x.
/// * `data` - The data of the node.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object)]
pub struct Node {
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
    /// Create a new node from an entity for the frontend to render.
    pub fn new(entity: &Entity) -> Self {
        let identity = Self::format_id(&entity.label, &entity.id);
        Node {
            combo_id: None,
            id: identity.clone(),
            label: identity,
            nlabel: entity.label.clone(),
            degree: None,
            style: NodeStyle::new(&entity.label),
            category: "node".to_string(),
            cluster: Some(entity.label.clone()),
            r#type: "graphin-circle".to_string(),
            x: None,
            y: None,
            data: NodeData::new(entity),
        }
    }

    /// Parse the node id to get the label and entity id.
    pub fn parse_id(id: &str) -> (String, String) {
        let parts: Vec<&str> = id.split(COMPOSED_ENTITY_DELIMETER).collect();
        (parts[0].to_string(), parts[1].to_string())
    }

    /// Format the node id, we use the label and entity id to format the node id.
    pub fn format_id(label: &str, entity_id: &str) -> String {
        format!("{}{}{}", label, COMPOSED_ENTITY_DELIMETER, entity_id)
    }

    /// Update the node position
    /// We will use the tsne coordinates to update the node position, so we need to set the method to update the node position
    pub fn update_position(&mut self, x: f64, y: f64) {
        self.x = Some(x);
        self.y = Some(y);
    }

    /// Update the node degree.
    ///
    /// TODO: We need to find a value as the degree of the node
    pub fn update_degree(&mut self, degree: i32) {
        self.degree = Some(degree);
    }

    /// Update the node cluster.
    ///
    /// Some layout algorithms will use the cluster information to group the nodes.
    pub fn update_cluster(&mut self, cluster: String) {
        self.cluster = Some(cluster);
    }
}

/// The EdgeLabel struct is used to store the edge label information. The value will be displayed on the edge.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object)]
pub struct EdgeLabel {
    pub value: String,
}

/// The EdgeKeyShape struct is used to store the edge key shape information.
/// In the current stage, we use the default value for the edge key shape. In future, we can add more fields to the EdgeKeyShape struct to customize the edge key shape.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object)]
pub struct EdgeKeyShape {
    #[serde(rename = "lineDash")]
    pub line_dash: [i32; 2],
    pub stroke: String,
    #[serde(rename = "lineWidth")]
    pub line_width: i32,
}

impl EdgeKeyShape {
    /// Create a new key shape for the edge.
    pub fn new() -> Self {
        EdgeKeyShape {
            line_dash: [5, 5],
            stroke: "#ccc".to_string(),
            line_width: 2,
        }
    }
}

/// The EdgeStyle struct is used to store the edge style information. The frontend will use these information to render the edge.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object)]
pub struct EdgeStyle {
    pub label: EdgeLabel,
    pub keyshape: EdgeKeyShape,
}

impl EdgeStyle {
    /// Create a new style for the edge.
    pub fn new(relation_type: &str) -> Self {
        EdgeStyle {
            label: EdgeLabel {
                value: relation_type.to_string(),
            },
            keyshape: EdgeKeyShape::new(),
        }
    }
}

/// The Edge struct is used to store the edge information. The frontend will use these information.
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
    // In future, we can add more fields here after we add additional fields for the Relation struct
}

impl EdgeData {
    /// Create a new EdgeData struct from a Relation struct
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

/// The edge struct which is compatible with the Graphin format
///
/// * `relid` - The id of the edge. It's the combination of the source id, the relation type and the target id.
/// * `source` - The source and target fields are the id of the node. It must be the same as the id field of the Node struct. Otherwise, the edge will not be connected to the node.
/// * `category` - The category of the edge. It must be "edge".
/// * `target` - Same as the source field.
/// * `reltype` - The relation type of the edge. Such as "Inhibitor::Gene:Gene".
/// * `style` - The style of the edge. It contains the label and the keyshape. More details can be found in the [`EdgeStyle`](struct.EdgeStyle.html) struct.
/// * `data` - The data of the edge. It contains the relation information. Its fields are the same as the [`Relation`](struct.Relation.html) struct.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object)]
pub struct Edge {
    pub relid: String,
    pub source: String,
    pub category: String,
    pub target: String,
    pub reltype: String,
    pub style: EdgeStyle,
    pub data: EdgeData,
}

impl Edge {
    /// Create a new edge. It will convert the [`Relation`](struct.Relation.html) struct to the [`Edge`](struct.Edge.html) struct.
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

/// The graph struct, which contains the nodes and edges
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object)]
pub struct Graph {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

impl Graph {
    /// Create a new graph
    ///
    /// # Returns
    ///
    /// * `Graph` - The new graph
    ///
    pub fn new() -> Self {
        Graph {
            nodes: vec![],
            edges: vec![],
        }
    }

    /// Get the graph from the nodes and edges.
    /// It will dedup the nodes and edges, and check if the related nodes are in the graph if the strict_mode is true.
    ///
    /// # Arguments
    ///
    /// * `strict_mode` - If the strict_mode is true, it will check if the related nodes are in the graph. Otherwise, it will not check it and just return the graph. If you don't care about the nodes, you can set it to false. Otherwise, you should set it to true, catch the error and get the missed nodes from the error by accessing the `error.data`. And then you can fetch the missed nodes by calling the `fetch_nodes_from_db` method of the `Graph` struct.
    ///
    /// # Returns
    ///
    /// * `Result<Graph, ValidationError>` - The graph or the error
    ///
    /// NOTE: If you don't care about the duplicated or missed nodes and edges, you can just call the `graph.to_owned()` method to get the graph.
    pub fn get_graph(&mut self, strict_mode: Option<bool>) -> Result<Graph, ValidationError> {
        match self.get_edges(strict_mode) {
            Ok(_) => Ok(self.to_owned()),
            Err(err) => Err(err),
        }
    }

    /// Get the nodes in the graph
    ///
    /// # Returns
    ///
    /// * `&Vec<Node>` - The nodes in the graph
    ///
    pub fn get_nodes(&mut self) -> &Vec<Node> {
        // Dedup the nodes
        self.nodes.sort_by(|a, b| a.id.cmp(&b.id));
        self.nodes.dedup_by(|a, b| a.id == b.id);
        &self.nodes
    }

    /// Get the edges in the graph and check if the related nodes are in the graph if the strict_mode is true. It will return the missed nodes here instead of fetching the missed nodes in the get_nodes function.
    ///
    /// # Arguments
    ///
    /// * `strict_mode` - If the strict_mode is true, it will check if the related nodes are in the graph. Otherwise, it will not check the related nodes.
    ///
    /// # Returns
    ///
    /// * `Result<&Vec<Edge>, ValidationError>` - If the strict_mode is true, it will return the missed nodes in the graph.
    ///
    pub fn get_edges(&mut self, strict_mode: Option<bool>) -> Result<&Vec<Edge>, ValidationError> {
        // Dedup the edges
        self.edges.sort_by(|a, b| a.relid.cmp(&b.relid));
        self.edges.dedup_by(|a, b| a.relid == b.relid);

        self.nodes = self.get_nodes().to_vec();

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

    /// Get the node ids from the edges, it contains the source and target node ids
    pub fn get_node_ids_from_edges(&self) -> Vec<String> {
        let mut node_ids: Vec<String> = vec![];
        for edge in &self.edges {
            node_ids.push(edge.source.clone());
            node_ids.push(edge.target.clone());
        }
        node_ids.sort();
        node_ids.dedup();
        node_ids
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
                "SELECT * FROM biomedgps_entity WHERE COALESCE(label, '') || '{}' || COALESCE(id, '') in ('{}');",
                COMPOSED_ENTITY_DELIMETER,
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
    async fn fetch_nodes_from_db(
        &self,
        pool: &sqlx::PgPool,
        node_ids: &Vec<&str>,
    ) -> Result<Vec<Node>, anyhow::Error> {
        let query_str = Self::gen_entity_query_from_node_ids(node_ids);

        debug!("query_str: {}", query_str);

        match sqlx::query_as::<_, Entity>(query_str.as_str())
            .fetch_all(pool)
            .await
        {
            Ok(records) => {
                let nodes = records
                    .iter()
                    .map(|record| Node::new(&record))
                    .collect::<Vec<Node>>();
                Ok(nodes)
            }
            Err(e) => {
                error!("Error in fetch_nodes_from_db: {}", e);
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
                 WHERE COALESCE(source_type, '') || '{}' || COALESCE(source_id, '') in ('{}') AND 
                       COALESCE(target_type, '') || '{}' || COALESCE(target_id, '') in ('{}');",
                COMPOSED_ENTITY_DELIMETER,
                filtered_node_ids.join("', '"),
                COMPOSED_ENTITY_DELIMETER,
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
            Ok(nodes) => {
                for node in nodes {
                    self.add_node(node);
                }
            }
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

    /// Fetch the nodes from the database by node ids. It will update the nodes in the graph directly.
    /// 
    /// # Arguments
    /// 
    /// * `pool` - The database connection pool
    /// * `node_ids` - The node ids, like `["Compound::MESH:D0001", "Compound::MESH:D0002"]`
    /// 
    /// # Returns
    /// 
    /// * `Ok(&Self)` - The graph
    /// * `Err(ValidationError)` - The error message
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
    ///     assert!(graph.fetch_nodes_by_ids(&pool, &node_ids).await.is_ok());
    /// }
    /// ```
    ///
    pub async fn fetch_nodes_by_ids(
        &mut self,
        pool: &sqlx::PgPool,
        node_ids: &Vec<&str>,
    ) -> Result<&Self, ValidationError> {
        let nodes = match self.fetch_nodes_from_db(pool, node_ids).await {
            Ok(nodes) => nodes,
            Err(e) => {
                return Err(ValidationError::new(
                    &format!("Error in fetch_nodes_from_db: {}", e),
                    vec![],
                ))
            }
        };

        for node in nodes {
            self.add_node(node);
        }

        Ok(self)
    }

    /// Fetch the linked nodes with some relation types or other conditions, but only one step
    pub async fn fetch_linked_nodes(
        &mut self,
        pool: &sqlx::PgPool,
        query: &Option<ComposeQuery>,
        page: Option<u64>,
        page_size: Option<u64>,
        order_by: Option<&str>,
    ) -> Result<&Self, ValidationError> {
        match RecordResponse::<Relation>::get_records(
            pool,
            "biomedgps_relation",
            query,
            page,
            page_size,
            order_by,
        )
        .await
        {
            Ok(records) => {
                for record in records.records {
                    let edge = Edge::new(&record);
                    self.add_edge(edge);
                }

                // Fetch the nodes
                let node_ids = self.get_node_ids_from_edges();
                let node_ids_str = &node_ids.iter().map(|id| id.as_str()).collect();
                match self.fetch_nodes_from_db(pool, node_ids_str).await {
                    Ok(nodes) => {
                        for node in nodes {
                            self.add_node(node);
                        }
                    }
                    Err(e) => {
                        let error_msg = format!("Error in fetch_nodes_from_db: {}", e);
                        return Err(ValidationError::new(&error_msg, vec![]));
                    }
                };

                Ok(self)
            }
            Err(e) => {
                let error_msg = format!("Error in fetch_linked_nodes: {}", e);
                Err(ValidationError::new(&error_msg, vec![]))
            }
        }
    }

    // Fetch the linked nodes within n steps with some relation types or other conditions
    pub async fn fetch_linked_nodes_within_steps() {}
}

#[cfg(test)]
mod tests {
    extern crate log;
    extern crate stderrlog;
    use super::*;
    use crate::init_log;
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
