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

-- Example 3: create a table with the embeddings of the compound-disease-symptom triples
WITH compound_disease AS (
  SELECT
    id,
    source_id AS compound_id,
    target_id AS disease_id,
    relation_type AS compound_disease_relation_type
  FROM public.biomedgps_relation
  WHERE relation_type = 'DRUGBANK::treats::Compound:Disease'
),
disease_symptom AS (
  SELECT
    id,
    source_id AS disease_id,
    target_id AS symptom_id,
    relation_type AS disease_symptom_relation_type
  FROM public.biomedgps_relation
  WHERE relation_type = 'HSDN::has_symptom:Disease:Symptom'
),
combined AS (
  SELECT
    cd.compound_id,
    cd.disease_id,
    ds.symptom_id,
    cd.compound_disease_relation_type,
    ds.disease_symptom_relation_type
  FROM compound_disease cd
  JOIN disease_symptom ds ON cd.disease_id = ds.disease_id
),
embeddings AS (
  SELECT
    c.*,
    cd_emb.embedding AS compound_disease_embedding,
    ds_emb.embedding AS disease_symptom_embedding
  FROM combined c
  JOIN public.biomedgps_relation_embedding cd_emb ON c.compound_disease_relation_type = cd_emb.relation_type
  JOIN public.biomedgps_relation_embedding ds_emb ON c.disease_symptom_relation_type = ds_emb.relation_type
),
final_embeddings AS (
  SELECT
    e.*,
    ce.embedding AS compound_embedding,
    de.embedding AS disease_embedding,
    se.embedding AS symptom_embedding
  FROM embeddings e
  JOIN public.biomedgps_entity_embedding ce ON e.compound_id = ce.entity_id
  JOIN public.biomedgps_entity_embedding de ON e.disease_id = de.entity_id
  JOIN public.biomedgps_entity_embedding se ON e.symptom_id = se.entity_id
)
SELECT *
INTO TEMP TABLE temp_compound_disease_symptom_embeddings
FROM final_embeddings;

-- Example 4: 
SELECT
    compound_id AS source_id,
	'Compound' AS source_type,
	disease_id,
	symptom_id AS target_id,
	'Symptom' AS target_type,
    pgml.mean(ARRAY[pgml.transe_l2_ndarray(
		vector_to_float4(tt.compound_embedding, 400, false),
		vector_to_float4(tt.compound_disease_embedding, 400, false),
		vector_to_float4(tt.disease_embedding, 400, false),
		12,
		true,
		false
	),
	pgml.transe_l2_ndarray(
		vector_to_float4(tt.disease_embedding, 400, false),
		vector_to_float4(tt.disease_symptom_embedding, 400, false),
		vector_to_float4(tt.symptom_embedding, 400, false),
		12,
		true,
		false
	)]) AS score
FROM
    temp_compound_disease_symptom_embeddings tt
WHERE
    tt.symptom_id in ('MESH:D005221', 'MESH:D054972')
ORDER BY score DESC
LIMIT 10;

-- Example 5:
WITH score_calculations AS (
  SELECT
    compound_id,
    disease_id,
    symptom_id,
    'Compound' AS source_type,
    'Symptom' AS target_type,
    pgml.mean(ARRAY[
      pgml.transe_l2_ndarray(
        vector_to_float4(compound_embedding, 400, false),
        vector_to_float4(compound_disease_embedding, 400, false),
        vector_to_float4(disease_embedding, 400, false),
        12,
        true,
        false
      ),
      pgml.transe_l2_ndarray(
        vector_to_float4(disease_embedding, 400, false),
        vector_to_float4(disease_symptom_embedding, 400, false),
        vector_to_float4(symptom_embedding, 400, false),
        12,
        true,
        false
      )
    ]) AS score
  FROM
    temp_compound_disease_symptom_embeddings
  WHERE
    symptom_id IN ('MESH:D005221', 'MESH:D054972')
),
aggregated_data AS (
  SELECT
    compound_id AS source_id,
    source_type,
    symptom_id AS target_id,
    target_type,
    string_agg(disease_id, '|') AS disease_id,
    percentile_cont(0.5) WITHIN GROUP (ORDER BY score) AS median_score
  FROM
    score_calculations
  GROUP BY
    compound_id,
    symptom_id,
    source_type,
	  target_type
)
SELECT
  source_id,
  source_type,
  disease_id,
  target_id,
  target_type,
  median_score AS score
FROM
  aggregated_data
ORDER BY
  median_score DESC
LIMIT 10;
