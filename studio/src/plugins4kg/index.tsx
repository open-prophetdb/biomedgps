import ExpressionAtlas from "./ExpressionAtlas";
import GeneInfo from "./GeneInfo";
import GTexViewer from "./GTexViewer";
import MolStarViewer from "./MolStarViewer";
import MutationViewer from "./MutationViewer";
import SangerCosmic from "./SangerCosmic";
import SgrnaSelector from "./SgrnaSelector";

import type { GeneInfo as GeneInfoType } from "./utils";

export {
  ExpressionAtlas,
  GeneInfo,
  GTexViewer,
  MolStarViewer,
  MutationViewer,
  SangerCosmic,
  SgrnaSelector,
}

export const getItems4GenePanel = (geneInfo: GeneInfoType, hiddenItems: string[] = []) => {
  const ensemblId = geneInfo.ensembl?.gene;
  const geneSymbol = geneInfo.symbol;
  const entrezId = geneInfo.entrezgene;

  const items = [
    {
      label: "Summary",
      key: "summary",
      children: <GeneInfo geneSymbol={geneSymbol} />
    },
    {
      label: "Gene",
      key: "gene",
      children: <GTexViewer officialGeneSymbol={ensemblId} type="gene" />
    },
    {
      label: "Transcript",
      key: "transcript",
      children: <GTexViewer officialGeneSymbol={ensemblId} type="transcript" />
    },
    {
      label: "Expression Atlas",
      key: "expression-atlas",
      children: <ExpressionAtlas geneSymbol={geneSymbol} />
    },
    {
      label: "Mutation",
      key: "mutation",
      children: <SangerCosmic geneSymbol={geneSymbol} />
    },
    {
      label: "3D Structure",
      key: "structure",
      children: <MolStarViewer />
    },
    {
      label: "Preferred sgRNAs",
      key: "preferred-sgrnas",
      children: <SgrnaSelector geneId={entrezId} />
    }
  ]

  return items.filter(item => !hiddenItems.includes(item.key))
}