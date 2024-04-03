type Article = {
    ref_id: string;
    pubmed_id: string;
    citation: string;
};

export type AtcCode = {
    code: string;
    level: AtcCodeLevel[];
};

export type AtcCodeLevel = {
    code: string;
    text: string;
};

export type Category = {
    category: string;
    mesh_id: string;
};

export type Classification = {
    description: string;
    direct_parent: string;
    kingdom: string;
    superclass: string;
    class: string;
    subclass: string;
};

export type Cost = {
    currency: string;
    text: string;
};

export type Dosage = {
    form: string;
    route: string;
    strength: string;
};

export type ExperimentalProperty = {
    property: Property[];
};

export type ExternalIdentifier = {
    resource: string;
    identifier: string;
};

export type ExternalLink = {
    resource: string;
    url: string;
};

export type GeneralReferences = {
    articles: Article[];
    links: Link[];
};

export type GoClassifier = {
    category: string;
    description: string;
};

export type Link = {
    ref_id: string;
    title: string;
    url: string;
};

export type Manufacturer = {
    text: string;
    generic: string;
    url: string;
};

export type Organism = {
    text: string;
    ncbi_taxonomy_id: string;
};

export type Packager = {
    name: string;
    url: string;
};

export type Patent = {
    number: string;
    country: string;
    approved: string;
    expires: string;
    pediatric_extension: string;
};

export type Pfam = {
    identifier: string;
    name: string;
};

export type Polypeptide = {
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

export type Price = {
    description: string;
    cost: Cost;
    unit: string;
};

export type Product = {
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

export type Property = {
    kind: string;
    value: string;
    source: string;
};

export type Sequence = {
    text: string;
    format: string;
};

export type Target = {
    position: string;
    id: string;
    name: string;
    organism: string;
    actions: string[];
    references: GeneralReferences;
    known_action: string;
    polypeptide?: Polypeptide[];
};

export type CompoundInfo = {
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
