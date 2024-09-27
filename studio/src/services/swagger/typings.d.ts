declare namespace swagger {
  type answerQuestionWithPublicationsParams = {
    question: string;
  };

  type Article = {
    ref_id: string;
    pubmed_id: string;
    citation: string;
  };

  type askLLMParams = {
    prompt_template_id: string;
  };

  type AtcCode = {
    code: string;
    level: AtcCodeLevel[];
  };

  type AtcCodeLevel = {
    code: string;
    text: string;
  };

  type Category = {
    category: string;
    mesh_id: string;
  };

  type Classification = {
    description: string;
    direct_parent: string;
    kingdom: string;
    superclass: string;
    class: string;
    subclass: string;
  };

  type CompoundAttr = {
    compound_type: string;
    created: string;
    updated: string;
    drugbank_id: string;
    xrefs: string[];
    name: string;
    description: string;
    cas_number: string;
    unii: string;
    compound_state: string;
    groups: string[];
    general_references?: GeneralReferences;
    synthesis_reference: string;
    indication: string;
    pharmacodynamics: string;
    mechanism_of_action: string;
    toxicity: string;
    metabolism: string;
    absorption: string;
    half_life: string;
    protein_binding: string;
    route_of_elimination: string;
    volume_of_distribution: string;
    clearance: string;
    classification?: Classification;
    synonyms: string[];
    products: Product[];
    packagers: Packager[];
    manufacturers: Manufacturer[];
    prices: Price[];
    categories: Category[];
    affected_organisms: string[];
    dosages: Dosage[];
    atc_codes: AtcCode[];
    patents: Patent[];
    food_interactions: string[];
    sequences: Sequence[];
    experimental_properties?: ExperimentalProperty;
    external_identifiers: ExternalIdentifier[];
    external_links: ExternalLink[];
    targets: Target[];
  };

  type Configuration = {
    id: number;
    config_name: string;
    config_title: string;
    config_description: string;
    category: string;
    owner: string;
  };

  type ConsensusResult = {
    results_analyzed_count: number;
    yes_percent: number;
    no_percent: number;
    possibly_percent: number;
    yes_doc_ids: string[];
    no_doc_ids: string[];
    possibly_doc_ids: string[];
    is_incomplete: boolean;
    is_disputed: boolean;
  };

  type Context = {
    entity?: Entity;
    expanded_relation?: ExpandedRelation;
    subgraph_with_ctx?: SubgraphWithCtx;
  };

  type Cost = {
    currency: string;
    text: string;
  };

  type deleteConfigurationParams = {
    config_name: string;
    category: string;
  };

  type deleteCuratedKnowledgeParams = {
    id: number;
  };

  type deleteEntityCurationParams = {
    id: number;
  };

  type deleteEntityCurationRecordParams = {
    fingerprint: string;
    curator: string;
    entity_id: string;
    entity_type: string;
    entity_name: string;
  };

  type deleteEntityMetadataCurationParams = {
    id: number;
  };

  type deleteEntityMetadataCurationRecordParams = {
    fingerprint: string;
    entity_id: string;
    entity_type: string;
    entity_name: string;
    field_name: string;
    field_value: string;
  };

  type deleteKeySentenceCurationByFingerprintParams = {
    fingerprint: string;
    key_sentence: string;
  };

  type deleteKeySentenceCurationParams = {
    id: number;
  };

  type deleteSubgraphParams = {
    id: string;
  };

  type deleteWebpageMetadataByFingerprintParams = {
    fingerprint: string;
  };

  type deleteWebpageMetadataParams = {
    id: number;
  };

  type Dosage = {
    form: string;
    route: string;
    strength: string;
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

  type Embedding = {
    text: string;
    embedding?: number[];
    created_time: string;
    model_name: string;
    text_source_type: string;
    text_source_field: string;
    text_source_id: string;
    payload?: any;
    owner: string;
    groups: string[];
    distance?: number;
  };

  type Entity = {
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

  type EntityAttr = {
    compounds?: EntityAttrRecordResponseCompoundAttr;
  };

  type EntityAttrRecordResponseCompoundAttr = {
    /** data */
    records: CompoundAttr[];
    /** total num */
    total: number;
    /** current page index */
    page: number;
    /** default 10 */
    page_size: number;
  };

  type EntityCuration = {
    id: number;
    entity_id: string;
    entity_type: string;
    entity_name: string;
    created_at: string;
    curator: string;
    fingerprint: string;
    payload?: any;
    annotation?: any;
  };

  type EntityMetadata = {
    id: number;
    resource: string;
    entity_type: string;
    entity_count: number;
  };

  type EntityMetadataCuration = {
    id: number;
    entity_id: string;
    entity_type: string;
    entity_name: string;
    field_name: string;
    field_value: string;
    field_title: string;
    key_sentence: string;
    created_at: string;
    curator: string;
    fingerprint: string;
    payload?: any;
    annotation?: any;
  };

  type ErrorMessage = {
    msg: string;
  };

  type ExpandedRelation = {
    relation: Relation;
    source: Entity;
    target: Entity;
  };

  type ExpandedTask = {
    task: Task;
    workflow: Workflow;
  };

  type ExperimentalProperty = {
    property: Property[];
  };

  type ExternalIdentifier = {
    resource: string;
    identifier: string;
  };

  type ExternalLink = {
    resource: string;
    url: string;
  };

  type fetchConfigurationsParams = {
    page?: number;
    page_size?: number;
    query_str?: string;
  };

  type fetchCuratedGraphParams = {
    curator?: string;
    project_id?: string;
    organization_id?: string;
    page?: number;
    page_size?: number;
    strict_mode: boolean;
  };

  type fetchCuratedKnowledgesByOwnerParams = {
    curator?: string;
    fingerprint?: string;
    project_id?: string;
    organization_id?: string;
    query_str?: string;
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

  type fetchEmbeddingsParams = {
    question: string;
    text_source_type: string;
    top_k: number;
  };

  type fetchEntitiesParams = {
    page?: number;
    page_size?: number;
    query_str?: string;
    model_table_prefix?: string;
  };

  type fetchEntity2DParams = {
    page?: number;
    page_size?: number;
    query_str?: string;
  };

  type fetchEntityAttributesParams = {
    query_str?: string;
    page?: number;
    page_size?: number;
    entity_type: string;
  };

  type fetchEntityCurationByOwnerParams = {
    fingerprint?: string;
    project_id?: string;
    organization_id?: string;
    page?: number;
    page_size?: number;
    query_str?: string;
  };

  type fetchEntityCurationParams = {
    page?: number;
    page_size?: number;
    query_str?: string;
  };

  type fetchEntityMetadataCurationByOwnerParams = {
    fingerprint?: string;
    project_id?: string;
    organization_id?: string;
    query_str?: string;
    page?: number;
    page_size?: number;
  };

  type fetchEntityMetadataCurationParams = {
    page?: number;
    page_size?: number;
    query_str?: string;
  };

  type fetchFileByFileNameParams = {
    task_id: string;
    file_name: string;
  };

  type fetchKeySentenceCurationByOwnerParams = {
    curator?: string;
    fingerprint?: string;
    project_id?: string;
    organization_id?: string;
    page?: number;
    page_size?: number;
    query_str?: string;
  };

  type fetchKeySentenceCurationParams = {
    page?: number;
    page_size?: number;
    query_str?: string;
  };

  type fetchNodesParams = {
    node_ids: string;
  };

  type fetchNotificationsParams = {
    page?: number;
    page_size?: number;
    query_str?: string;
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

  type fetchPublicationParams = {
    id: string;
  };

  type fetchPublicationsConsensusParams = {
    search_id: string;
  };

  type fetchPublicationsParams = {
    query_str: string;
    page?: number;
    page_size?: number;
  };

  type fetchPublicationsSummaryParams = {
    search_id: string;
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
    start_node_id?: string;
    topk?: number;
    nhops?: number;
    nums_shared_by?: number;
  };

  type fetchSubgraphsParams = {
    page?: number;
    page_size?: number;
    query_str?: string;
  };

  type fetchTaskByTaskIdParams = {
    task_id: string;
  };

  type fetchTasksParams = {
    page?: number;
    page_size?: number;
    query_str?: string;
  };

  type fetchWebpageMetadataParams = {
    page?: number;
    page_size?: number;
    query_str?: string;
  };

  type fetchWorkflowSchemaParams = {
    id: string;
  };

  type fetchWorkflowsParams = {
    page?: number;
    page_size?: number;
    query_str?: string;
  };

  type fetchWorkspacesParams = {
    page?: number;
    page_size?: number;
  };

  type GeneralReferences = {
    articles: Article[];
    links: Link[];
  };

  type GoClassifier = {
    category: string;
    description: string;
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

  type KeySentenceCuration = {
    id: number;
    fingerprint: string;
    curator: string;
    key_sentence: string;
    description: string;
    created_at: string;
    payload?: any;
    annotation?: any;
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
    fingerprint: string;
    payload?: any;
    annotation?: any;
  };

  type Label = {
    value: string;
    fill: string;
    fontSize: number;
    offset: number;
    position: string;
  };

  type Link = {
    ref_id: string;
    title: string;
    url: string;
  };

  type LlmResponse = {
    prompt: string;
    response: string;
    created_at: string;
  };

  type Manufacturer = {
    text: string;
    generic: string;
    url: string;
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

  type Notification = {
    id: string;
    title: string;
    description?: string;
    notification_type: string;
    created_time: string;
    status: string;
    owner: string;
  };

  type Organism = {
    text: string;
    ncbi_taxonomy_id: string;
  };

  type Packager = {
    name: string;
    url: string;
  };

  type Patent = {
    number: string;
    country: string;
    approved: string;
    expires: string;
    pediatric_extension: string;
  };

  type Pfam = {
    identifier: string;
    name: string;
  };

  type Polypeptide = {
    id: string;
    source: string;
    name: string;
    general_function: string;
    specific_function: string;
    gene_name: string;
    locus: string;
    cellular_location: string;
    transmembrane_regions: string;
    signal_regions: string;
    theoretical_pi: string;
    molecular_weight: string;
    chromosome_location: string;
    organism: Organism;
    external_identifiers: ExternalIdentifier[];
    synonyms: string[];
    amino_acid_sequence: Sequence;
    gene_sequence: Sequence;
    pfams: Pfam[];
    go_classifiers: GoClassifier[];
  };

  type postKeySentenceCurationImageParams = {
    id: number;
  };

  type Price = {
    description: string;
    cost: Cost;
    unit: string;
  };

  type Product = {
    name: string;
    labeller: string;
    ndc_id: string;
    ndc_product_code: string;
    dpd_id: string;
    ema_product_code: string;
    ema_ma_number: string;
    started_marketing_on: string;
    ended_marketing_on: string;
    dosage_form: string;
    strength: string;
    route: string;
    fda_application_number: string;
    generic: string;
    over_the_counter: string;
    approved: string;
    country: string;
    source: string;
  };

  type PromptList = {
    /** data */
    records: Record<string, any>[];
    /** total num */
    total: number;
    /** current page index */
    page: number;
    /** default 10 */
    page_size: number;
  };

  type Property = {
    kind: string;
    value: string;
    source: string;
  };

  type Publication = {
    authors: string[];
    citation_count?: number;
    summary: string;
    journal: string;
    title: string;
    year?: number;
    doc_id: string;
    article_abstract?: string;
    doi?: string;
    provider_url?: string;
  };

  type PublicationRecords = {
    records: Publication[];
    total: number;
    page: number;
    page_size: number;
    search_id?: string;
  };

  type PublicationsSummary = {
    summary: string;
    daily_limit_reached: boolean;
    is_disputed: boolean;
    is_incomplete: boolean;
    results_analyzed_count: number;
  };

  type putConfigurationParams = {
    id: number;
  };

  type putCuratedKnowledgeParams = {
    id: number;
  };

  type putEntityCurationParams = {
    id: number;
  };

  type putEntityMetadataCurationParams = {
    id: number;
  };

  type putKeySentenceCurationParams = {
    id: number;
  };

  type putSubgraphParams = {
    id: string;
  };

  type putWebpageMetadataParams = {
    id: number;
  };

  type RecordResponseConfiguration = {
    /** data */
    records: Configuration[];
    /** total num */
    total: number;
    /** current page index */
    page: number;
    /** default 10 */
    page_size: number;
  };

  type RecordResponseEmbedding = {
    /** data */
    records: Embedding[];
    /** total num */
    total: number;
    /** current page index */
    page: number;
    /** default 10 */
    page_size: number;
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

  type RecordResponseEntityCuration = {
    /** data */
    records: EntityCuration[];
    /** total num */
    total: number;
    /** current page index */
    page: number;
    /** default 10 */
    page_size: number;
  };

  type RecordResponseEntityMetadataCuration = {
    /** data */
    records: EntityMetadataCuration[];
    /** total num */
    total: number;
    /** current page index */
    page: number;
    /** default 10 */
    page_size: number;
  };

  type RecordResponseKeySentenceCuration = {
    /** data */
    records: KeySentenceCuration[];
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

  type RecordResponseNotification = {
    /** data */
    records: Notification[];
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

  type RecordResponseTask = {
    /** data */
    records: Task[];
    /** total num */
    total: number;
    /** current page index */
    page: number;
    /** default 10 */
    page_size: number;
  };

  type RecordResponseWebpageMetadata = {
    /** data */
    records: WebpageMetadata[];
    /** total num */
    total: number;
    /** current page index */
    page: number;
    /** default 10 */
    page_size: number;
  };

  type RecordResponseWorkflow = {
    /** data */
    records: Workflow[];
    /** total num */
    total: number;
    /** current page index */
    page: number;
    /** default 10 */
    page_size: number;
  };

  type RecordResponseWorkspace = {
    /** data */
    records: Workspace[];
    /** total num */
    total: number;
    /** current page index */
    page: number;
    /** default 10 */
    page_size: number;
  };

  type Relation = {
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
    prompt_template?: string;
  };

  type Sequence = {
    text: string;
    format: string;
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

  type SubgraphWithCtx = {
    context_str: string;
    subgraph: string;
  };

  type Target = {
    position: string;
    id: string;
    name: string;
    organism: string;
    actions: string[];
    references: GeneralReferences;
    known_action: string;
    polypeptide?: Polypeptide[];
  };

  type Task = {
    workspace_id: string;
    workflow_id: string;
    task_id: string;
    task_name: string;
    description?: string;
    submitted_time: string;
    started_time: string;
    finished_time: string;
    task_params: any;
    labels?: string[];
    status?: string;
    results?: any;
    log_message?: string;
    owner: string;
    groups?: string[];
  };

  type WebpageMetadata = {
    id: number;
    fingerprint: string;
    curator: string;
    note: string;
    metadata: any;
    created_at: string;
  };

  type Workflow = {
    id: string;
    name: string;
    version: string;
    description?: string;
    category: string;
    home: string;
    source: string;
    short_name: string;
    icons?: any;
    author: string;
    maintainers?: string[];
    tags?: string[];
    readme?: string;
  };

  type WorkflowSchema = {
    schema: any;
  };

  type Workspace = {
    workspace_name: string;
    description?: string;
    created_time: string;
    updated_time: string;
    archived_time?: string;
    payload?: any;
    owner: string;
    groups: string[];
  };
}
