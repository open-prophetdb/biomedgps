import { Empty, Tabs } from "antd";
import ExpressionAtlas from "./ExpressionAtlas";
import GeneInfoPanel from "./GeneInfoPanel";
import MolStarViewer from "./MolStarViewer";
import MutationViewer from "./MutationViewer";
import SangerCosmic from "./SangerCosmic";
import SgrnaSelector from "./SgrnaSelector";
import type { GeneInfo } from "./index.t";
import { fetchMyGeneInfo } from "./ProteinInfoPanel/utils";
import React, { useEffect, useState } from "react";
import type { GraphNode } from "biominer-components/dist/typings";
import ComposedProteinPanel from "./ComposedProteinPanel";

import "./index.less";

export {
  ExpressionAtlas,
  GeneInfoPanel,
  // GTexViewer,
  MolStarViewer,
  MutationViewer,
  SangerCosmic,
  SgrnaSelector,
}

const NodeInfoPanel: React.FC<{ node?: GraphNode, hiddenItems?: string[] }> = ({ node, hiddenItems }) => {
  const [geneInfo, setGeneInfo] = useState<GeneInfo | null>(null);
  const [items, setItems] = useState<any[]>([]);

  const defaultItems = [
    {
      label: "Summary",
      key: "summary",
      children: <Empty description="Comming soon..." />
    }
  ]

  useEffect(() => {
    if (!node) {
      return;
    }

    if (node.data.label !== "Gene") {
      return;
    } else {
      const entrezId = node.data.id.replace("ENTREZ:", "");
      fetchMyGeneInfo(entrezId).then(setGeneInfo);
    }
  }, []);

  useEffect(() => {
    if (!geneInfo) {
      return;
    }

    const geneSymbol = geneInfo.symbol;
    const entrezId = geneInfo.entrezgene;
    const defaultItems = [
      {
        label: "Summary",
        key: "summary",
        children: <ComposedProteinPanel geneInfo={geneInfo} />
      },
      {
        label: "Gene Expression",
        key: "gene",
        // children: <GTexViewer officialGeneSymbol={ensemblId} type="gene" />
        children: <Empty description="Comming soon..." />
      },
      {
        label: "Transcript Expression",
        key: "transcript",
        // children: <GTexViewer officialGeneSymbol={ensemblId} type="transcript" />
        children: <Empty description="Comming soon..." />
      },
      {
        label: "Expression Atlas",
        key: "expression-atlas",
        children: <ExpressionAtlas geneSymbol={geneSymbol} />
      },
      {
        label: "Mutation",
        key: "mutation",
        // children: <SangerCosmic geneSymbol={geneSymbol} />
        children: <Empty description="Comming soon..." />
      },
      {
        label: "Preferred sgRNAs",
        key: "preferred-sgrnas",
        children: <SgrnaSelector geneId={entrezId} />
      }
    ]

    if (hiddenItems) {
      setItems(defaultItems.filter(item => !hiddenItems.includes(item.key)));
    } else {
      setItems(defaultItems);
    };
  }, [geneInfo]);

  return <Tabs
    className="plugins4kg tabs-nav-right"
    items={items && items.length > 0 ? items : defaultItems}
  />
}

export default NodeInfoPanel;
