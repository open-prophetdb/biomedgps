import React, { useEffect, useState } from 'react';
import { history } from 'umi';
import { Table, Row, Tag, Space, message, Popover, Button, Empty, Tooltip, Drawer, Spin } from 'antd';
import { QuestionCircleOutlined } from '@ant-design/icons';
import type { ColumnsType } from 'antd/es/table';
import { useAuth0 } from "@auth0/auth0-react";
import { useLocation } from "react-router-dom";
import { fetchOneStepLinkedNodes, fetchRelationCounts } from '@/services/swagger/KnowledgeGraph';
import type { ComposeQueryItem, Entity, GraphData, GraphEdge, GraphNode } from 'biominer-components/dist/typings';
import { guessLink } from 'biominer-components/dist/utils';
import { pushGraphDataToLocalStorage } from 'biominer-components/dist/KnowledgeGraph/utils';
import type { EdgeInfo } from '@/EdgeInfoPanel/index.t';
import NodeInfoPanel from '@/NodeInfoPanel';
import EdgeInfoPanel from '@/EdgeInfoPanel';
import { sortBy, filter, uniqBy } from 'lodash';
import { guessColor } from '@/components/util';
import EntityCard from '@/components/EntityCard';

import './index.less';

export type GraphTableData = {
    data: GraphEdge[];
    total: number;
    page: number;
    pageSize: number;
};

const makeQueryStr = (entityType: string, entityId: string): string => {
    const source_query: ComposeQueryItem = {
        operator: 'and',
        items: [
            {
                field: 'source_type',
                operator: '=',
                value: entityType
            },
            {
                field: 'source_id',
                operator: '=',
                value: entityId
            },
        ],
    };

    const target_query: ComposeQueryItem = {
        operator: 'and',
        items: [
            {
                field: 'target_type',
                operator: '=',
                value: entityType
            },
            {
                field: 'target_id',
                operator: '=',
                value: entityId
            },
        ],
    };

    let query: ComposeQueryItem = {
        operator: 'or',
        items: [source_query, target_query],
    };

    return JSON.stringify(query);
}

const KnowledgeTable: React.FC = (props) => {
    const search = useLocation().search;
    // Such as Disease::MONDO:0005404
    const queriedNodeId = new URLSearchParams(search).get('nodeId') || undefined;
    const queriedNodeName = new URLSearchParams(search).get('nodeName') || undefined;
    const [nodeId, setNodeId] = useState<string | undefined>(undefined);
    const [nodeName, setNodeName] = useState<string | undefined>(undefined);
    const [drawerVisible, setDrawerVisible] = useState<boolean>(false);
    const [edgeInfo, setEdgeInfo] = useState<EdgeInfo | undefined>(undefined);
    const [currentNode, setCurrentNode] = useState<GraphNode | undefined>(undefined);

    const [selectedRowKeys, setSelectedRowKeys] = useState<React.Key[]>([]);
    const [graphData, setGraphData] = useState<GraphData>({} as GraphData);
    const [tableData, setTableData] = useState<any[]>([] as any[]);
    const [loading, setLoading] = useState<boolean>(false);
    const [total, setTotal] = useState<number>(0);
    const [page, setPage] = useState<number>(1);
    const [pageSize, setPageSize] = useState<number>(30);
    const [refreshKey, setRefreshKey] = useState<number>(0);
    const { isAuthenticated } = useAuth0();

    useEffect(() => {
        if (!isAuthenticated) {
            history.push('/not-authorized');
        }
    }, [isAuthenticated])

    useEffect(() => {
        if (queriedNodeId) {
            setNodeId(queriedNodeId);
        }

        if (queriedNodeName) {
            setNodeName(queriedNodeName);
        }
    }, [])

    const onSelectChange = (newSelectedRowKeys: React.Key[]) => {
        console.log('selectedRowKeys changed: ', newSelectedRowKeys);
        setSelectedRowKeys(newSelectedRowKeys);
    };

    const explainGraph = (selectedRowKeys: string[]) => {
        if (selectedRowKeys.length === 0) {
            message.error('Please select at least one edge to explain');
            return;
        }
        const selectedEdges = graphData.edges.filter((edge) => {
            return selectedRowKeys.includes(edge.relid);
        });
        const selectedNodes = graphData.nodes.filter((node) => {
            return selectedEdges.some((edge) => {
                return edge.data.source_id === node.data.id || edge.data.target_id === node.data.id;
            });
        });
        const selectedGraphData = {
            nodes: selectedNodes,
            edges: selectedEdges,
        };
        pushGraphDataToLocalStorage(selectedGraphData);
        history.push('/knowledge-graph');
    }

    const truncateString = (str: string) => {
        if (str.length > 20) {
            return str.substring(0, 20) + '...';
        } else {
            return str;
        }
    }

    const getKnowledgesData = (nodeId: string, page: number, pageSize: number): Promise<GraphData> => {
        return new Promise((resolve, reject) => {
            let pairs = nodeId.split('::');
            const entityType = pairs[0];
            const entityId = pairs[1];
            if (entityType && entityId) {
                fetchOneStepLinkedNodes({
                    query_str: makeQueryStr(entityType, entityId),
                    page_size: pageSize,
                    page: page
                })
                    .then((response) => {
                        resolve(response);
                    })
                    .catch((error) => {
                        reject(error);
                    });
            } else {
                resolve({ nodes: [], edges: [] });
            }
        })
    };

    // EdgeData
    const columns: ColumnsType<any> = [
        {
            title: 'Relation Type',
            key: 'relation_type',
            align: 'left',
            dataIndex: 'relation_type',
            fixed: 'left',
            width: 300,
            filters: sortBy(uniqBy(tableData.map((item) => {
                return {
                    text: item.relation_type,
                    value: item.relation_type,
                };
            }), 'value'), 'value'),
            filterMode: 'menu',
            filterSearch: true,
            sorter: (a, b) => a.relation_type.localeCompare(b.relation_type),
            onFilter: (value, record) => record.relation_type.indexOf(value) === 0,
        },
        {
            title: 'PMID',
            dataIndex: 'pmids',
            align: 'center',
            width: 100,
            key: 'pmids',
            render: (text) => {
                return (
                    <a target="_blank" href={`https://pubmed.ncbi.nlm.nih.gov/?term=${text}`}>
                        {text}
                    </a>
                );
            },
            fixed: 'left',
            filters: sortBy(uniqBy(filter(tableData.map((item) => {
                return {
                    text: item.pmids,
                    value: item.pmids,
                };
            }), (item) => item.text !== ""), 'value'), 'value'),
            filterMode: 'menu',
            filterSearch: true,
            sorter: (a, b) => a.pmids.localeCompare(b.pmids),
            onFilter: (value, record) => record.pmids.indexOf(value) === 0,
        },
        {
            title: 'Source Name',
            dataIndex: 'source_name',
            key: 'source_name',
            align: 'center',
            filters: sortBy(uniqBy(tableData.map((item) => {
                return {
                    text: item.source_name,
                    value: item.source_name,
                };
            }), 'value'), 'value'),
            filterMode: 'menu',
            width: 200,
            filterSearch: true,
            sorter: (a, b) => a.source_name.localeCompare(b.source_name),
            onFilter: (value, record) => record.source_name.indexOf(value) === 0,
            render: (text, record) => {
                const option = record.source_node as GraphNode;
                const nodeData = option.data;
                return <>
                    {nodeData ? (
                        <Popover
                            placement="rightTop"
                            title={
                                <span>
                                    <Tag color={guessColor(nodeData.label)}>{nodeData.label}</Tag>
                                    {nodeData.id} | {nodeData.name}
                                </span>
                            }
                            content={EntityCard({ ...nodeData, idx: 0 })}
                            trigger="hover"
                            getPopupContainer={(triggeredNode: any) => document.body}
                            overlayClassName="entity-id-popover"
                            autoAdjustOverflow={false}
                            destroyTooltipOnHide={true}
                            zIndex={1500}
                        >
                            {truncateString(text)}
                        </Popover>
                    ) : (
                        <Tooltip title={text}>
                            {truncateString(text)}
                        </Tooltip>
                    )}
                    {<br />}
                    {record.source_id.startsWith('ENTREZ:') ?
                        <a onClick={() => { setCurrentNode(record.source_node) }}>
                            {record.source_id}
                        </a> :
                        <a target="_blank" href={guessLink(record.source_id)}>
                            {record.source_id}
                        </a>
                    }
                </>;
            }
        },
        // {
        //     title: 'Source ID',
        //     dataIndex: 'source_id',
        //     align: 'center',
        //     key: 'source_id',
        //     render: (text, record) => {
        //         console.log("Source ID: ", text, record);
        //         return (
        //             text.startsWith('ENTREZ:') ?
        //                 <a onClick={() => { setCurrentNode(record.source_node) }}>
        //                     {text}
        //                 </a> :
        //                 <a target="_blank" href={guessLink(text)}>
        //                     {text}
        //                 </a>
        //         );
        //     }
        // },
        {
            title: 'Source Type',
            dataIndex: 'source_type',
            width: 200,
            align: 'center',
            key: 'source_type',
            filters: sortBy(uniqBy(tableData.map((item) => {
                return {
                    text: item.source_type,
                    value: item.source_type,
                };
            }), 'value'), 'value'),
            filterMode: 'menu',
            filterSearch: true,
            sorter: (a, b) => a.source_type.localeCompare(b.source_type),
            onFilter: (value, record) => record.source_type.indexOf(value) === 0,
            render: (text) => {
                return <Tag color={guessColor(text)}>{text}</Tag>;
            }
        },
        {
            title: 'Target Name',
            dataIndex: 'target_name',
            align: 'center',
            key: 'target_name',
            filters: sortBy(uniqBy(tableData.map((item) => {
                return {
                    text: item.target_name,
                    value: item.target_name,
                };
            }), 'value'), 'value'),
            filterMode: 'menu',
            filterSearch: true,
            sorter: (a, b) => a.target_name.localeCompare(b.target_name),
            onFilter: (value, record) => record.target_name.indexOf(value) === 0,
            render: (text, record) => {
                const option = record.target_node as GraphNode;
                const nodeData = option.data;
                return <>
                    {nodeData ? (
                        <Popover
                            placement="rightTop"
                            title={
                                <span>
                                    <Tag color={guessColor(nodeData.label)}>{nodeData.label}</Tag>
                                    {nodeData.id} | {nodeData.name}
                                </span>
                            }
                            content={EntityCard({ ...nodeData, idx: 0 })}
                            trigger="hover"
                            getPopupContainer={(triggeredNode: any) => document.body}
                            overlayClassName="entity-id-popover"
                            autoAdjustOverflow={false}
                            destroyTooltipOnHide={true}
                            zIndex={1500}
                        >
                            {truncateString(text)}
                        </Popover>
                    ) : (
                        <Tooltip title={text}>
                            {truncateString(text)}
                        </Tooltip>
                    )}
                    {<br />}
                    {record.target_id.startsWith('ENTREZ:') ?
                        <a onClick={() => { setCurrentNode(record.target_node) }}>
                            {record.target_id}
                        </a> :
                        <a target="_blank" href={guessLink(record.target_id)}>
                            {record.target_id}
                        </a>
                    }
                </>;
            }
        },
        // {
        //     title: 'Target ID',
        //     dataIndex: 'target_id',
        //     align: 'center',
        //     key: 'target_id',
        //     render: (text, record) => {
        //         return (
        //             text.startsWith('ENTREZ:') ?
        //                 <a onClick={() => { setCurrentNode(record.target_node) }}>
        //                     {text}
        //                 </a> :
        //                 <a target="_blank" href={guessLink(text)}>
        //                     {text}
        //                 </a>
        //         );
        //     }
        // },
        {
            title: 'Target Type',
            dataIndex: 'target_type',
            align: 'center',
            key: 'target_type',
            filters: sortBy(uniqBy(tableData.map((item) => {
                return {
                    text: item.target_type,
                    value: item.target_type,
                };
            }), 'value'), 'value'),
            filterMode: 'menu',
            filterSearch: true,
            sorter: (a, b) => a.target_type.localeCompare(b.target_type),
            onFilter: (value, record) => record.target_type.indexOf(value) === 0,
            render: (text) => {
                return <Tag color={guessColor(text)}>{text}</Tag>;
            }
        },
        {
            title: 'Score',
            dataIndex: 'score',
            align: 'center',
            key: 'score',
            width: 100,
            render: (text) => {
                return <span>{text.toFixed(3)}</span>;
            },
            sorter: (a, b) => a.score - b.score,
        },
        {
            title: 'Action',
            key: 'operation',
            fixed: 'right',
            align: 'center',
            width: 120,
            render: (text, record) => (
                <Space size="middle">
                    <Popover
                        content={
                            <>
                                <p>Show the top N publications and related information about this knowledge.</p>
                            </>
                        }
                        title="Note"
                    >
                        <Button type="primary" onClick={() => {
                            setEdgeInfo({
                                startNode: record.source_node,
                                endNode: record.target_node,
                                edge: record as GraphEdge
                            });
                            setDrawerVisible(true);
                        }}>Publications</Button>
                    </Popover>
                </Space>
            ),
        }
    ];

    useEffect(() => {
        setLoading(true);
        if (!nodeId) {
            setLoading(false);
            return;
        }

        let pairs = nodeId.split('::');
        const entityType = pairs[0];
        const entityId = pairs[1];
        if (entityType && entityId) {
            fetchRelationCounts({ query_str: makeQueryStr(entityType, entityId) })
                .then((response) => {
                    const n = response.map((item) => item.ncount).reduce((a, b) => a + b, 0);
                    setTotal(n);
                })
                .catch((error) => {
                    console.log('Get relation counts error: ', error);
                    setTotal(0);
                });
        } else {
            // Reset the component
            setTotal(0);
        }

        getKnowledgesData(nodeId, page, pageSize)
            .then((response) => {
                setGraphData(response);
                setLoading(false);
                const edges = response.edges.map((item) => {
                    return {
                        // We need it for the row selection
                        ...item,
                        ...item.data,
                    }
                });
                let tableData = edges.map((item) => {
                    const newItem: any = { ...item };
                    const sourceName = response.nodes.find((node) => node.data.id === item.source_id)?.data.name;
                    const targetName = response.nodes.find((node) => node.data.id === item.target_id)?.data.name;
                    newItem.source_name = sourceName;
                    newItem.source_node = response.nodes.find((node) => node.data.id === item.source_id);
                    newItem.target_name = targetName;
                    newItem.target_node = response.nodes.find((node) => node.data.id === item.target_id);

                    return newItem;
                })
                setTableData(tableData);
            })
            .catch((error) => {
                console.log('Get knowledges error: ', error);
                setGraphData({} as GraphData);
                setLoading(false);
            });
    }, [nodeId, page, pageSize, refreshKey]);

    const getRowKey = (record: GraphEdge) => {
        return record.relid || `${JSON.stringify(record)}`;
    };

    return (total == 0) ? (
        <Row className='knowledge-table-container'>
            <Spin spinning={loading}>
                <Empty description={
                    <>
                        <p>No Knowledges for Your Query {nodeId ? `(${nodeId} ${nodeName ? nodeName : ''})` : ''}</p>
                        <Button type="primary" onClick={() => history.push('/')}>
                            Go Back to Home Page
                        </Button>
                    </>
                }
                    style={{
                        height: '100%',
                        flexDirection: 'column',
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                    }} />
            </Spin>
        </Row>
    ) : (
        <Row className="knowledge-table-container">
            <div className='button-container'>
                <span>
                    Selected {selectedRowKeys.length} items
                    <Tooltip title="You can select several items by clicking on the checkboxes and explain them together.">
                        <Button type="link" style={{ marginLeft: '5px', padding: '4px 0' }}>
                            <QuestionCircleOutlined />Help
                        </Button>
                    </Tooltip>
                </span>
                <Button size="large" onClick={() => {
                    history.push('/')
                }}>
                    Back to Home
                </Button>
                <Button type="primary" danger size="large"
                    // disabled={selectedRowKeys.length === 0}
                    onClick={() => {
                        if (selectedRowKeys.length === 0) {
                            message.warning('Please select at least one row to explain.', 5);
                            return;
                        }
                        explainGraph(selectedRowKeys as string[]);
                    }}>
                    Explain [{nodeName ? nodeName : nodeId}]
                </Button>
            </div>
            <Table
                className={'graph-table'}
                style={{ width: '100%', height: '100%' }}
                size="small"
                columns={columns}
                loading={loading}
                scroll={{ x: 1000, y: 'calc(100vh - 165px)' }}
                dataSource={tableData || []}
                rowSelection={{
                    selectedRowKeys,
                    onChange: onSelectChange,
                }}
                rowKey={(record) => getRowKey(record)}
                expandable={{
                    expandedRowRender: (record) => (
                        <p style={{ margin: 0 }}>
                            <Tag>Key Sentence</Tag> {record.key_sentence || 'No Key Sentence'}
                        </p>
                    ),
                }}
                pagination={{
                    showSizeChanger: true,
                    showQuickJumper: true,
                    pageSizeOptions: ['10', '20', '50', '100', '300', '500', '1000'],
                    current: page,
                    pageSize: pageSize,
                    total: total || 0,
                    position: ['topLeft'],
                    showTotal: (total) => {
                        return `Total ${total} items`;
                    },
                }}
                onChange={(pagination) => {
                    setPage(pagination.current || 1);
                    setPageSize(pagination.pageSize || 10);
                }}
            ></Table>
            <Drawer
                width={'80%'}
                className='node-drawer'
                height={'100%'}
                title={currentNode ? `Node Information - ${currentNode.data.name}` : 'Node Information'}
                rootStyle={{ position: 'absolute' }}
                closable={true}
                mask={true}
                placement={'right'}
                onClose={() => {
                    setCurrentNode(undefined);
                }}
                open={currentNode !== undefined}
            >
                {
                    currentNode ?
                        <NodeInfoPanel node={currentNode} /> :
                        <Empty description="No node data for this knowledge." />
                }
            </Drawer>
            <Drawer
                width={'80%'}
                className='knowledge-drawer'
                height={'100%'}
                title={`Knowledge Information - ${edgeInfo?.startNode?.data.name} - ${edgeInfo?.edge?.reltype} - ${edgeInfo?.endNode?.data.name}`}
                rootStyle={{ position: 'absolute' }}
                closable={true}
                mask={true}
                placement={'right'}
                onClose={() => {
                    setDrawerVisible(false);
                }
                }
                open={drawerVisible}
            >
                {edgeInfo ?
                    <EdgeInfoPanel edgeInfo={edgeInfo} />
                    : <Empty description="No publication data for this knowledge." />}
            </Drawer>
        </Row>
    );
};

export default KnowledgeTable;
