import { Empty, Tabs } from "antd";
import ExpressionAtlas from "./Components/ExpressionAtlas";
import GeneInfoPanel from "./GeneInfoPanel";
import CompoundInfoPanel from "./CompoundInfoPanel";
import MolStarViewer from "./Components/MolStarViewer";
import MutationViewer from "./Components/MutationViewer";
import SangerCosmic from "./Components/SangerCosmic";
import SgrnaSelector from "./Components/SgrnaSelector";
import type { GeneInfo } from "./index.t";
import { fetchEntityAttributes } from "@/services/swagger/KnowledgeGraph";
import { fetchMyGeneInfo } from "./ProteinInfoPanel/utils";
import React, { useEffect, useState } from "react";
import type { GraphNode } from "biominer-components/dist/typings";
import ProteinInfoPanel from "./ProteinInfoPanel";
import type { OptionType, Entity, ComposeQueryItem, QueryItem } from 'biominer-components/dist/typings';

import "./index.less";

export {
  CompoundInfoPanel,
  ExpressionAtlas,
  GeneInfoPanel,
  // GTexViewer,
  MolStarViewer,
  MutationViewer,
  SangerCosmic,
  SgrnaSelector,
}

const makeQueryEntityStr = (compoundId: string) => {
  let query_item = {} as QueryItem;
  if (compoundId) {
    query_item = {
      operator: '=',
      field: 'drugbank_id',
      value: compoundId,
    };
  }

  return JSON.stringify(query_item);
}

const NodeInfoPanel: React.FC<{ node?: GraphNode, hiddenItems?: string[] }> = ({ node, hiddenItems }) => {
  const [geneInfo, setGeneInfo] = useState<GeneInfo | null>(null);
  // How to define the type of compoundInfo?
  const [compoundInfo, setCompoundInfo] = useState<any | null>(null);
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

    if (node.data.label == "Gene") {
      const entrezId = node.data.id.replace("ENTREZ:", "");
      fetchMyGeneInfo(entrezId).then(setGeneInfo);
    } else if (node.data.label == "Compound") {
      fetchEntityAttributes({
        entity_type: "Compound",
        query_str: makeQueryEntityStr(node.data.id)
      }).then((res) => {
        const compoundInfo = res.compounds;
        if (compoundInfo && compoundInfo.records && compoundInfo.records.length > 0) {
          setCompoundInfo(compoundInfo.records[0]);
        } else {
          setCompoundInfo(null);
        }
      }).catch((err) => {
        console.error(err);
        setCompoundInfo(null);
      });
      setCompoundInfo(node.data);
    } else {
      setGeneInfo(null);
      setCompoundInfo(null);
    }
  }, []);

  useEffect(() => {
    if (!geneInfo) {
      return;
    }

    const geneSymbol = geneInfo.symbol;
    const entrezId = geneInfo.entrezgene;
    const geneItems = [
      {
        label: "Summary",
        key: "summary",
        children: <ProteinInfoPanel geneInfo={geneInfo} />
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
      setItems(geneItems.filter(item => !hiddenItems.includes(item.key)));
    } else {
      setItems(geneItems);
    };
  }, [geneInfo]);

  useEffect(() => {
    if (!compoundInfo) {
      return;
    }

    const compoundItems = [
      {
        label: "Summary",
        key: "summary",
        children: <CompoundInfoPanel compoundInfo={compoundInfo} />
      },
    ]

    if (hiddenItems) {
      setItems(compoundItems.filter(item => !hiddenItems.includes(item.key)));
    } else {
      setItems(compoundItems);
    };
  }, [compoundInfo]);

  return <Tabs
    className="plugins4kg tabs-nav-right"
    items={items && items.length > 0 ? items : defaultItems}
  />
}

export default NodeInfoPanel;
