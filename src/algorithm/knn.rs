//! KNN algorithm for finding nearest neighbours

use crate::{model::core::EntityEmbedding, pgvector::Vector};
use kiddo::distance::squared_euclidean;
use kiddo::KdTree;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Entity {
    pub embedding_id: i64,
    pub entity_id: String,
    pub entity_name: String,
    pub entity_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Neighbour {
    pub source: Entity,
    pub target: Entity,
    pub distance: f32,
}

// The length of the embedding vector, if your embedding is 100 dimensional, then LEN = 100
// You may need to change this value depending on your model
const LEN: usize = 3;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NodeSimilarity {
    pub source: EntityEmbedding,
    pub targets: Vec<EntityEmbedding>,
}

impl NodeSimilarity {
    pub fn new(source: EntityEmbedding, targets: Vec<EntityEmbedding>) -> Self {
        NodeSimilarity { source, targets }
    }

    fn vec2array(vec: &Vector) -> [f32; LEN] {
        let mut arr: [f32; LEN] = [0.0; LEN];
        for (i, v) in vec.to_vec().iter().enumerate() {
            arr[i] = *v;
        }
        arr
    }

    pub fn get_neighbours(&self, k: usize) -> Vec<Neighbour> {
        let mut tree: KdTree<f32, LEN> = KdTree::with_capacity(self.targets.len());
        for (index, target) in self.targets.iter().enumerate() {
            let arr: [f32; LEN] = Self::vec2array(&target.embedding);

            tree.add(&arr, index);
        }

        let source_arr: [f32; LEN] = Self::vec2array(&self.source.embedding);
        let neighbours = tree.nearest_n(&source_arr, k, &squared_euclidean);
        let mut result: Vec<Neighbour> = Vec::new();
        for neighbour in neighbours {
            let target = &self.targets[neighbour.item];
            let neighbour = Neighbour {
                source: Entity {
                    embedding_id: self.source.embedding_id,
                    entity_id: self.source.entity_id.clone(),
                    entity_name: self.source.entity_name.clone(),
                    entity_type: self.source.entity_type.clone(),
                },
                target: Entity {
                    embedding_id: target.embedding_id,
                    entity_id: target.entity_id.clone(),
                    entity_name: target.entity_name.clone(),
                    entity_type: target.entity_type.clone(),
                },
                distance: neighbour.distance,
            };

            result.push(neighbour);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    extern crate log;
    extern crate stderrlog;
    use super::*;
    use crate::model::core::{EmbeddingRecordResponse, EntityEmbedding};
    use crate::{import_data, init_log, run_migrations};

    // Setup the test database
    async fn setup_test_db() -> sqlx::PgPool {
        init_log();
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

    #[tokio::test]
    async fn test_get_neighbours() {
        let pool = setup_test_db().await;

        match EmbeddingRecordResponse::<EntityEmbedding>::get_records(
            &pool,
            "biomedgps_entity_embedding",
            &None,
            Some(1),
            Some(10),
            None,
        )
        .await
        {
            Ok(records) => {
                assert!(records.records.len() > 0);
                println!("records: {:?}", records);
            }
            Err(e) => {
                println!("{}", e);
                assert!(false);
            }
        }
    }

    #[test]
    fn test_knn() {
        let source = EntityEmbedding::new(1, "MESH:C0001", "source1", "Gene", &vec![1.0, 2.0, 3.0]);

        let targets = vec![
            EntityEmbedding::new(1, "MESH:C0002", "target1", "Gene", &vec![0.1, 0.2, 0.3]),
            EntityEmbedding::new(1, "MESH:C0003", "target2", "Gene", &vec![0.4, 0.5, 0.6]),
            EntityEmbedding::new(1, "MESH:C0004", "target3", "Gene", &vec![1.1, 1.2, 1.3]),
            EntityEmbedding::new(1, "MESH:C0005", "target4", "Gene", &vec![2.1, 2.2, 2.3]),
        ];

        let knn = NodeSimilarity::new(source, targets);

        let neighbours = knn.get_neighbours(3);

        assert_eq!(neighbours.len(), 3);
        // Get all distances
        let distances: Vec<f32> = neighbours.iter().map(|n| n.distance).collect();
        assert_eq!(distances, vec![1.7399998, 3.54, 8.370001]);
    }
}
