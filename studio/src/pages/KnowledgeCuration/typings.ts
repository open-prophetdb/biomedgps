export type GraphEdge = {
    source_name: string;
    source_id: string;
    source_type: string;
    target_name: string;
    target_id: string;
    target_type: string;
    key_sentence: string;
    relation_type: string;
    id?: number;
    curator?: string;
    created_at?: string;
    fingerprint: string;
    annotation?: any;
};

export type GraphTableData = {
    data: GraphEdge[];
    total: number;
    page: number;
    pageSize: number;
};

export type KeySentence = {
    id: number;
    key_sentence: string;
    description: string;
    curator: string;
    created_at: string;
    fingerprint: string;
    annotation?: any;
};

export type KeySentenceTableData = {
    data: KeySentence[];
    total: number;
    page: number;
    pageSize: number;
};

export type EntityCuration = {
    id: number;
    entity_id: string;
    entity_name: string;
    entity_type: string;
    curator: string;
    created_at: string;
    fingerprint: string;
    annotation?: any;
};

export type EntityTableData = {
    data: EntityCuration[];
    total: number;
    page: number;
    pageSize: number;
};
