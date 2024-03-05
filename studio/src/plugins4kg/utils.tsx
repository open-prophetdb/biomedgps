import axios from 'axios';

export type GeneInfo = {
  _id: string;
  _version: number;
  entrezgene: number;
  hgnc: number;
  name: string;
  symbol: string;
  taxid: number;
  summary: string;
  type_of_gene: string;
  ensembl: {
    gene: string;
    transcript: string[];
    protein: string[];
    translation: string[];
  };
  genomic_pos: {
    chr: string;
    start: number;
    end: number;
    strand: number;
  };
}

// geneId: e.g. 7157. It's a entrez gene id
export const getGeneInfo = async (geneId: string) => {
  const { data } = await axios.get(`https://mygene.info/v3/gene/${geneId}`)

  const formatedData: GeneInfo = {
    _id: data._id,
    _version: data._version,
    entrezgene: data.entrezgene,
    hgnc: data['HGNC'],
    name: data.name,
    symbol: data.symbol,
    taxid: data.taxid,
    type_of_gene: data.type_of_gene,
    summary: data.summary,
    // TODO: handle the case when ensembl is undefined
    ensembl: {
      gene: data.ensembl.gene,
      transcript: data.ensembl.transcript,
      protein: data.ensembl.protein,
      translation: data.ensembl.translation,
    },
    genomic_pos: {
      chr: data.genomic_pos.chr,
      start: data.genomic_pos.start,
      end: data.genomic_pos.end,
      strand: data.genomic_pos.strand,
    }
  }

  return formatedData;
}