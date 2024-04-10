SELECT
    r.id,
    r.source_id,
    e1.name AS source_name,
    r.source_type,
    r.target_id,
    e2.name AS target_name,
    r.target_type,
    r.relation_type,
    r.formatted_relation_type,
    r.key_sentence,
    r.resource,
    r.dataset,
    r.pmids,
    r.score
FROM
    public.biomedgps_relation_with_score r
LEFT JOIN public.biomedgps_entity e1 ON r.source_id = e1.id AND r.source_type = e1.label
LEFT JOIN public.biomedgps_entity e2 ON r.target_id = e2.id AND r.target_type = e2.label;