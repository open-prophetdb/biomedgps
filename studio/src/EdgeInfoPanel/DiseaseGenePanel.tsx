import React, { useEffect } from 'react';
import { Tabs } from 'antd';
import type { EdgeInfo } from './index.t';

import './index.less';

type DiseaseGenePanelProps = {
  edgeInfo?: EdgeInfo;
  children?: React.ReactNode;
};

const DiseaseGenePanel: React.FC<DiseaseGenePanelProps> = (props) => {
  const { edge, startNode, endNode } = props.edgeInfo || {
    edge: undefined,
    startNode: undefined,
    endNode: undefined,
  };

  useEffect(() => {
    console.log('GeneDiseasePanel: ', edge, startNode, endNode);
  }, [edge, startNode, endNode]);

  return (
    <Tabs className="gene-disease-info-panel tabs-nav-right">
      {props.children ? (
        <Tabs.TabPane tab={'Summary'} key={'summary'}>
          {props.children}
        </Tabs.TabPane>
      ) : null}
      <Tabs.TabPane tab={'GeneDiease Info'} key={'gene-disease-info'}>
        We can show the gene-disease association information here. Maybe it's summarized information
        from publications.
      </Tabs.TabPane>
      <Tabs.TabPane tab={'Diff Expression'} key={'diff-expr'}>
        we can show the diff expression here. It can tell us whether the gene is up-regulated or
        down-regulated in the disease.
      </Tabs.TabPane>
      <Tabs.TabPane tab={'Biomarkers'} key={'biomarker'}>
        we can show the related biomarkers. It can tell us which genes are the biomarkers of the
        disease.
      </Tabs.TabPane>
    </Tabs>
  );
};

export default DiseaseGenePanel;
