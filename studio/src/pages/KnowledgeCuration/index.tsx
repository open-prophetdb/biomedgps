import React, { useRef, useState } from 'react';
import { Tabs, Table, Empty, Button, message } from 'antd';
import KnowledgeTable from './KnowledgeTable';
import KeySentenceTable from './KeySentenceTable';
import EntityTable from './EntityTable';

import './index.less';

const KnowledgeCuration: React.FC = () => {
    const keySentenceTableRef = useRef(null);
    const entityTableRef = useRef(null);
    const knowledgeTableRef = useRef(null);
    const entityMetadataTableRef = useRef(null);
    const [currentTab, setCurrentTab] = useState('1');

    return (
        <Tabs defaultActiveKey="1" className='knowledge-curation-tabs'
            onChange={(key) => {
                setCurrentTab(key);
            }}
            tabBarExtraContent={
                <Button type="primary" onClick={() => {
                    if (currentTab === '1') {
                        // @ts-ignore
                        keySentenceTableRef.current?.downloadTable();
                    } else if (currentTab === '2') {
                        // @ts-ignore
                        entityTableRef.current?.downloadTable();
                    } else if (currentTab === '3') {
                        // @ts-ignore
                        knowledgeTableRef.current?.downloadTable();
                    } else if (currentTab === '4') {
                        message.warning("No data to download.")
                    }
                }}>Download Table</Button>
            }
        >
            <Tabs.TabPane tab="Key Sentences" key="1">
                {/* @ts-ignore */}
                <KeySentenceTable ref={keySentenceTableRef} />
            </Tabs.TabPane>
            <Tabs.TabPane tab="Entities" key="2">
                {/* @ts-ignore */}
                <EntityTable ref={entityTableRef} />
            </Tabs.TabPane>
            <Tabs.TabPane tab="Knowledges" key="3">
                {/* @ts-ignore */}
                <KnowledgeTable ref={knowledgeTableRef} />
            </Tabs.TabPane>
            <Tabs.TabPane tab="Entity Metadata" key="4">
                <Empty description="No data" />
            </Tabs.TabPane>
        </Tabs>
    );
};

export default KnowledgeCuration;
