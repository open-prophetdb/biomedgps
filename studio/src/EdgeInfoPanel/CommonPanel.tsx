import React, { useEffect } from 'react';
import { Empty, Tabs } from 'antd';
import type { EdgeInfo } from './index.t';

import './index.less';

type CommonPanelProps = {
    edgeInfo?: EdgeInfo;
    children?: React.ReactNode;
};

const CommonPanel: React.FC<CommonPanelProps> = (props) => {
    const { edge, startNode, endNode } = props.edgeInfo || {
        edge: undefined,
        startNode: undefined,
        endNode: undefined,
    };

    useEffect(() => { }, [edge, startNode, endNode]);

    return (
        <Tabs className="common-info-panel">
            {
                props.children ? (
                    <Tabs.TabPane tab={'Summary'} key={'summary'}>
                        {props.children}
                    </Tabs.TabPane>
                ) : <Empty />
            }
        </Tabs>
    );
};

export default CommonPanel;
