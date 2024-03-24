import { GeneInfo, UniProtEntry } from '../index.t';

export const fetchMyGeneInfo = async (entrezId: string): Promise<GeneInfo> => {
    const response = await fetch(`https://mygene.info/v3/gene/${entrezId}?fields=all&dotfield=false&size=10`);
    if (!response.ok) {
        throw new Error("Failed to fetch gene information");
    }

    const data = await response.json();

    if (Object.keys(data).length === 0) {
        throw new Error("No gene found");
    } else {
        // Perform a type assertion to GeneInfo. This assumes the shape of hits[0] matches GeneInfo.
        // For more robust type safety, consider validating the shape of hits[0] before casting.
        return data as GeneInfo;
    }
};

export const isProteinCoding = (geneInfo: GeneInfo): boolean => {
    return geneInfo.type_of_gene === 'protein-coding';
}

export const fetchProteinInfo = async (uniprotId: string): Promise<UniProtEntry> => {
    const response = await fetch(`https://rest.uniprot.org/uniprotkb/${uniprotId}`);
    if (!response.ok) {
        throw new Error("Failed to fetch protein information");
    }

    const data: UniProtEntry = await response.json();

    if (Object.keys(data).length === 0) {
        throw new Error("No protein found");
    } else {
        return data;
    }
}
