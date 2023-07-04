//! KNN algorithm for finding nearest neighbours

use crate::model::core::EntityEmbedding;
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

    fn vec2array(vec: &Vec<f32>) -> [f32; LEN] {
        let mut arr: [f32; LEN] = [0.0; LEN];
        for (i, v) in vec.iter().enumerate() {
            arr[i] = *v;
        }
        arr
    }

    pub fn get_neighbours(&self, k: usize) -> Vec<Neighbour> {
        let mut tree: KdTree<f32, LEN> = KdTree::with_capacity(self.targets.len());
        for (index, target) in self.targets.iter().enumerate() {
            let arr: [f32; LEN] = Self::vec2array(&target.embedding_array);

            tree.add(&arr, index);
        }

        let source_arr: [f32; LEN] = Self::vec2array(&self.source.embedding_array);
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
        };

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::core::EntityEmbedding;

    #[test]
    fn test_knn() {
        let source = EntityEmbedding {
            embedding_id: 1,
            entity_id: "MESH:C0001".to_string(),
            entity_name: "source1".to_string(),
            entity_type: "Gene".to_string(),
            embedding_array: vec![1.0, 2.0, 3.0],
        };

        let targets = vec![
            EntityEmbedding {
                embedding_id: 1,
                entity_id: "MESH:C0002".to_string(),
                entity_name: "target1".to_string(),
                entity_type: "Gene".to_string(),
                embedding_array: vec![0.1, 0.2, 0.3],
            },
            EntityEmbedding {
                embedding_id: 1,
                entity_id: "MESH:C0003".to_string(),
                entity_name: "target2".to_string(),
                entity_type: "Gene".to_string(),
                embedding_array: vec![0.4, 0.5, 0.6],
            },
            EntityEmbedding {
                embedding_id: 1,
                entity_id: "MESH:C0004".to_string(),
                entity_name: "target3".to_string(),
                entity_type: "Gene".to_string(),
                embedding_array: vec![1.1, 1.2, 1.3],
            },
            EntityEmbedding {
                embedding_id: 1,
                entity_id: "MESH:C0005".to_string(),
                entity_name: "target4".to_string(),
                entity_type: "Gene".to_string(),
                embedding_array: vec![2.1, 2.2, 2.3],
            },
        ];

        let knn = NodeSimilarity::new(source, targets);

        let neighbours = knn.get_neighbours(3);

        assert_eq!(neighbours.len(), 3);
        // Get all distances
        let distances: Vec<f32> = neighbours.iter().map(|n| n.distance).collect();
        assert_eq!(distances, vec![1.7399998, 3.54, 8.370001]);
    }
}
