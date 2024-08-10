import React, { useEffect, useState } from 'react';
import { Row } from 'antd';
import DrugGene from './DrugGenePanel';
import DrugDisease from './DrugDiseasePanel';
import GeneDisease from './DiseaseGenePanel';
import PublicationPanel from './PublicationPanel';
import CommonPanel from './CommonPanel';
import type { EdgeInfoPanelProps } from './index.t';

import './index.less';

const EdgeInfoPanel: React.FC<EdgeInfoPanelProps> = (props) => {
  const { edge, startNode, endNode } = props.edgeInfo || {
    edge: undefined,
    startNode: undefined,
    endNode: undefined,
  };
  const [relationType, setRelationType] = useState<string>('Unknown');

  const format_prompt = (prompt_template: string, source_type: string,
    source_name: string, target_type: string, target_name: string) => {
    if (source_type == target_type) {
      return prompt_template.replace(`#${source_type}1#`, source_name).replace(`#${target_type}2#`, target_name)
    } else {
      return prompt_template.replace(`#${source_type}#`, source_name).replace(`#${target_type}#`, target_name)
    }
  }

  const whichPanel = () => {
    if (relationType !== 'Unknown') {
      console.log('whichPanel: ', relationType, edge, startNode, endNode);
      let queryStr = '';
      if (startNode && endNode) {
        queryStr = edge.prompt_template ?
          format_prompt(edge.prompt_template, startNode.data.label,
            startNode.data.name, endNode.data.label, endNode.data.name) :
          `${edge.description}; ${edge.data.source_type}: ${startNode.data.name}, ${edge.data.target_type}: ${endNode.data.name}`
      }

      if (queryStr) {
        switch (relationType) {
          case 'DrugDisease':
            return <DrugDisease edgeInfo={props.edgeInfo}>
              <PublicationPanel queryStr={queryStr} />
            </DrugDisease>;
          case 'DrugGene':
            return <DrugGene edgeInfo={props.edgeInfo}>
              <PublicationPanel queryStr={queryStr} />
            </DrugGene>;
          case 'GeneDisease':
            return <GeneDisease edgeInfo={props.edgeInfo}>
              <PublicationPanel queryStr={queryStr} />
            </GeneDisease>;
          default:
            return <CommonPanel edgeInfo={props.edgeInfo}>
              <PublicationPanel queryStr={queryStr} />
            </CommonPanel>;
        }
      }
    }
  };

  useEffect(() => {
    console.log('EdgeInfoPanel: ', edge, startNode, endNode);
    if (edge && startNode && endNode) {
      const startNodeType = startNode.data.label;
      const endNodeType = endNode.data.label;
      const relationTypes = [startNodeType, endNodeType].sort().join('');

      console.log('relationTypes: ', relationTypes, relationType);

      setRelationType('');

      if (['CompoundDisease', 'ChemicalDisease', 'DiseaseDrug'].indexOf(relationTypes) >= 0) {
        setRelationType('DrugDisease');
      }

      if (['DiseaseGene', 'GeneDisease'].indexOf(relationTypes) >= 0) {
        setRelationType('GeneDisease');
      }

      if (['CompoundGene', 'ChemicalGene', 'DrugGene'].indexOf(relationTypes) >= 0) {
        setRelationType('DrugGene');
      }
    }
  }, [edge, startNode, endNode]);

  return <Row className="edge-info-panel">{whichPanel()}</Row>;
};

export default EdgeInfoPanel;
