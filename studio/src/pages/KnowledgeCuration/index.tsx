import React, { useRef } from 'react';
import { Tabs, Table, Empty, Button } from 'antd';
import KnowledgeTable from './KnowledgeTable';
import KeySentenceTable from './KeySentenceTable';
import EntityTable from './EntityTable';

import './index.less';

const KnowledgeCuration: React.FC = () => {
    const keySentenceTableRef = useRef(null);
    const entityTableRef = useRef(null);
    const knowledgeTableRef = useRef(null);

    return (
        <Tabs defaultActiveKey="1" className='knowledge-curation-tabs' tabBarExtraContent={
            <Button type="primary" onClick={() => {
                // @ts-ignore
                keySentenceTableRef.current?.downloadTable();
            }}>Download Table</Button>
        }>
            <Tabs.TabPane tab="Key Sentences" key="1">
                {/* @ts-ignore */}
                <KeySentenceTable ref={keySentenceTableRef} />
            </Tabs.TabPane>
            <Tabs.TabPane tab="Entities" key="2">
                <EntityTable />
            </Tabs.TabPane>
            <Tabs.TabPane tab="Knowledges" key="3">
                <KnowledgeTable />
            </Tabs.TabPane>
            <Tabs.TabPane tab="Entity Metadata" key="4">
                <Empty description="No data" />
            </Tabs.TabPane>
        </Tabs>
    );
};

export default KnowledgeCuration;
