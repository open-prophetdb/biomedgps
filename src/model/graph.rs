//! Graph module is used to define the graph data structure and its related functions.
//!
//! NOTICE:
//! - The graph data structure is different from the Entity struct. The Entity struct is used to represent the entity data in the database. The graph data structure is used to represent the graph data which can be used by the frontend to render the graph.
//! - The module is used to fetch the graph data from the postgresql database or neo4j graph database and convert it to the graph data structure which can be used by the frontend.
//!

use super::core::KnowledgeCuration;
use super::init_sql::get_kg_score_table_name;
use crate::model::core::{Entity, RecordResponse, Relation, DEFAULT_DATASET_NAME};
use crate::model::init_sql::get_triple_entity_score_table_name;
use crate::model::kge::{
    get_embedding_metadata, get_entity_emb_table_name, get_relation_emb_table_name,
    EmbeddingMetadata, DEFAULT_MODEL_NAME,
};
use crate::model::util::match_color;
use crate::model::util::ValidationError;
use crate::query_builder::sql_builder::ComposeQuery;
use lazy_static::lazy_static;
use log::{debug, error};
use neo4rs::{Node as NeoNode, Relation as NeoRelation};
use poem_openapi::Object;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::vec;

// The delimiter is defined here, if we want to change it, please change it here.
pub const COMPOSED_ENTITY_DELIMITER: &str = "::";
pub const PREDICTED_EDGE_TYPE: &str = "PredictedRelation";

lazy_static! {
    pub static ref COMPOSED_ENTITY_REGEX: Regex =
        Regex::new(r"^[A-Za-z]+::[A-Za-z0-9\-]+:[a-z0-9A-Z\.\-_]+$").unwrap();

    // There is a comma between the composed entitys, each composed entity must be composed of entity type, ::, and entity id. e.g. Disease::MESH:D001755,Drug::CHEMBL:CHEMBL88
    pub static ref COMPOSED_ENTITIES_REGEX: Regex =
        Regex::new(r"^[A-Za-z]+::[A-Za-z0-9\-]+:[a-z0-9A-Z\.\-_]+(,[A-Za-z]+::[A-Za-z0-9\-]+:[a-z0-9A-Z\.\-_]+)*$").unwrap();

    // The composed relation type is like "biomedgps::relation_type_abbr::Compound:Disease"
    pub static ref RELATION_TYPE_REGEX: Regex = Regex::new(r"^([A-Za-z0-9\-_]+)::([A-Za-z0-9\-_ \+]+)::([A-Z][a-z]+):([A-Z][a-z]+)$").unwrap();

    // Only for predicted edge
    pub static ref PREDICTED_EDGE_COLOR_MAP: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        // #89CFF0 is light green
        m.insert(PREDICTED_EDGE_TYPE, "#89CFF0");
        // #ccc is gray
        m.insert("Default", "#ccc");
        // ...add more pairs here
        m
    };

    pub static ref PREDICTED_EDGE_TYPES: Vec<&'static str> = {
        let mut v = Vec::new();
        v.push(PREDICTED_EDGE_TYPE);
        v.push("Default");
        v
    };
}

/// A NodeKeyShape struct for the node rendering.
///
/// Need to rename the field `fill_opacity` to `fillOpacity` in the frontend. More details on https://docs.rs/poem-openapi/latest/poem_openapi/derive.Object.html
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object)]
pub struct NodeKeyShape {
    pub fill: String,
    pub stroke: String,
    pub opacity: f64,
    #[oai(rename = "fillOpacity")]
    #[serde(rename(serialize = "fillOpacity", deserialize = "fill_opacity"))]
    pub fill_opacity: f64,
}

impl NodeKeyShape {
    /// Create a NodeKeyShape according to the node label.
    pub fn new(entity_type: &str) -> Self {
        let color = match_color(entity_type);

        NodeKeyShape {
            fill: color.clone(),
            stroke: color,
            opacity: 0.95,
            fill_opacity: 0.95,
        }
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
    pub fn new(entity_type: &str) -> Self {
        let first_char = entity_type
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

/// A label struct for the node rendering.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object)]
pub struct Label {
    pub value: String,
    pub fill: String,
    #[oai(rename = "fontSize")]
    #[serde(rename(serialize = "fontSize", deserialize = "font_size"))]
    pub font_size: i32,
    pub offset: i32,
    pub position: String, // "top" or "bottom"
}

impl Label {
    /// Use the node name as the label value.
    pub fn new(name: &str) -> Self {
        Label {
            value: name.to_string(), // It will be shown at the bottom of the node
            fill: "#000".to_string(),
            font_size: 12,
            offset: 0,
            position: "bottom".to_string(),
        }
    }
}

/// A style struct for the node rendering.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object)]
pub struct NodeStyle {
    pub label: Label,
    pub keyshape: NodeKeyShape,
    pub icon: Icon,
}

impl NodeStyle {
    /// Create a NodeStyle according to the node label.
    pub fn new(entity: &Entity) -> Self {
        NodeStyle {
            label: Label::new(&entity.name.as_str()),
            keyshape: NodeKeyShape::new(entity.label.as_str()),
            icon: Icon::new(entity.label.as_str()),
        }
    }

    /// Create a NodeStyle from a NodeData struct.
    pub fn from_node_data(node: &NodeData) -> Self {
        NodeStyle {
            label: Label::new(&node.name),
            keyshape: NodeKeyShape::new(&node.label.as_str()),
            icon: Icon::new(&node.label.as_str()),
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
    #[oai(skip_serializing_if_is_none)]
    pub description: Option<String>,
    pub resource: String,
    // In future, we can add more fields here after we add additional fields for the Entity struct
}

impl NodeData {
    /// Create a NodeData from an Entity.
    pub fn new(entity: &Entity) -> Self {
        // The identity is same with the id in the graph database.
        let identity = Self::format_id(&entity.label, &entity.id);
        NodeData {
            identity: identity.clone(),
            id: entity.id.clone(),
            label: entity.label.clone(),
            name: entity.name.clone(),
            description: entity.description.clone(),
            resource: entity.resource.clone(),
        }
    }

    pub fn from_neo_node(node: NeoNode) -> Self {
        let labels = node.labels();
        let id = node.get::<String>("id").unwrap_or_default();
        let identity = Self::format_id(&labels[0], &id);

        NodeData {
            identity: identity,
            id: id,
            label: labels[0].to_owned(),
            name: node.get::<String>("name").unwrap_or_default(),
            description: node.get::<String>("description"),
            resource: node.get::<String>("resource").unwrap_or_default(),
        }
    }

    /// Parse the node id to get the label and entity id.
    pub fn parse_id(id: &str) -> (String, String) {
        let parts: Vec<&str> = id.split(COMPOSED_ENTITY_DELIMITER).collect();
        (parts[0].to_string(), parts[1].to_string())
    }

    /// Format the node id, we use the label and entity id to format the node id.
    pub fn format_id(label: &str, entity_id: &str) -> String {
        // format!("{}{}{}", label, COMPOSED_ENTITY_DELIMITER, entity_id)
        Node::format_id(label, entity_id)
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
    #[oai(rename = "comboId")]
    #[serde(rename(serialize = "comboId", deserialize = "combo_id"))]
    #[oai(skip_serializing_if_is_none)]
    pub combo_id: Option<String>,
    pub id: String,
    // For showing a name in the frontend
    pub label: String,
    pub nlabel: String,
    pub degree: Option<i32>, // Map degree to node size
    pub style: NodeStyle,
    pub category: String, // node or edge
    #[oai(skip_serializing_if_is_none)]
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
            label: entity.id.clone(),
            nlabel: entity.label.clone(),
            degree: None,
            style: NodeStyle::new(&entity),
            category: "node".to_string(),
            cluster: Some(entity.label.clone()),
            r#type: "graphin-circle".to_string(),
            x: None,
            y: None,
            data: NodeData::new(entity),
        }
    }

    pub fn from_node_data(node: &NodeData) -> Self {
        Node {
            combo_id: None,
            id: node.identity.clone(),
            label: node.id.clone(),
            nlabel: node.label.clone(),
            degree: None,
            style: NodeStyle::from_node_data(&node),
            category: "node".to_string(),
            cluster: Some(node.label.clone()),
            r#type: "graphin-circle".to_string(),
            x: None,
            y: None,
            data: node.clone(),
        }
    }

    /// Parse the node id to get the label and entity id.
    pub fn parse_id(id: &str) -> (String, String) {
        let parts: Vec<&str> = id.split(COMPOSED_ENTITY_DELIMITER).collect();
        (parts[0].to_string(), parts[1].to_string())
    }

    /// Format the node id, we use the label and entity id to format the node id.
    pub fn format_id(label: &str, entity_id: &str) -> String {
        format!("{}{}{}", label, COMPOSED_ENTITY_DELIMITER, entity_id)
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

/// The EdgeKeyShape struct is used to store the edge key shape information. Only for the predicted edges.
/// In the current stage, we use the default value for the edge key shape. In future, we can add more fields to the EdgeKeyShape struct to customize the edge key shape.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object)]
pub struct EdgeKeyShape {
    #[oai(rename = "lineDash")]
    #[serde(rename(serialize = "lineDash", deserialize = "line_dash"))]
    pub line_dash: [i32; 2],

    pub stroke: String,

    #[oai(rename = "lineWidth")]
    #[serde(rename(serialize = "lineWidth", deserialize = "line_width"))]
    pub line_width: i32,
}

impl EdgeKeyShape {
    /// Create a new key shape for the edge.
    pub fn new(relation_type: &str) -> Self {
        let color = if relation_type == PREDICTED_EDGE_TYPE {
            PREDICTED_EDGE_COLOR_MAP.get(PREDICTED_EDGE_TYPE).unwrap()
        } else {
            PREDICTED_EDGE_COLOR_MAP.get("Default").unwrap()
        };

        EdgeKeyShape {
            line_dash: [5, 5],
            stroke: color.to_string(),
            line_width: 2,
        }
    }
}

/// The EdgeStyle struct is used to store the edge style information. The frontend will use these information to render the edge.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Object)]
pub struct EdgeStyle {
    pub label: EdgeLabel,
    #[oai(skip_serializing_if_is_none)]
    pub keyshape: Option<EdgeKeyShape>,
}

impl EdgeStyle {
    /// Create a new style for the edge. Keyshape is only for the predicted edges.
    pub fn new(relation_type: &str) -> Self {
        if PREDICTED_EDGE_TYPES.contains(&relation_type) {
            EdgeStyle {
                label: EdgeLabel {
                    value: relation_type.to_string(),
                },
                keyshape: Some(EdgeKeyShape::new(relation_type)),
            }
        } else {
            EdgeStyle {
                label: EdgeLabel {
                    value: relation_type.to_string(),
                },
                keyshape: None,
            }
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
    pub pmids: String,
    pub dataset: String,
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
            dataset: relation
                .dataset
                .clone()
                .unwrap_or(DEFAULT_DATASET_NAME.to_string()),
            pmids: relation.pmids.clone().unwrap_or("".to_string()),
        }
    }

    pub fn from_neo_edge(
        relation: &NeoRelation,
        start_node: &NodeData,
        end_node: &NodeData,
    ) -> Self {
        Self {
            relation_type: relation.typ().to_string(),
            source_id: start_node.id.clone(),
            source_type: start_node.label.clone(),
            target_id: end_node.id.clone(),
            target_type: end_node.label.clone(),
            score: relation.get::<f64>("score").unwrap_or_default(),
            key_sentence: relation.get::<String>("key_sentence").unwrap_or_default(),
            resource: relation.get::<String>("resource").unwrap_or_default(),
            dataset: relation.get::<String>("dataset").unwrap_or_default(),
            pmids: relation.get::<String>("pmids").unwrap_or_default(),
        }
    }
}

/// The edge struct which is compatible with the Graphin format
///
/// * `relid` - The id of the edge. It's the combination of the source id, the relation type and the target id.
/// * `source` - The source and target fields are the id of the node. It must be the same as the id field of the Node struct. Otherwise, the edge will not be connected to the node. such as "Compound::MESH:D0001"
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
    /// Create a new edge.
    pub fn new(
        relation_type: &str,
        source_id: &str,
        source_type: &str,
        target_id: &str,
        target_type: &str,
        distance: Option<f64>,
    ) -> Self {
        let relid = format!("{}-{}-{}", source_id, relation_type, target_id);

        Edge {
            relid: relid.clone(),
            source: Node::format_id(source_type, source_id),
            category: "edge".to_string(),
            target: Node::format_id(target_type, target_id),
            reltype: relation_type.to_string(),
            style: EdgeStyle::new(relation_type),
            data: EdgeData {
                relation_type: relation_type.to_string(),
                source_id: source_id.to_string(),
                source_type: source_type.to_string(),
                target_id: target_id.to_string(),
                target_type: target_type.to_string(),
                score: distance.unwrap_or(0.0),
                key_sentence: "".to_string(),
                resource: "".to_string(),
                dataset: DEFAULT_DATASET_NAME.to_string(),
                pmids: "".to_string(),
            },
        }
    }

    /// Create a new edge from an EdgeData struct.
    pub fn from_edge_data(edge: &EdgeData) -> Self {
        Edge {
            relid: format!(
                "{}-{}-{}",
                edge.source_id, edge.relation_type, edge.target_id
            ),
            source: Node::format_id(&edge.source_type, &edge.source_id),
            category: "edge".to_string(),
            target: Node::format_id(&edge.target_type, &edge.target_id),
            reltype: edge.relation_type.clone(),
            style: EdgeStyle::new(&edge.relation_type),
            data: edge.clone(),
        }
    }

    /// It will convert the [`Relation`](struct.Relation.html) struct to the [`Edge`](struct.Edge.html) struct.
    pub fn from_relation(relation: &Relation) -> Self {
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

    pub fn from_curated_knowledge(knowledge: &KnowledgeCuration) -> Self {
        let relid = format!(
            "{}-{}-{}",
            knowledge.source_id, knowledge.relation_type, knowledge.target_id
        );
        Edge {
            relid: relid.clone(),
            source: Node::format_id(&knowledge.source_type, &knowledge.source_id),
            category: "edge".to_string(),
            target: Node::format_id(&knowledge.target_type, &knowledge.target_id),
            reltype: knowledge.relation_type.clone(),
            style: EdgeStyle::new(&knowledge.relation_type),
            data: EdgeData::new(&knowledge.to_relation()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::FromRow)]
struct TargetNode {
    query_node_id: String,
    node_id: String,
    score: Option<f32>, // The score is the distance between the nodes and the relation type
}

impl TargetNode {
    /// Fetch the target nodes from the database by node id. It is based on the node embeddings and relation_type embedding.
    /// We will use custom functions in the pgml extension to calculate the score between the nodes and the relation_type.
    ///
    /// # Arguments
    ///
    /// * `pool` - The database connection pool.
    /// * `node_id` - The id of the node. It is the combination of the node type and the node id. Such as "Gene::ENTREZ:123".
    /// * `relation_type` - The relation type of the nodes. It is the combination of the source type and the target type. Such as "STRING::BINDING::Gene:Gene".
    /// * `query` - The query to filter the nodes. It is a compose query. More details on the compose query can be found in the [`ComposeQuery`](struct.ComposeQuery.html) struct.
    /// * `topk` - The number of the target nodes to be fetched. default is 10.
    ///
    /// # Returns
    ///
    /// * `Result<Vec<Self>, ValidationError>` - The target nodes.
    ///
    pub async fn fetch_target_nodes(
        pool: &sqlx::PgPool,
        node_id: &str, // The id of the node, it might be a list of node ids separated by comma
        relation_type: &str,
        query: &Option<ComposeQuery>,
        topk: Option<u64>,
        model_table_name: Option<String>,
    ) -> Result<Vec<Self>, ValidationError> {
        let model_or_table_name = match model_table_name {
            Some(name) => name,
            None => DEFAULT_MODEL_NAME.to_string(),
        };

        // The first one is the node itself, so we need to add 1 to the topk
        let topk = match topk {
            Some(topk) => topk,
            None => 10,
        };

        let node_ids = node_id.split(",").collect::<Vec<&str>>();
        let mut entity_types: Vec<String> = vec![];
        let mut entity_ids: Vec<String> = vec![];
        for id in node_ids {
            let (entity_type, entity_id) = Node::parse_id(id);
            entity_types.push(entity_type);
            entity_ids.push(entity_id);
        }

        let entity_id = entity_ids.join(",");
        let entity_type = entity_types.join(",");

        let embedding_metadata = match get_embedding_metadata(&model_or_table_name) {
            Some(metadata) => metadata,
            None => {
                error!("Failed to get the embedding metadata from the database");
                return Err(ValidationError::new(
                    "Failed to get the embedding metadata from the database, so we don't know how to calculate the similarity for the node. Please check the database or the model/table name you provided.",
                    vec![],
                ));
            }
        };

        // TODO: We need to allow the user to set the score function, gamma and exp_enabled
        let gamma = 12.0;
        let sql_str = match Graph::format_score_sql(
            &entity_id,
            &entity_type,
            relation_type,
            &embedding_metadata,
            topk,
            gamma,
        ) {
            Ok(sql_str) => sql_str,
            Err(err) => {
                let err_msg = format!("Failed to format the score sql: {}", err);
                error!("{}", &err_msg);
                return Err(ValidationError::new(&err_msg, vec![]));
            }
        };

        debug!(
            "sql_str: {} with arguments node_id: `{}`, relation_type: `{}`, topk: `{:?}`",
            sql_str, node_id, relation_type, topk
        );

        match sqlx::query_as::<_, Self>(sql_str.as_str())
            .fetch_all(pool)
            .await
        {
            Ok(nodes) => {
                let filtered_nodes = nodes
                    .into_iter()
                    .filter(|node| node.score.is_some())
                    .collect::<Vec<Self>>();

                if filtered_nodes.is_empty() {
                    let err_msg = format!(
                        "No similar nodes found for the node id `{}` and the relation type `{}`, you may need to check the node id or ask the admin to check if the embedding database matches the entity database",
                        node_id, relation_type
                    );
                    error!("{}", &err_msg);
                    return Err(ValidationError::new(&err_msg, vec![]));
                } else {
                    return Ok(filtered_nodes);
                }
            }
            Err(err) => {
                let err_msg = format!("Failed to fetch similarity nodes from database: {}", err);
                error!("{}", &err_msg);
                Err(ValidationError::new(&err_msg, vec![]))
            }
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

    ///
    pub fn from_data(nodes: Vec<&NodeData>, edges: Vec<&EdgeData>) -> Self {
        let mut graph = Graph::new();
        for node in nodes {
            let node = Node::from_node_data(&node);
            graph.add_node(node);
        }

        for edge in edges {
            let edge = Edge::from_edge_data(&edge);
            graph.add_edge(edge);
        }

        graph
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
    /// let edge = Edge::from_relation(&relation);
    /// graph.add_edge(edge);
    ///
    /// let edges = graph.get_edges(None).unwrap();
    /// assert_eq!(edges.len(), 1);
    /// ```
    ///
    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.push(edge);
    }

    /// Remove the edges by node id
    /// It will remove the edges which contain the node id as the source or target node id.
    pub fn remove_edges_by_node_id(&mut self, node_id: &str) {
        self.edges = self
            .edges
            .iter()
            .filter(|edge| edge.source != node_id && edge.target != node_id)
            .map(|edge| edge.to_owned())
            .collect::<Vec<Edge>>();
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
                COMPOSED_ENTITY_DELIMITER,
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

    /// Parse the relation type to get the source type and target type
    ///
    /// # Example
    ///
    /// ```
    /// use biomedgps::model::graph::Graph;
    ///
    /// let relation_type = "biomedgps::Inhibitor::Gene:Gene";
    /// let (source_type, target_type) = Graph::parse_relation_type(relation_type).unwrap();
    /// assert_eq!(source_type, "Gene");
    /// assert_eq!(target_type, "Gene");
    /// ```
    ///
    /// # Errors
    ///
    /// If the relation type is not valid, it will return an error.
    ///
    /// ```
    /// use biomedgps::model::graph::Graph;
    ///
    /// let relation_type = "biomedgps::Inhibitor::Gene:Gene::";
    /// let result = Graph::parse_relation_type(relation_type);
    /// assert!(result.is_err());
    /// ```
    ///
    /// # Arguments
    ///
    /// * `relation_type` - The relation type, like "biomedgps::Inhibitor::Gene:Gene"
    ///
    /// # Returns
    ///
    /// * `Ok((source_type, target_type))` - The source type and target type
    /// * `Err(ValidationError)` - The error message and the invalid relation type
    ///
    pub fn parse_relation_type(relation_type: &str) -> Result<(String, String), ValidationError> {
        RELATION_TYPE_REGEX
            .captures(relation_type)
            .map(|captures| {
                (
                    captures.get(3).unwrap().as_str().to_string(),
                    captures.get(4).unwrap().as_str().to_string(),
                )
            })
            .ok_or_else(|| {
                ValidationError::new(
                    &format!("The relation type is not valid: {}", relation_type),
                    vec![relation_type.to_string()],
                )
            })
    }

    /// Generate the SQL to fetch the target nodes from the database
    ///
    /// # Arguments
    ///
    /// * `source_id` - The id of the source node
    /// * `source_type` - The type of the source node
    /// * `target_type` - The type of the target node
    /// * `relation_type` - The relation type of the nodes
    /// * `embedding_metadata` - The metadata of the embedding
    /// * `topk` - The number of the target nodes to be fetched
    /// * `gamma` - The gamma value for the score function
    ///
    /// # Returns
    ///
    /// * `Result<String, ValidationError>` - The SQL to fetch the target nodes
    ///
    /// # Example
    ///
    /// ```
    /// use regex::Regex;
    /// use chrono::{Utc, NaiveDateTime, DateTime};
    /// use biomedgps::model::graph::Graph;
    /// use biomedgps::model::kge::EmbeddingMetadata;
    ///
    /// let source_id = "ENTREZ:6747";
    /// let source_type = "Gene";
    /// let relation_type = "STRING::BINDING::Gene:Gene";
    /// let embedding_metadata = EmbeddingMetadata {
    ///    id: 1,
    ///    metadata: None,
    ///    model_name: "biomedgps_transe_l2".to_string(),
    ///    model_type: "TransE_l2".to_string(),
    ///    dimension: 400,
    ///    table_name: "biomedgps".to_string(),
    ///    created_at: DateTime::from_utc(NaiveDateTime::from_timestamp(0, 0), Utc),
    ///    datasets: vec!("STRING".to_string()),
    ///    description: "The entity embedding trained by the TransE_l2 model".to_string(),
    /// };
    /// let topk = 10;
    /// let gamma = 12.0;
    /// let sql_str = Graph::format_score_sql(source_id, source_type, relation_type, &embedding_metadata, topk, gamma).unwrap();
    /// let expected_sql_str = "
    /// SELECT
    ///     COALESCE(ee2.entity_type, '') || '::' || COALESCE(ee2.entity_id, '') AS node_id,
    ///     pgml.transe_l2_ndarray(
    ///         vector_to_float4(ee1.embedding, 400, false),
    ///         vector_to_float4(rte.embedding, 400, false),
    ///         vector_to_float4(ee2.embedding, 400, false),
    ///         12,
    ///         true,
    ///         true
    ///     ) AS score
    /// FROM
    ///     biomedgps_entity_embedding ee1,
    ///     biomedgps_relation_embedding rte,
    ///     biomedgps_entity_embedding ee2
    /// WHERE
    ///     ee1.entity_id = 'ENTREZ:6747'
    ///     AND ee1.entity_type = 'Gene'
    ///     AND ee2.entity_type = 'Gene'
    ///     AND rte.relation_type = 'STRING::BINDING::Gene:Gene'
    /// GROUP BY
    ///     ee1.embedding_id,
    ///     rte.embedding_id,
    ///     ee2.embedding_id
    /// ORDER BY score DESC
    /// LIMIT 10";
    ///
    /// // Clear the white spaces
    /// let re = Regex::new(r"\s+").unwrap();
    /// let trimmed_sql_str = re.replace_all(&sql_str, " ").trim().to_string();
    /// let trimmed_expected_sql_str = re.replace_all(&expected_sql_str, " ").trim().to_string();
    /// assert_eq!(trimmed_sql_str, trimmed_expected_sql_str);
    /// ```
    pub fn format_score_sql(
        source_id: &str, // The id of the source node, it might be a list of ids separated by comma
        source_type: &str,
        relation_type: &str,
        embedding_metadata: &EmbeddingMetadata,
        topk: u64,
        gamma: f64,
    ) -> Result<String, ValidationError> {
        let source_id = source_id.split(",").collect::<Vec<&str>>().join("', '");
        let source_type_vec = source_type
            .split(",")
            .collect::<HashSet<&str>>()
            .into_iter()
            .collect::<Vec<&str>>();
        if source_type_vec.len() > 1 {
            return Err(ValidationError::new(
                "The source type is not valid, it should be a single type",
                vec![source_type_vec.join(",")],
            ));
        }
        let source_type = source_type_vec[0];

        let (r_source_type, r_target_type) = match Graph::parse_relation_type(relation_type) {
            Ok((source_type, target_type)) => (source_type, target_type),
            Err(err) => {
                error!("Failed to parse the relation type: {}", err);
                return Err(ValidationError::new(
                    "Failed to parse the relation type, please check your input.",
                    vec![],
                ));
            }
        };

        if r_source_type != source_type && r_target_type != source_type {
            return Err(ValidationError::new(
                &format!(
                    "The source type {} is not in the relation type {}",
                    source_type, relation_type
                ),
                vec![source_type.to_string()],
            ));
        }

        let reverse = if source_type == r_target_type {
            true
        } else {
            false
        };

        let target_type = if source_type == r_target_type {
            r_source_type
        } else {
            r_target_type
        };

        // TODO: We need to add more score functions here
        let score_function_name = if embedding_metadata.model_type == "TransE_l2" {
            "pgml.transe_l2_ndarray"
        } else if embedding_metadata.model_type == "TransE_l1" {
            "pgml.transe_l1_ndarray"
        } else if embedding_metadata.model_type == "DistMult" {
            "pgml.distmult_ndarray"
        } else if embedding_metadata.model_type == "ComplEx" {
            "pgml.complex_ndarray"
        } else {
            "pgml.transe_l2_ndarray"
        };

        let sql_str = if relation_type == "DrugBank::treats::Compound:Symptom" {
            format!(
                "
                    SELECT
                        COALESCE(target_type, '') || '::' || COALESCE(target_id, '') AS query_node_id,
                        COALESCE(source_type, '') || '::' || COALESCE(source_id, '') AS node_id,
                        percentile_cont(0.5) WITHIN GROUP (ORDER BY score)::FLOAT4 AS score
                    FROM
                        {table_name} ee1
                    WHERE
                        target_id IN ('{source_id}') AND target_type = '{source_type}'
                    GROUP BY
                        target_id, target_type, source_id, source_type
                    ORDER BY score DESC, node_id ASC
                    LIMIT {topk};
                ",
                table_name = get_triple_entity_score_table_name(
                    &embedding_metadata.table_name,
                    "Compound",
                    "Disease",
                    "Symptom"
                ),
                source_id = source_id,
                source_type = source_type,
                topk = topk
            )
        } else {
            // Example SQL:
            // SELECT
            //     ee1.entity_id AS head,
            //     rte.relation_type AS relation_type,
            //     ee2.entity_id AS tail,
            //     pgml.transe_l2_ndarray(
            //             vector_to_float4(ee1.embedding, 400, false),
            //             vector_to_float4(rte.embedding, 400, false),
            //             vector_to_float4(ee2.embedding, 400, false),
            //             12.0,
            //             true
            //     ) AS score
            // FROM
            //     biomedgps_entity_embedding ee1,
            //     biomedgps_relation_embedding rte,
            //     biomedgps_entity_embedding ee2
            // WHERE
            //     ee1.entity_id = 'ENTREZ:6747'
            //     AND ee1.entity_type = 'Gene'
            //     AND ee2.entity_type = 'Gene'
            //     AND rte.relation_type = 'STRING::BINDING::Gene:Gene'
            // GROUP BY
            //     ee1.embedding_id,
            //     rte.embedding_id,
            //     ee2.embedding_id
            // ORDER BY score DESC
            // LIMIT 10
            format!(
                "SELECT
                    COALESCE(ee1.entity_type, '') || '::' || COALESCE(ee1.entity_id, '') AS query_node_id,
                    COALESCE(ee2.entity_type, '') || '::' || COALESCE(ee2.entity_id, '') AS node_id,
                    {score_function_name}(
                            vector_to_float4(ee1.embedding, {dimension}, false),
                            vector_to_float4(rte.embedding, {dimension}, false),
                            vector_to_float4(ee2.embedding, {dimension}, false),
                            {gamma},
                            true,
                            {reverse}
                    ) AS score	
                FROM
                    {entity_embedding_table} ee1,
                    {relation_type_embedding_table} rte,
                    {entity_embedding_table} ee2
                WHERE
                    ee1.entity_id IN ('{source_id}')
                    AND ee1.entity_type = '{source_type}'
                    AND ee2.entity_type = '{target_type}'
                    AND rte.relation_type = '{relation_type}'
                GROUP BY
                    ee1.embedding_id,
                    rte.embedding_id,
                    ee2.embedding_id
                ORDER BY score DESC
                LIMIT {topk}",
                source_id = source_id,
                source_type = source_type,
                target_type = target_type,
                relation_type = relation_type,
                dimension = embedding_metadata.dimension,
                topk = topk,
                reverse = reverse,
                entity_embedding_table = get_entity_emb_table_name(&embedding_metadata.table_name),
                relation_type_embedding_table =
                    get_relation_emb_table_name(&embedding_metadata.table_name),
                score_function_name = score_function_name,
                gamma = gamma
            )
        };

        Ok(sql_str)
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
                COMPOSED_ENTITY_DELIMITER,
                filtered_node_ids.join("', '"),
                COMPOSED_ENTITY_DELIMITER,
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
                    let edge = Edge::from_relation(&record);
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

    /// Fetch the similar nodes from the database by node id and convert them to nodes and edges in the graph.
    ///
    /// NOTICE: All edges are not real edges, they are just used to show the similarity between nodes.
    ///
    /// # Arguments
    ///
    /// * `pool` - The database connection pool
    /// * `node_id` - The node id, like `Compound::MESH:D0001`
    /// * `query` - The query to filter the nodes
    /// * `topk` - The number of nodes to return
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
    /// use biomedgps::query_builder::sql_builder::ComposeQuery;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let database_url = "postgres://postgres:password@localhost:5432/test_biomedgps";
    ///     let pool = PgPool::connect(database_url).await.unwrap();
    ///     let mut graph = Graph::new();
    ///     let node_id = "Compound::MESH:C000601183";
    ///     let query = None;
    ///     let topk = Some(10);
    ///
    ///     // If you choose None as the model_table_name, it will use the default model/table name `DEFAULT_MODEL_NAME`.
    ///     match graph.fetch_predicted_nodes(&pool, &node_id, &query, topk, None).await {
    ///         Ok(graph) => {
    ///             println!("graph: {:?}", graph);
    ///         }
    ///         Err(e) => {
    ///             println!("Error: {}", e);
    ///         }
    ///     }
    /// }
    pub async fn fetch_predicted_nodes(
        &mut self,
        pool: &sqlx::PgPool,
        node_id: &str, // The id of the source node, it might be a list of ids separated by comma
        relation_type: &str,
        query: &Option<ComposeQuery>,
        topk: Option<u64>,
        model_table_name: Option<String>,
    ) -> Result<&Self, ValidationError> {
        match TargetNode::fetch_target_nodes(
            pool,
            node_id,
            relation_type,
            query,
            topk,
            model_table_name,
        )
        .await
        {
            Ok(predicted_nodes) => {
                let mut node_ids = predicted_nodes
                    .iter()
                    .map(|predicted_node| predicted_node.node_id.as_str())
                    .collect::<Vec<&str>>();

                let node_id_vec = node_id.split(",").collect::<Vec<&str>>();
                for id in &node_id_vec {
                    node_ids.push(id);
                }

                // Convert predicted nodes to a hashmap which key is node id and value is distance.
                let predicted_node_map = predicted_nodes
                    .iter()
                    .map(|predicted_node| {
                        let key = format!(
                            "{}-{}",
                            predicted_node.query_node_id, predicted_node.node_id
                        );
                        (key, predicted_node.score.unwrap() as f64)
                    })
                    .collect::<HashMap<String, f64>>();

                // Allow to label the existing records with any relation type
                let existing_records = match Relation::exist_records(
                    pool, node_id, &node_ids, None, // Some(relation_type),
                    true,
                )
                .await
                {
                    Ok(records) => records,
                    Err(e) => {
                        return Err(ValidationError::new(
                            &format!("Error in exist_records: {}", e),
                            vec![],
                        ))
                    }
                };

                for node_id in &node_id_vec {
                    let edges = match self.fetch_nodes_by_ids(pool, &node_ids).await {
                        Ok(graph) => {
                            let nodes = &graph.nodes;
                            let source_node =
                                match nodes.iter().find(|node| node.id == node_id.to_string()) {
                                    Some(node) => node,
                                    None => {
                                        return Err(ValidationError::new(
                                            &format!("The source node {} is not found", node_id),
                                            vec![node_id.to_string()],
                                        ))
                                    }
                                };

                            let mut edges = vec![];
                            for node in nodes {
                                let key = format!("{}-{}", source_node.id, node.id);
                                let distance = predicted_node_map.get(&key);
                                match distance {
                                    Some(&d) => {
                                        if node.id == source_node.id {
                                            continue;
                                        }

                                        let first_node_id = format!(
                                            "{}{}{}",
                                            source_node.data.label,
                                            COMPOSED_ENTITY_DELIMITER,
                                            source_node.data.id
                                        );
                                        let second_node_id = format!(
                                            "{}{}{}",
                                            node.data.label,
                                            COMPOSED_ENTITY_DELIMITER,
                                            node.data.id
                                        );
                                        let ordered_key_str = Relation::gen_composed_key(
                                            &first_node_id,
                                            &second_node_id,
                                        );
                                        let edge = match existing_records.get(&ordered_key_str) {
                                            Some(record) => Edge::new(
                                                &record.relation_type,
                                                source_node.data.id.as_str(),
                                                source_node.data.label.as_str(),
                                                node.data.id.as_str(),
                                                node.data.label.as_str(),
                                                Some(d),
                                            ),
                                            None => Edge::new(
                                                PREDICTED_EDGE_TYPE,
                                                source_node.data.id.as_str(),
                                                source_node.data.label.as_str(),
                                                node.data.id.as_str(),
                                                node.data.label.as_str(),
                                                Some(d),
                                            ),
                                        };

                                        edges.push(edge);
                                    }
                                    None => {
                                        continue;
                                    }
                                }
                            }

                            edges
                        }
                        Err(e) => {
                            return Err(ValidationError::new(
                                &format!("Error in fetch_nodes_by_ids: {}", e),
                                vec![],
                            ))
                        }
                    };

                    for edge in edges {
                        self.add_edge(edge);
                    }
                }

                Ok(self)
            }
            Err(e) => Err(ValidationError::new(
                &format!("Error in fetch_predicted_nodes: {}", e),
                vec![],
            )),
        }
    }

    /// Fetch the curated knowledges and convert them to nodes and edges in the graph.
    pub async fn fetch_curated_knowledges(
        &mut self,
        pool: &sqlx::PgPool,
        curator: &str,
        project_id: i32,
        organization_id: i32,
        page: Option<u64>,
        page_size: Option<u64>,
        order_by: Option<&str>,
        strict_mode: bool,
    ) -> Result<&Self, ValidationError> {
        match KnowledgeCuration::get_records_by_owner(
            pool,
            curator,
            project_id,
            organization_id,
            page,
            page_size,
            order_by,
        )
        .await
        {
            Ok(records) => {
                for record in records.records {
                    // Skip the records with unknown source or target id or the source or target id is not composed of node type and node id
                    if strict_mode {
                        if record.source_id == "Unknown:Unknown"
                            || record.target_id == "Unknown:Unknown"
                        {
                            continue;
                        }

                        if !record.source_id.contains(":") || !record.target_id.contains(":") {
                            continue;
                        }
                    }

                    let edge = Edge::from_curated_knowledge(&record);
                    self.add_edge(edge);
                }

                // Fetch the nodes
                let node_ids = self.get_node_ids_from_edges();
                let node_ids_str = &node_ids.iter().map(|id| id.as_str()).collect();
                match self.fetch_nodes_from_db(pool, node_ids_str).await {
                    Ok(nodes) => {
                        // Keep all nodes which don't exist in our knowledge graph
                        let missed_node_ids = node_ids
                            .iter()
                            .filter(|&node_id| {
                                !nodes
                                    .iter()
                                    .map(|node| node.id.as_str())
                                    .collect::<Vec<&str>>()
                                    .contains(&&node_id[..])
                            })
                            .map(|node_id| node_id.to_string())
                            .collect::<Vec<String>>();

                        log::debug!("nodes: {:?}", nodes);
                        for node in nodes {
                            self.add_node(node);
                        }

                        log::info!("missed_node_ids: {:?}", missed_node_ids);
                        for node_id in missed_node_ids {
                            self.remove_edges_by_node_id(node_id.as_str());
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
                let error_msg = format!("Error in fetch_curated_knowledges: {}", e);
                Err(ValidationError::new(&error_msg, vec![]))
            }
        }
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
        let table_name = if order_by.is_some() && order_by.unwrap().starts_with("score") {
            // TODO: We need to add the model name to the query if we allow users to use different model.
            // TODO: We need to ensure the table exists before we use it.
            get_kg_score_table_name(DEFAULT_MODEL_NAME)
        } else {
            "biomedgps_relation".to_string()
        };

        match RecordResponse::<Relation>::get_records(
            pool,
            table_name.as_str(),
            query,
            page,
            page_size,
            order_by,
        )
        .await
        {
            Ok(records) => {
                for record in records.records {
                    let edge = Edge::from_relation(&record);
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
}

#[cfg(test)]
mod tests {
    extern crate log;
    use super::*;
    use crate::{init_logger, setup_test_db};
    use log::LevelFilter;
    use regex::Regex;

    #[test]
    fn test_parse_composed_node_ids() {
        let _ = init_logger("biomedgps-test", LevelFilter::Debug);
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
        let _ = init_logger("biomedgps-test", LevelFilter::Debug);
        let node_ids = vec!["Gene::ENTREZ:1", "Gene::ENTREZ:2", "Gene::ENTREZ:3"];
        let query_str = Graph::gen_entity_query_from_node_ids(&node_ids);

        // Remove the newlines and unnecessary spaces by using regex
        let re = Regex::new(r"\s+").unwrap();
        let query_str = re.replace_all(query_str.as_str(), " ");

        assert_eq!(query_str, "SELECT * FROM biomedgps_entity WHERE COALESCE(label, '') || '::' || COALESCE(id, '') in ('Gene::ENTREZ:1', 'Gene::ENTREZ:2', 'Gene::ENTREZ:3');")
    }

    #[test]
    fn test_gen_relation_query_from_node_ids() {
        let _ = init_logger("biomedgps-test", LevelFilter::Debug);
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
        let _ = init_logger("biomedgps-test", LevelFilter::Debug);
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

    #[tokio::test]
    async fn test_fetch_predicted_nodes() {
        let _ = init_logger("biomedgps-test", LevelFilter::Debug);

        let mut graph = Graph::new();

        let pool = setup_test_db().await;

        let node_id = "Chemical::MESH:C000601183";
        let relation_type = "biomedgps::treats::Compound:Disease";
        let query = None;
        let topk = Some(10);

        match graph
            .fetch_predicted_nodes(&pool, &node_id, &relation_type, &query, topk, None)
            .await
        {
            Ok(graph) => {
                debug!("graph: {:?}", graph);
            }
            Err(e) => {
                error!("Error: {}", e);
            }
        }
    }
}
