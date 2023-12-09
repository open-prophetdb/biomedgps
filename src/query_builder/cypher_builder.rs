use crate::model::graph::{EdgeData, NodeData, COMPOSED_ENTITY_DELIMITER, COMPOSED_ENTITY_REGEX};
use neo4rs::{query, Graph, Node as NeoNode, Relation};
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
    let query_str = format!(
        "MATCH path = (n:{})-[r*..{}]-(m:{}) WHERE n.id IN ['{}'] AND m.id IN ['{}'] UNWIND nodes(path) AS node UNWIND relationships(path) AS edge RETURN DISTINCT node, edge",
        start_node_type,
        nhops,
        end_node_type,
        start_node_id,
        end_node_id,
    );

    query_str
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

        let relation = match row.get::<Relation>("edge") {
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

    let nodes = node_map.into_values().collect();

    Ok((nodes, edges))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_db_url;
    use log::{debug, error, info};
    use neo4rs::{ConfigBuilder, Graph};
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
        // Get host, username and password from neo4j_url. the neo4j_url format is neo4j://<username>:<password>@<host>:<port>
        let mut host = "".to_string();
        let mut username = "".to_string();
        let mut password = "".to_string();
        if neo4j_url.starts_with("neo4j://") {
            let (hostname, port, user, pass) = parse_db_url(&neo4j_url);
            host = format!("{}:{}", hostname, port);
            username = user;
            password = pass;
        } else {
            error!("Invalid neo4j_url: {}", neo4j_url);
            std::process::exit(1);
        };

        if host.is_empty() || username.is_empty() {
            debug!("Invalid neo4j_url: {}", neo4j_url);
            std::process::exit(1);
        };

        let graph = Graph::connect(
            ConfigBuilder::default()
                .uri(host)
                .user(username)
                .password(password)
                .build()
                .unwrap(),
        )
        .await
        .unwrap();

        match query_nhops(&graph, "Compound::DrugBank:DB00818", "Disease::MONDO:0005404", 2).await {
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
