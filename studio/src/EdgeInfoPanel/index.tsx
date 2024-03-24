import React, { useEffect, useState } from 'react';
import { Row } from 'antd';
import type { EdgeInfoPanelProps } from './index.t';
import DrugGene from './DrugGenePanel';
import DrugDisease from './DrugDiseasePanel';
import GeneDisease from './DiseaseGenePanel';
import PublicationPanel from './PublicationPanel';
import { SEPARATOR } from './PublicationDesc';
import CommonPanel from './CommonPanel';

import './index.less';

const EdgeInfoPanel: React.FC<EdgeInfoPanelProps> = (props) => {
  const { edge, startNode, endNode } = props.edgeInfo || {
    edge: undefined,
    startNode: undefined,
    endNode: undefined,
  };
  const [relationType, setRelationType] = useState<string>('Unknown');

  const whichPanel = (relationType: string) => {
    console.log('whichPanel: ', relationType);
    let queryStr = '';
    if (startNode && endNode) {
      queryStr = startNode.data.name + SEPARATOR + endNode.data.name;
    }

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

  return <Row className="edge-info-panel">{whichPanel(relationType)}</Row>;
};

export default EdgeInfoPanel;
