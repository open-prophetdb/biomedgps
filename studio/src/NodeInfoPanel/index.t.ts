// You can follow this link https://mygene.info/v3/query?q=IL6&fields=all&size=10&from=0&fetch_all=false&facet_size=10&entrezonly=false&ensemblonly=false&dotfield=false to know the structure of the response
export interface GeneInfo {
    AllianceGenome?: string;
    HGNC: string;
    _id: string;
    _score: number;
    accession?: Accession;
    ensembl: Ensembl;
    entrezgene: string;
    exons?: Exon[];
    exons_hg19?: Exon[];
    generif?: Generif[];
    genomic_pos: GenomicPos;
    map_location?: string;
    name: string;
    refseq?: RefSeq;
    reporter?: Reporter;
    summary?: string;
    symbol: string;
    taxid: number;
    homologene?: {
        genes: number[][], // Such as "genes": [[3702, 824036], [9606, 1017]], the first number is the taxid and the second number is the entrezid.
        id: number;
    };
    type_of_gene: string;
    unigene?: string[];
    uniprot?: {
        'Swiss-Prot': string;
        TrEMBL?: string[];
    },
    alias?: string[];
}

export interface Accession {
    genomic?: string[];
    protein?: string;
    rna?: string[];
    translation?: Translation;
}

export interface Translation {
    protein: string;
    rna: string;
}

export interface Ensembl {
    gene: string;
    transcript: string;
    translation: any[]; // Consider defining this more accurately if possible.
    type_of_gene: string;
}

export interface Exon {
    cdsend?: number;
    cdsstart?: number;
    chr: string;
    position: number[][];
    strand: number;
    transcript: string;
    txend: number;
    txstart: number;
}

export interface Generif {
    pubmed: number;
    text: string;
}

export interface GenomicPos {
    chr: string;
    end: number;
    ensemblgene: string;
    start: number;
    strand: number;
}

export interface RefSeq {
    genomic?: string[];
    rna?: string;
}

export interface Reporter {
    [key: string]: string;
}

// Please follow this link https://rest.uniprot.org/uniprotkb/P24941 for knowing the structure of the response
export interface UniProtEntry {
    entryType: string;
    primaryAccession: string;
    secondaryAccessions: string[];
    uniProtkbId: string;
    entryAudit: EntryAudit;
    annotationScore: number;
    organism: Organism;
    proteinExistence: string;
    proteinDescription: ProteinDescription;
    genes: Gene[];
    comments: Comment[];
    features: Feature[];
    keywords: Keyword[];
    references: Reference[];
    uniProtKBCrossReferences: UniProtKBCrossReference[];
    sequence: Sequence;
}

export interface Sequence {
    value: string;
    length: number;
    molWeight: number;
    crc64: string;
    md5: string;
}

export interface EntryAudit {
    firstPublicDate: string;
    lastAnnotationUpdateDate: string;
    lastSequenceUpdateDate: string;
    entryVersion: number;
    sequenceVersion: number;
}

export interface Organism {
    scientificName: string;
    commonName: string;
    taxonId: number;
    lineage: string[];
}

export interface ProteinDescription {
    recommendedName: RecommendedName;
    alternativeNames: AlternativeName[];
    flag: string;
}

export interface RecommendedName {
    fullName: ValueWithEvidences;
    shortNames: Value[];
}

export interface AlternativeName {
    fullName: Value;
    shortNames?: Value[];
}

export interface Value {
    value: string;
}

export interface ValueWithEvidences extends Value {
    evidences?: Evidence[];
}

export interface Evidence {
    evidenceCode: string;
    source: string;
    id?: string;
}

export interface Gene {
    geneName: ValueWithEvidences;
    synonyms?: Value[];
}

export interface Comment {
    texts: Text[];
    commentType: string;
}

export interface Text {
    evidences?: Evidence[];
    value: string;
}

export interface Feature {
    type: string;
    location: Location;
    description?: string;
    evidences?: Evidence[];
    featureId?: string;
}

export interface Location {
    start: LocationDetail;
    end: LocationDetail;
}

export interface LocationDetail {
    value: number;
    modifier: string;
}

export interface Keyword {
    id: string;
    category: string;
    name: string;
}

export interface Reference {
    citation: Citation;
    referencePositions: string[];
}

export interface Citation {
    id: string;
    citationType: string;
    authors?: string[];
    citationCrossReferences?: CitationCrossReference[];
    title: string;
    publicationDate: string;
    journal: string;
    firstPage?: string;
    lastPage?: string;
    volume?: string;
}

export interface CitationCrossReference {
    database: string;
    id: string;
}

export interface UniProtKBCrossReference {
    database: string;
    id: string;
    properties: CrossReferenceProperty[];
}

export interface CrossReferenceProperty {
    key: string;
    value: string;
}

export interface AlignmentData {
    proteinDescription: string;
    proteinName: string;
    sequenceVersion: number;
    score: number;
    uniProtId: string;
    sequence: string;
    species: string;
    geneSymbol: string;
    entrezgene: string;
};
