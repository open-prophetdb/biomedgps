import React, { useEffect } from 'react';
import { Tabs } from 'antd';
import type { EdgeInfo } from './index.t';

import './index.less';

type DrugGenePanelProps = {
  edgeInfo?: EdgeInfo;
  children?: React.ReactNode;
};

const DrugGenePanel: React.FC<DrugGenePanelProps> = (props) => {
  const { edge, startNode, endNode } = props.edgeInfo || {
    edge: undefined,
    startNode: undefined,
    endNode: undefined,
  };

  useEffect(() => { }, [edge, startNode, endNode]);

  return (
    <Tabs className="drug-gene-info-panel">
      {props.children ? (
        <Tabs.TabPane tab={'Summary'} key={'summary'}>
          {props.children}
        </Tabs.TabPane>
      ) : null}
      <Tabs.TabPane tab={'DrugGene Info'} key={'drug-gene-info'}>
        We can show the drug-gene association information here. Maybe it's summarized information
        from publications.
      </Tabs.TabPane>
      <Tabs.TabPane tab={'Drug Targets'} key={'clinical-trails'}>
        Comming soon...
      </Tabs.TabPane>
    </Tabs>
  );
};

export default DrugGenePanel;
