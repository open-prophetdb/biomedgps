use crate::model::graph::{EdgeData, NodeData, COMPOSED_ENTITY_DELIMITER, COMPOSED_ENTITY_REGEX};
use log::{debug, error, info};
use neo4rs::{query, Graph, Node as NeoNode, Relation, RowStream};
use std::collections::HashMap;

/// Split the composed entity id into two parts: the entity type and the entity id.
///
/// # Arguments
/// * `id` - The composed entity id. Such as 'Compound::DrugBank:DB00818'
///
/// # Returns
/// * `Ok((start_node_type, start_node_id))` - The start node type and the start node id.
/// * `Err(e)` - The error message.
///
/// # Example
/// ```
/// use biomedgps::query_builder::cypher_builder::split_id;
///
/// let id = "Compound::DrugBank:DB00818";
/// let (start_node_type, start_node_id) = split_id(id).unwrap();
/// assert_eq!(start_node_type, "Compound");
/// assert_eq!(start_node_id, "DrugBank:DB00818");
/// ```
fn split_id(id: &str) -> Result<(String, String), anyhow::Error> {
    // Check if the ids are composed entity ids.
    if !COMPOSED_ENTITY_REGEX.is_match(id) {
        return Err(anyhow::anyhow!(
            "Invalid composed entity id: {}",
            id.to_string()
        ));
    };

    let (start_node_type, start_node_id) = id.split_once(COMPOSED_ENTITY_DELIMITER).unwrap();
    Ok((start_node_type.to_string(), start_node_id.to_string()))
}

/// Generate the query string to get the nodes and edges between two nodes.
///
/// # Arguments
/// * `start_node_type` - The start node type. Such as 'Compound'
/// * `start_node_id` - The start node id. Such as 'DrugBank:DB00818'
/// * `end_node_type` - The end node type. Such as 'Disease'
/// * `end_node_id` - The end node id. Such as 'MONDO:0005404'
/// * `nhops` - The number of hops between the start node and the end node.
///
/// # Returns
/// * `query_str` - The query string.
///
/// # Example
/// ```
/// use biomedgps::query_builder::cypher_builder::gen_nhops_query_str;
///
/// let start_node_type = "Compound";
/// let start_node_id = "DrugBank:DB00818";
/// let end_node_type = "Disease";
/// let end_node_id = "MONDO:0005404";
/// let nhops = 2;
/// let query_str = gen_nhops_query_str(
///    start_node_type,
///    start_node_id,
///    end_node_type,
///    end_node_id,
///    nhops,
/// );
/// assert_eq!(
///    query_str,
///    "MATCH path = (n:Compound)-[r*..2]-(m:Disease) WHERE n.id IN ['DrugBank:DB00818'] AND m.id IN ['MONDO:0005404'] UNWIND nodes(path) AS node UNWIND relationships(path) AS edge RETURN DISTINCT node, edge"
/// );
fn gen_nhops_query_str(
    start_node_type: &str,
    start_node_id: &str,
    end_node_type: &str,
    end_node_id: &str,
    nhops: usize,
) -> String {
    let hop_str = match nhops {
        1 => "*1",
        2 => "*1..2",
        _ => "",
    };

    let query_str = format!(
        "MATCH path = (n:{start_node_type})-[r{hop_str}]-(m:{end_node_type}) WHERE n.id IN ['{start_node_id}'] AND m.id IN ['{end_node_id}'] UNWIND nodes(path) AS node UNWIND relationships(path) AS edge RETURN DISTINCT node, edge",
        start_node_type = start_node_type,
        hop_str = hop_str,
        end_node_type = end_node_type,
        start_node_id = start_node_id,
        end_node_id = end_node_id
    );

    query_str
}

async fn parse_nhops_results(
    result: &mut RowStream,
) -> Result<(Vec<NodeData>, Vec<EdgeData>), anyhow::Error> {
    let mut node_map: HashMap<i64, NodeData> = HashMap::new();
    let mut edges = Vec::new();
    let mut relations = Vec::new();

    while let Some(row) = result.next().await? {
        match row.get::<NeoNode>("node") {
            Some(node) => {
                let id = node.id();
                let node = NodeData::from_neo_node(node);
                node_map.insert(id, node);
            }
            None => continue,
        };

        let relation: Relation = match row.get::<Relation>("edge") {
            Some(relation) => relation,
            None => continue,
        };
        relations.push(relation);
    }

    for relation in relations.iter() {
        let start_node_id = relation.start_node_id();
        let end_node_id = relation.end_node_id();
        let start_node = node_map.get(&start_node_id).unwrap();
        let end_node = node_map.get(&end_node_id).unwrap();
        let edge = EdgeData::from_neo_edge(relation, start_node, end_node);
        edges.push(edge);
    }

    let nodes: Vec<NodeData> = node_map.into_values().collect();
    info!("Number of nodes: {}", &nodes.len());
    info!("Number of edges: {}", &edges.len());

    Ok((nodes, edges))
}

/// Query the graph database to get the nodes and edges between two nodes.
///
/// # Arguments
/// * `graph` - The graph database connection.
/// * `start_node_id` - The start node id. Such as 'Compound::DrugBank:DB00818'
/// * `end_node_id` - The end node id. Such as 'Disease::MONDO:0005404'
/// * `nhops` - The number of hops between the start node and the end node.
///
/// # Returns
/// * `Ok((nodes, edges))` - The nodes and edges between the start node and the end node.
/// * `Err(e)` - The error message.
pub async fn query_nhops(
    graph: &Graph,
    start_node_id: &str, // Such as 'Compound::DrugBank:DB00818'
    end_node_id: &str,   // Such as 'Disease::MONDO:0005404'
    nhops: usize,
) -> Result<(Vec<NodeData>, Vec<EdgeData>), anyhow::Error> {
    let (start_node_type, start_node_id) = split_id(start_node_id)?;
    let (end_node_type, end_node_id) = split_id(end_node_id)?;
    let query_str = gen_nhops_query_str(
        &start_node_type,
        &start_node_id,
        &end_node_type,
        &end_node_id,
        nhops,
    );

    let mut result = graph.execute(query(&query_str)).await?;
    let r = parse_nhops_results(&mut result).await?;
    Ok(r)
}

// Parse the shared nodes and edges from the result.
// NOTE: the name of the results should be 'common', 'relatedNodes', and 'relations'.
//
// # Arguments
// * `result` - The result of the query.
//
// # Returns
// * `Ok((nodes, edges))` - The nodes and edges between the start node and the end node.
// * `Err(e)` - The error message.
async fn parse_shared_results(
    result: &mut RowStream,
) -> Result<(Vec<NodeData>, Vec<EdgeData>), anyhow::Error> {
    let mut node_map: HashMap<i64, NodeData> = HashMap::new();
    let mut edges = Vec::new();
    let mut relations = Vec::new();

    while let Some(row) = result.next().await? {
        match row.get::<NeoNode>("common") {
            Some(node) => {
                let id = node.id();
                let node = NodeData::from_neo_node(node);
                node_map.insert(id, node);
            }
            None => (),
        };

        match row.get::<Vec<NeoNode>>("relatedNodes") {
            Some(related_start_nodes) => {
                for node in related_start_nodes {
                    let id = node.id();
                    let node = NodeData::from_neo_node(node);
                    node_map.insert(id, node);
                }
            }
            None => (),
        };

        match row.get::<NeoNode>("startNode") {
            Some(node) => {
                let id = node.id();
                let node = NodeData::from_neo_node(node);
                node_map.insert(id, node);
            }
            None => (),
        };

        // If you want to know the format of the relations, you can run the related cypher query in the Neo4j Browser.
        match row.get::<Vec<Vec<Relation>>>("relations") {
            Some(relation) => {
                for r1 in relation {
                    for r2 in r1 {
                        relations.push(r2);
                    }
                }
            }
            None => (),
        };

        match row.get::<Vec<Relation>>("relations1") {
            Some(relation) => {
                for r in relation {
                    relations.push(r);
                }
            }
            None => (),
        };

        match row.get::<Vec<Relation>>("relations2") {
            Some(relation) => {
                for r in relation {
                    relations.push(r);
                }
            }
            None => (),
        };
    }
    info!("Number of entities: {}", &node_map.len());
    info!("Number of relations: {}", &relations.len());

    for relation in relations.iter() {
        let start_node_id = relation.start_node_id();
        let end_node_id = relation.end_node_id();
        let start_node = node_map.get(&start_node_id).unwrap();
        let end_node = node_map.get(&end_node_id).unwrap();
        let edge = EdgeData::from_neo_edge(relation, start_node, end_node);
        edges.push(edge);
    }

    let nodes: Vec<NodeData> = node_map.into_values().collect();
    info!("Number of nodes: {}", &nodes.len());
    info!("Number of edges: {}", &edges.len());

    Ok((nodes, edges))
}

// Query the graph database to get the shared shared nodes between the start nodes.
//
// # Arguments
// * `graph` - The graph database connection.
// * `node_ids` - The start node ids. Such as ['Compound::DrugBank:DB00818', 'Disease::MONDO:0005404']
// * `target_node_types` - The target node types. Such as ['Disease']
// * `nhops` - The number of hops between the start node and the end node.
// * `topk` - The number of top k shared nodes.
// * `nums_shared_by` - The number of nodes shared by.
//
// # Returns
// * `Ok((nodes, edges))` - The nodes and edges between the start node and the end node.
// * `Err(e)` - The error message.
pub async fn query_shared_nodes(
    graph: &Graph,
    node_ids: &Vec<&str>,
    target_node_types: Option<Vec<&str>>,
    nhops: usize,
    topk: usize,
    nums_shared_by: usize,
    start_node_id: Option<&str>,
) -> Result<(Vec<NodeData>, Vec<EdgeData>), anyhow::Error> {
    if nums_shared_by > node_ids.len() {
        return Err(anyhow::anyhow!(
            "The number of shared nodes should be less than the number of start nodes."
        ));
    };

    if nhops > 2 {
        return Err(anyhow::anyhow!("The number of hops should be less than 3."));
    };

    let query_str = if start_node_id.is_none() {
        // Example query string:
        // WITH ['MONDO:0100233', 'MONDO:0005404'] AS diseaseIds
        // UNWIND diseaseIds AS diseaseId
        // MATCH (start:Disease) WHERE start.id = diseaseId
        // WITH COLLECT(DISTINCT start) AS startNodes
        // UNWIND startNodes AS startNode
        // MATCH p=(startNode)-[r*1]-(common:Disease)
        // WHERE NOT startNode = common AND ALL(x IN nodes(p) WHERE x IN startNodes OR x = common)
        // WITH common, COLLECT(DISTINCT startNode) AS relatedNodes, COLLECT(DISTINCT r) AS relations, COUNT(DISTINCT startNode) AS sharedBy
        // WHERE sharedBy = 2
        // RETURN common, relatedNodes, relations
        // ORDER BY sharedBy DESC
        // LIMIT 100

        // Build the startNodesDetails string
        let mut start_nodes_details = String::new();
        for (i, node_id) in node_ids.iter().enumerate() {
            let (node_type, node_id) = split_id(node_id)?;
            start_nodes_details.push_str(&format!("{{label: '{}', id: '{}'}}", node_type, node_id));
            if i < node_ids.len() - 1 {
                start_nodes_details.push_str(", ");
            }
        }
        let nums_shared_by = if nums_shared_by == 0 || nums_shared_by > node_ids.len() {
            node_ids.len()
        } else {
            nums_shared_by
        };

        let where_clauses = match target_node_types {
            Some(target_node_types) => {
                format!(
                    "sharedBy = {} AND ANY(label IN labels(common) WHERE label IN ['{}'])",
                    nums_shared_by,
                    target_node_types.join("', '")
                )
            }
            None => format!("sharedBy = {}", nums_shared_by),
        };

        if nhops == 1 {
            format!("
                WITH [{start_nodes_details}] AS startNodesDetails
                UNWIND startNodesDetails AS nodeDetails
                MATCH (start)
                WHERE start.id = nodeDetails.id AND ANY(label IN labels(start) WHERE label = nodeDetails.label)
                WITH COLLECT(DISTINCT start) AS startNodes
                UNWIND startNodes AS startNode
                MATCH p=(startNode)-[r]-(common)
                WHERE (NOT startNode = common) AND ALL(x IN nodes(p) WHERE x IN startNodes) AND startNode IN startNodes
                WITH common, COLLECT(DISTINCT startNode) AS relatedNodes, COLLECT(DISTINCT r) AS relations, COUNT(DISTINCT startNode) AS sharedBy
                WHERE {where_clauses}
                WITH common, relatedNodes, relations, sharedBy
                ORDER BY sharedBy DESC
                LIMIT {topk}
                RETURN common, relatedNodes, relations",
                topk = topk,
                start_nodes_details = start_nodes_details,
                where_clauses = where_clauses
            )
        } else {
            format!("
                WITH [{start_nodes_details}] AS startNodesDetails
                UNWIND startNodesDetails AS nodeDetails
                MATCH (start)
                WHERE start.id = nodeDetails.id AND ANY(label IN labels(start) WHERE label = nodeDetails.label)
                WITH COLLECT(DISTINCT start) AS startNodes
                UNWIND startNodes AS startNode
                MATCH p=(startNode)-[r*1..2]-(common)
                WHERE NOT startNode = common AND ALL(x IN nodes(p) WHERE x IN startNodes OR x = common) AND startNode IN startNodes
                WITH common, COLLECT(DISTINCT startNode) AS relatedNodes, COLLECT(DISTINCT r) AS relations, COUNT(DISTINCT startNode) AS sharedBy
                WHERE {where_clauses}
                WITH common, relatedNodes, relations, sharedBy
                ORDER BY sharedBy DESC
                LIMIT {topk}
                RETURN common, relatedNodes, relations",
                topk = topk,
                start_nodes_details = start_nodes_details,
                where_clauses = where_clauses
            )
        }
    } else {
        let start_node_id = start_node_id.unwrap();
        let (start_node_label, start_node_id) = split_id(start_node_id)?;
        let other_nodes_details = node_ids
            .iter()
            .filter(|id| *id != &start_node_id)
            .map(|id| {
                let (label, id) = split_id(id).unwrap();
                format!("{{label: '{}', id: '{}'}}", label, id)
            })
            .collect::<Vec<String>>()
            .join(", ");

        if nhops == 1 {
            format!("
                MATCH (start)
                WHERE start.id = '{start_node_id}' AND ANY(label IN labels(start) WHERE label = '{start_node_label}')
                WITH start AS startNode
                UNWIND [{other_nodes_details}] AS otherNodeDetails
                MATCH (other)
                WHERE other.id = otherNodeDetails.id AND ANY(label IN labels(other) WHERE label = otherNodeDetails.label)
                WITH COLLECT(DISTINCT other) AS otherNodes, startNode
                UNWIND otherNodes AS otherNode
                MATCH (startNode)-[r]-(sharedNode)
                WHERE sharedNode = otherNode
                WITH sharedNode, COLLECT(r) AS relations1, startNode, COLLECT(DISTINCT otherNode) AS relatedNodes
                RETURN sharedNode AS common, relations1, relatedNodes, startNode",
                other_nodes_details = other_nodes_details,
                start_node_id = start_node_id,
                start_node_label = start_node_label
            )
        } else {
            let where_clauses = match target_node_types {
                Some(target_node_types) => {
                    format!(
                        "sharedBy = {} AND ANY(label IN labels(common) WHERE label IN ['{}'])",
                        nums_shared_by,
                        target_node_types.join("', '")
                    )
                }
                None => format!("sharedBy = {}", nums_shared_by),
            };

            // Example query string:
            // MATCH (startNode)
            // WHERE startNode.id = 'ENTREZ:3569' AND ANY(label IN labels(startNode) WHERE label = 'Gene')
            // WITH startNode
            // UNWIND [{label: 'Gene', id: 'ENTREZ:3107'}, {label: 'Gene', id: 'ENTREZ:3119'}, {label: 'Gene', id: 'ENTREZ:3569'}] AS otherNodeDetail
            // MATCH (otherNode)
            // WHERE otherNode.id = otherNodeDetail.id AND ANY(label IN labels(otherNode) WHERE label = otherNodeDetail.label)
            // WITH COLLECT(DISTINCT otherNode) AS otherNodes, startNode
            // MATCH (startNode)-[r1]-(sharedNode)
            // WHERE NOT (sharedNode IN otherNodes)
            // WITH sharedNode, COLLECT(DISTINCT r1) AS relations, startNode, otherNodes
            // MATCH (sharedNode)-[r2]-(otherNode)
            // WHERE otherNode IN otherNodes AND otherNode <> startNode
            // WITH sharedNode, relations, startNode, COLLECT(DISTINCT otherNode) AS relatedNodes, COLLECT(DISTINCT r2) AS relations2, COUNT(DISTINCT otherNode) AS sharedBy
            // WHERE sharedBy = 2 AND ANY(label IN labels(sharedNode) WHERE label IN ['Gene'])
            // WITH sharedNode AS common, relations AS relations1, relatedNodes, startNode, relations2, sharedBy
            // ORDER BY sharedBy DESC
            // LIMIT 3
            // RETURN common, relations1, relatedNodes, startNode, relations2
            format!("
                MATCH (startNode)
                WHERE startNode.id = '{start_node_id}' AND ANY(label IN labels(startNode) WHERE label = '{start_node_label}')
                WITH startNode
                UNWIND [{other_nodes_details}] AS otherNodeDetail
                MATCH (otherNode)
                WHERE otherNode.id = otherNodeDetail.id AND ANY(label IN labels(otherNode) WHERE label = otherNodeDetail.label)
                WITH COLLECT(DISTINCT otherNode) AS otherNodes, startNode
                MATCH (startNode)-[r1]-(sharedNode)
                WHERE NOT (sharedNode IN otherNodes)
                WITH sharedNode, COLLECT(DISTINCT r1) AS relations, startNode, otherNodes
                MATCH (sharedNode)-[r2]-(otherNode)
                WHERE otherNode IN otherNodes AND otherNode <> startNode
                WITH sharedNode AS common, relations, startNode, COLLECT(DISTINCT otherNode) AS relatedNodes, COLLECT(DISTINCT r2) AS relations2, COUNT(DISTINCT otherNode) AS sharedBy
                WHERE {where_clauses}
                WITH common, relations AS relations1, relatedNodes, startNode, relations2, sharedBy
                ORDER BY sharedBy DESC
                LIMIT {topk}
                RETURN common, relations1, relatedNodes, startNode, relations2",
                other_nodes_details = other_nodes_details,
                where_clauses = where_clauses,
                start_node_id = start_node_id,
                start_node_label = start_node_label,
                topk = topk
            )
        }
    };

    info!("query_shared_nodes's query_str: {}", query_str);
    let mut result = graph.execute(query(&query_str)).await?;
    let r = parse_shared_results(&mut result).await?;

    Ok(r)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connect_graph_db;
    use log::{debug, error, info};
    use std::env;
    use tokio::test as async_test;

    #[test]
    fn test_split_id() {
        let id = "Compound::DrugBank:DB00818";
        let (start_node_type, start_node_id) = split_id(id).unwrap();
        assert_eq!(start_node_type, "Compound");
        assert_eq!(start_node_id, "DrugBank:DB00818");
    }

    #[test]
    fn test_gen_nhops_query_str() {
        let start_node_type = "Compound";
        let start_node_id = "DrugBank:DB00818";
        let end_node_type = "Disease";
        let end_node_id = "MONDO:0005404";
        let nhops = 2;
        let query_str = gen_nhops_query_str(
            start_node_type,
            start_node_id,
            end_node_type,
            end_node_id,
            nhops,
        );
        assert_eq!(
            query_str,
            "MATCH path = (n:Compound)-[r*..2]-(m:Disease) WHERE n.id IN ['DrugBank:DB00818'] AND m.id IN ['MONDO:0005404'] UNWIND nodes(path) AS node UNWIND relationships(path) AS edge RETURN DISTINCT node, edge"
        );
    }

    #[async_test]
    async fn test_query_neo4j() {
        // 从环境变量中获取数据库连接字符串
        let neo4j_url =
            env::var("NEO4J_URL").unwrap_or("neo4j://neo4j:password@localhost:7687".to_string());

        let graph = connect_graph_db(&neo4j_url).await;
        match query_nhops(
            &graph,
            "Compound::DrugBank:DB00818",
            "Disease::MONDO:0005404",
            2,
        )
        .await
        {
            Ok((nodes, edges)) => {
                // 进行测试断言
                debug!("nodes: {:?}", nodes);
                info!("Number of nodes: {}", nodes.len());
                assert!(!nodes.is_empty());
                debug!("edges: {:?}", edges);
                info!("Number of edges: {}", edges.len());
                assert!(!edges.is_empty());
            }
            Err(e) => panic!("Query failed: {}", e),
        }
    }
}
