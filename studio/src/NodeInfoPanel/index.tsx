import { Empty, Tabs } from "antd";
import GeneInfoPanel from "./GeneInfoPanel";
import CompoundInfoPanel from "./CompoundInfoPanel";
import GTexViewer from "./Components/GTexViewer";
import MolStarViewer from "./Components/MolStarViewer";
import MutationViewer from "./Components/MutationViewer";
import SangerCosmic from "./Components/SangerCosmic";
import ProteinAtlas from "./Components/ProteinAtlas";
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
    // TODO: We might get several ensembl ids, how to handle this?
    const ensemblId = geneInfo?.ensembl?.gene;
    const geneItems = [
      {
        label: "Summary",
        key: "summary",
        children: <ProteinInfoPanel geneInfo={geneInfo} />
      },
      {
        label: "Gene Expression",
        key: "gene",
        children: <GTexViewer ensemblId={ensemblId} type="gene" description={
          <span>
            <span>
              <b>Data Source:</b> GTEx Analysis Release V8 (dbGaP Accession phs000424.v8.p2), Data processing and normalization. <a href={`https://gtexportal.org/home/gene/${ensemblId}`} target="_blank">More information</a>
            </span>
            <br />
            <span>
              <b>Method:</b> Expression values are shown in TPM (Transcripts Per Million), calculated from a gene model with isoforms collapsed to a single gene. No other normalization steps have been applied. Box plots are shown as median and 25th and 75th percentiles; points are displayed as outliers if they are above or below 1.5 times the interquartile range.
            </span>
          </span>
        } />
        // children: <Empty description="Comming soon..." />
      },
      {
        label: "Transcript Expression",
        key: "transcript",
        children: <GTexViewer ensemblId={ensemblId} type="transcript" description={
          <span>
            Data Source: GTEx Analysis Release V8 (dbGaP Accession phs000424.v8.p2). <a href={`https://gtexportal.org/home/gene/${ensemblId}`} target="_blank">More information</a>
          </span>
        } />
        // children: <Empty description="Comming soon..." />
      },
      // {
      //   label: "Expression Atlas",
      //   key: "expression-atlas",
      //   children: <ExpressionAtlas geneSymbol={geneSymbol} />
      // },
      {
        label: "Mutation - SangerCosmic",
        key: "mutation",
        children: <SangerCosmic geneSymbol={geneSymbol} />
        // children: <Empty description="Comming soon..." />
      },
      {
        label: "Protein Atlas",
        key: "protein-atlas",
        children: <ProteinAtlas geneSymbol={geneSymbol} ensemblId={ensemblId} />,
        // children: <Empty description="Comming soon..." />
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
    className="plugins4kg tabs-nav-right" destroyInactiveTabPane
    items={items && items.length > 0 ? items : defaultItems}
  />
}

export default NodeInfoPanel;
