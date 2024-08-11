import React, { useEffect } from 'react';
import { Empty, Tabs } from 'antd';
import type { EdgeInfo } from './index.t';

import './index.less';

type CommonPanelProps = {
    relationType: string;  // DrugDisease, DrugGene, GeneDisease, see index.tsx for more details
    edgeInfo?: EdgeInfo;
    children?: React.ReactNode;
};

const CommonPanel: React.FC<CommonPanelProps> = (props) => {
    const { relationType } = props;
    const { edge, startNode, endNode } = props.edgeInfo || {
        edge: undefined,
        startNode: undefined,
        endNode: undefined,
    };

    const whichPanel = () => {
        switch (relationType) {
            case 'DrugDisease':
                return <>
                    <Tabs.TabPane tab={'Clinical Trials'} key={'drug-disease-info'}>
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
                </>;
            case 'DrugGene':
                return <>
                    <Tabs.TabPane tab={'DrugGene Info'} key={'drug-gene-info'}>
                        We can show the drug-gene association information here. Maybe it's summarized information
                        from publications.
                    </Tabs.TabPane>
                    <Tabs.TabPane tab={'Drug Targets'} key={'clinical-trails'}>
                        Comming soon...
                    </Tabs.TabPane>
                </>;
            case 'GeneDisease':
                return <>
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
                </>;
            default:
                return null;
        }
    }

    useEffect(() => { }, [edge, startNode, endNode]);

    return (
        <Tabs className="common-info-panel">
            {/* <Tabs.TabPane tab={'Summary'} key={'summary'}>
                Comming soon...
            </Tabs.TabPane> */}
            {
                props.children ? (
                    <Tabs.TabPane tab={'Publication'} key={'publication'}>
                        {props.children}
                    </Tabs.TabPane>
                ) : <Empty />
            }
            {whichPanel()}
        </Tabs>
    );
};

export default CommonPanel;
