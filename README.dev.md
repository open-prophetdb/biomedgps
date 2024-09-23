## How to publish a new version of the docker image

1. Update the version in `studio/package.json` and `Cargo.toml`, you need to keep the version in sync.
2. Commit the changes with the message `Release vX.Y.Z` and tag the commit with the same version (e.g. `git tag vX.Y.Z`).
3. Push the commit and the tag to the repository.
4. You can get the docker image from the [GitHub Container Registry](https://github.com/orgs/open-prophetdb/packages?repo_name=biomedgps) or from the command line with `docker pull ghcr.io/open-prophetdb/biomedgps:vX.Y.Z`.


## FAQs

1. ModuleNotFoundError: No module named 'InstructorEmbedding'

For some reason, the InstructorEmbedding module did not get installed. You can manually install it using the following command:

```bash
docker exec -it xxx bash
source /var/lib/postgresml-python/pgml-venv/bin/activate
pip install InstructorEmbedding
```

NOTE: After you've installed the module, you don't need to restart the docker container, just need to launch a new postgresql session.

2. Examples

```SQL
CREATE TEMPORARY TABLE key_sentence_embeddings AS
SELECT id AS document_id,
       key_sentence AS key_sentence,
       pgml.embed('intfloat/e5-small-v2', key_sentence) AS embedding
FROM biomedgps_key_sentence_curation;
```

```SQL
WITH query AS (
    SELECT pgml.embed('intfloat/e5-small-v2', 'What is the relationship between BDNF and 7,8-DHF?') AS embedding
)
SELECT kse.document_id,
       kse.key_sentence,
       pgml.distance_l2(query.embedding, kse.embedding) AS distance
FROM key_sentence_embeddings kse, query
ORDER BY distance
```

```SQL
INSERT INTO biomedgps_text_embedding (
    text, 
    text_source_type, 
    text_source_id, 
    text_source_field, 
    payload, 
    owner, 
    groups, 
    model_name
)
SELECT 
    key_sentence AS text, 
    'key_sentence' AS text_source_type, 
    id AS text_source_id, 
    'key_sentence' AS text_source_field, 
    '{}'::jsonb AS payload, 
    curator AS owner, 
    ARRAY[]::VARCHAR[] AS groups, 
    'intfloat/e5-small-v2' AS model_name
FROM 
    biomedgps_key_sentence_curation;
```

```SQL
SELECT
  *,
  pgml.distance_l2(
    embedding,
    pgml.embed(
      'intfloat/e5-small-v2',
      'Whats the relationship between BDNF and ME/CFS?'
    )
  ) AS distance
FROM
  biomedgps_text_embedding
WHERE
  text_source_type = 'key_sentence'
  AND owner = 'yjcyxky@163.com'
ORDER BY
  distance ASC
LIMIT
  10;
```
