-- Example 1: given a compound, find the 5 closest compounds based on the embedding and their distance
SELECT COALESCE(entity_type, '') || '::' || COALESCE(entity_id, '') AS node_id,
        embedding <-> (SELECT embedding FROM biomedgps_entity_embedding
                            WHERE COALESCE(entity_type, '') || '::' || COALESCE(entity_id, '') = 'Compound::MESH:C000601183') AS distance
FROM biomedgps_entity_embedding
WHERE entity_type = 'Compound' AND COALESCE(entity_type, '') || '::' || COALESCE(entity_id, '') <> 'Compound::MESH:C000601183'
ORDER BY distance ASC
LIMIT 5;

-- Example 2: given a gene, find the 5 closest genes based on the embedding and their transe score
SELECT
    ee1.entity_id AS head,
    rte.relation_type AS relation_type,
    ee2.entity_id AS tail,
	pgml.transe_l2_ndarray(
			vector_to_float4(ee1.embedding, 400, false),
			vector_to_float4(rte.embedding, 400, false),
			vector_to_float4(ee2.embedding, 400, false),
			12.0,
			true
	) AS score	
FROM
    biomedgps_entity_embedding ee1,
    biomedgps_relation_embedding rte,
    biomedgps_entity_embedding ee2
WHERE
    ee1.entity_id = 'ENTREZ:6747'
    AND rte.relation_type = 'STRING::BINDING::Gene:Gene'
GROUP BY
    ee1.embedding_id,
    rte.embedding_id,
    ee2.embedding_id
ORDER BY score DESC
LIMIT 10