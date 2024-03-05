declare namespace API {
  type askLLMParams = {
    prompt_template_id: string;
  };

  type Context = {
    entity?: Entity;
    expanded_relation?: ExpandedRelation;
    symptoms_with_disease_ctx?: SymptomsWithDiseaseCtx;
  };

  type deleteCuratedKnowledgeParams = {
    id: number;
  };

  type deleteSubgraphParams = {
    id: string;
  };

  type Edge = {
    relid: string;
    source: string;
    category: string;
    target: string;
    reltype: string;
    style: EdgeStyle;
    data: EdgeData;
  };

  type EdgeData = {
    relation_type: string;
    source_id: string;
    source_type: string;
    target_id: string;
    target_type: string;
    score: number;
    key_sentence: string;
    resource: string;
    pmids: string;
    dataset: string;
  };

  type EdgeKeyShape = {
    lineDash: number[];
    stroke: string;
    lineWidth: number;
  };

  type EdgeLabel = {
    value: string;
  };

  type EdgeStyle = {
    label: EdgeLabel;
    keyshape?: EdgeKeyShape;
  };

  type Entity = {
    idx: number;
    id: string;
    name: string;
    label: string;
    resource: string;
    description?: string;
    taxid?: string;
    synonyms?: string;
    pmids?: string;
    xrefs?: string;
  };

  type Entity2D = {
    embedding_id: number;
    entity_id: string;
    entity_type: string;
    entity_name: string;
    umap_x: number;
    umap_y: number;
    tsne_x: number;
    tsne_y: number;
  };

  type EntityMetadata = {
    id: number;
    resource: string;
    entity_type: string;
    entity_count: number;
  };

  type ErrorMessage = {
    msg: string;
  };

  type ExpandedRelation = {
    relation: Relation;
    source: Entity;
    target: Entity;
  };

  type fetchCuratedGraphParams = {
    curator: string;
    project_id?: string;
    organization_id?: string;
    page?: number;
    page_size?: number;
    strict_mode: boolean;
  };

  type fetchCuratedKnowledgesByOwnerParams = {
    curator: string;
    project_id?: string;
    organization_id?: string;
    page?: number;
    page_size?: number;
  };

  type fetchCuratedKnowledgesParams = {
    page?: number;
    page_size?: number;
    query_str?: string;
  };

  type fetchEdgesAutoConnectNodesParams = {
    node_ids: string;
  };

  type fetchEntitiesParams = {
    page?: number;
    page_size?: number;
    query_str?: string;
    model_table_prefix?: string;
  };

  type fetchEntity2dParams = {
    page?: number;
    page_size?: number;
    query_str?: string;
  };

  type fetchNodesParams = {
    node_ids: string;
  };

  type fetchOneStepLinkedNodesParams = {
    page?: number;
    page_size?: number;
    query_str?: string;
  };

  type fetchPathsParams = {
    start_node_id: string;
    end_node_id: string;
    nhops?: number;
  };

  type fetchPredictedNodesParams = {
    node_id: string;
    relation_type: string;
    query_str?: string;
    topk?: number;
    model_name?: string;
  };

  type fetchRelationCountsParams = {
    query_str?: string;
  };

  type fetchRelationsParams = {
    page?: number;
    page_size?: number;
    query_str?: string;
  };

  type fetchSharedNodesParams = {
    node_ids: string;
    target_node_types?: string;
    topk?: number;
    nhops?: number;
    nums_shared_by?: number;
  };

  type fetchSubgraphsParams = {
    page?: number;
    page_size?: number;
    query_str?: string;
  };

  type Graph = {
    nodes: Node[];
    edges: Edge[];
  };

  type Icon = {
    type: string;
    value: string;
    fill: string;
    size: number;
    color: string;
  };

  type KnowledgeCuration = {
    id: number;
    relation_type: string;
    source_name: string;
    source_type: string;
    source_id: string;
    target_name: string;
    target_type: string;
    target_id: string;
    key_sentence: string;
    created_at: string;
    curator: string;
    pmid: number;
    payload?: any;
  };

  type Label = {
    value: string;
    fill: string;
    fontSize: number;
    offset: number;
    position: string;
  };

  type LlmResponse = {
    prompt: string;
    response: string;
    created_at: string;
  };

  type Node = {
    comboId?: string;
    id: string;
    label: string;
    nlabel: string;
    degree?: number;
    style: NodeStyle;
    category: string;
    cluster?: string;
    type: string;
    x?: number;
    y?: number;
    data: NodeData;
  };

  type NodeData = {
    identity: string;
    id: string;
    label: string;
    name: string;
    description?: string;
    resource: string;
    xrefs?: string;
    pmids?: string;
    taxid?: string;
    synonyms?: string;
  };

  type NodeKeyShape = {
    fill: string;
    stroke: string;
    opacity: number;
    fillOpacity: number;
  };

  type NodeStyle = {
    label: Label;
    keyshape: NodeKeyShape;
    icon: Icon;
  };

  type putCuratedKnowledgeParams = {
    id: number;
  };

  type putSubgraphParams = {
    id: string;
  };

  type RecordResponseEntity = {
    /** data */
    records: Entity[];
    /** total num */
    total: number;
    /** current page index */
    page: number;
    /** default 10 */
    page_size: number;
  };

  type RecordResponseEntity2D = {
    /** data */
    records: Entity2D[];
    /** total num */
    total: number;
    /** current page index */
    page: number;
    /** default 10 */
    page_size: number;
  };

  type RecordResponseKnowledgeCuration = {
    /** data */
    records: KnowledgeCuration[];
    /** total num */
    total: number;
    /** current page index */
    page: number;
    /** default 10 */
    page_size: number;
  };

  type RecordResponseRelation = {
    /** data */
    records: Relation[];
    /** total num */
    total: number;
    /** current page index */
    page: number;
    /** default 10 */
    page_size: number;
  };

  type RecordResponseSubgraph = {
    /** data */
    records: Subgraph[];
    /** total num */
    total: number;
    /** current page index */
    page: number;
    /** default 10 */
    page_size: number;
  };

  type Relation = {
    id: number;
    relation_type: string;
    formatted_relation_type?: string;
    source_id: string;
    source_type: string;
    target_id: string;
    target_type: string;
    score?: number;
    key_sentence?: string;
    resource: string;
    dataset?: string;
    pmids?: string;
  };

  type RelationCount = {
    relation_type: string;
    target_type: string;
    source_type: string;
    resource: string;
    ncount: number;
  };

  type RelationMetadata = {
    id: number;
    resource: string;
    dataset: string;
    relation_type: string;
    formatted_relation_type: string;
    relation_count: number;
    start_entity_type: string;
    end_entity_type: string;
    description?: string;
  };

  type Statistics = {
    entity_stat: EntityMetadata[];
    relation_stat: RelationMetadata[];
  };

  type Subgraph = {
    id: string;
    name: string;
    description?: string;
    payload: string;
    created_time: string;
    owner: string;
    version: string;
    db_version: string;
    parent?: string;
  };

  type SymptomsWithDiseaseCtx = {
    disease_name: string;
    subgraph: string;
    symptoms: string[];
  };
}
