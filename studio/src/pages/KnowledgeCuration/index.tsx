import React, { useRef, useState } from 'react';
import { Tabs, Table, Empty, Button, message, Row, Modal } from 'antd';
import KnowledgeTable from './KnowledgeTable';
import KeySentenceTable from './KeySentenceTable';
import EntityTable from './EntityTable';
import Statistics from './Statistics';
import KnowledgeGraphEditorWrapper from './KnowledgeGraphEditor';

import './index.less';

const KnowledgeCuration: React.FC = () => {
    const keySentenceTableRef = useRef(null);
    const entityTableRef = useRef(null);
    const knowledgeTableRef = useRef(null);
    const entityMetadataTableRef = useRef(null);
    const [currentTab, setCurrentTab] = useState('1');
    const [graphVisible, setGraphVisible] = useState(false);

    return (
        <Row>
            <Tabs defaultActiveKey="1" className='knowledge-curation-tabs'
                onChange={(key) => {
                    setCurrentTab(key);
                }}
                tabBarExtraContent={
                    <Row>
                        <Button type="primary" onClick={() => {
                            setGraphVisible(true);
                        }} style={{ marginRight: '10px' }}>Show Graph</Button>
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
                            } else if (currentTab === '5') {
                                message.warning("Statistics data cannot be downloaded.")
                            }
                        }}>Download Table</Button>
                    </Row>
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
                <Tabs.TabPane tab="Statistics" key="5">
                    <Statistics />
                </Tabs.TabPane>
            </Tabs>
            <Modal open={graphVisible} onCancel={() => setGraphVisible(false)} className='knowledge-graph-editor-modal' footer={null} width={'100%'}>
                <KnowledgeGraphEditorWrapper />
            </Modal>
        </Row>
    );
};

export default KnowledgeCuration;
