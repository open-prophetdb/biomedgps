import React, { useEffect } from 'react';
import { Tabs } from 'antd';
import type { EdgeInfo } from './index.t';

import './index.less';

type DrugDiseasePanel = {
  edgeInfo?: EdgeInfo;
  children?: React.ReactNode;
};

const DrugDiseasePanel: React.FC<DrugDiseasePanel> = (props) => {
  const { edge, startNode, endNode } = props.edgeInfo || {
    edge: undefined,
    startNode: undefined,
    endNode: undefined,
  };

  useEffect(() => { }, [edge, startNode, endNode]);

  return (
    <Tabs className="drug-disease-info-panel tabs-nav-right">
      {props.children ? (
        <Tabs.TabPane tab={'Summary'} key={'summary'}>
          {props.children}
        </Tabs.TabPane>
      ) : null}
      <Tabs.TabPane tab={'DrugDisease Info'} key={'drug-disease-info'}>
        We can show the drug-disease association information here. Maybe it's summarized information
        from clinical trials, or publications.
      </Tabs.TabPane>
      <Tabs.TabPane tab={'Patents'} key={'drug-patent-info'}>
        We can show the patents information here. Maybe it's summarized information from patents
        database.
      </Tabs.TabPane>
      <Tabs.TabPane tab={'Products'} key={'drug-product-info'}>
        We can show the production information here. Maybe it's summarized information from drug
        production database.
      </Tabs.TabPane>
    </Tabs>
  );
};

export default DrugDiseasePanel;
