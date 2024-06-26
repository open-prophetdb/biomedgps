import React, { useEffect, useState } from 'react';
import { history } from 'umi';
import { Table, Row, Tag, Space, message, Popover, Button, Empty, Tooltip, Drawer, Spin, Select, Collapse } from 'antd';
import { ArrowsAltOutlined, DownloadOutlined, ExpandAltOutlined, QuestionCircleOutlined, ShrinkOutlined } from '@ant-design/icons';
import type { ColumnsType } from 'antd/es/table';
import { useLocation } from "react-router-dom";
import { fetchOneStepLinkedNodes, fetchRelationCounts, fetchRelationMetadata } from '@/services/swagger/KnowledgeGraph';
import type { ComposeQueryItem, OptionType, GraphData, GraphEdge, GraphNode, RelationCount } from 'biominer-components/dist/typings';
import { guessLink, makeRelationTypes } from 'biominer-components/dist/utils';
import { pushGraphDataToLocalStorage } from 'biominer-components/dist/KnowledgeGraph/utils';
import type { EdgeInfo } from '@/EdgeInfoPanel/index.t';
import NodeInfoPanel from '@/NodeInfoPanel';
import EdgeInfoPanel from '@/EdgeInfoPanel';
import { sortBy, filter, uniqBy, groupBy, map, sumBy, set } from 'lodash';
import { guessColor, truncateString, getUsername, logoutWithRedirect, isAuthEnabled, isAuthenticated } from '@/components/util';
import EntityCard from '@/components/EntityCard';

import './index.less';

export type GraphTableData = {
    data: GraphEdge[];
    total: number;
    page: number;
    pageSize: number;
};

const isValidNodeIds = (nodeIds: string[] | undefined): boolean => {
    // Check whether the label is same among the nodeIds
    if (nodeIds && nodeIds.length > 0) {
        const label = nodeIds[0].split('::')[0];
        return nodeIds.every((nodeId) => nodeId.split('::')[0] === label);
    }

    return false;
}

const makeQueryStr = (nodeIds: string[], relationTypes?: string[], resources?: string[]): string => {
    // The source_type and target_type must be the same among the nodeIds
    const source_query: ComposeQueryItem = {
        operator: 'and',
        items: [
            {
                field: 'source_type',
                operator: 'in',
                value: nodeIds.map((nodeId) => nodeId.split('::')[0]),
            },
            {
                field: 'source_id',
                operator: 'in',
                value: nodeIds.map((nodeId) => nodeId.split('::')[1]),
            },
        ],
    };

    const target_query: ComposeQueryItem = {
        operator: 'and',
        items: [
            {
                field: 'target_type',
                operator: 'in',
                value: nodeIds.map((nodeId) => nodeId.split('::')[0]),
            },
            {
                field: 'target_id',
                operator: 'in',
                value: nodeIds.map((nodeId) => nodeId.split('::')[1]),
            },
        ],
    };

    let query: ComposeQueryItem = {
        operator: 'or',
        items: [source_query, target_query],
    };

    if (relationTypes && relationTypes.length > 0) {
        query = {
            operator: 'and',
            items: [
                query,
                {
                    field: 'relation_type',
                    operator: 'in',
                    value: relationTypes,
                },
            ],
        };
    }

    if (resources && resources.length > 0) {
        query = {
            operator: 'and',
            items: [
                query,
                {
                    field: 'resource',
                    operator: 'in',
                    value: resources,
                },
            ],
        };
    }

    return JSON.stringify(query);
}

type KnowledgeTableProps = {
    nodeId?: string;
};

const KnowledgeTable: React.FC<KnowledgeTableProps> = (props) => {
    const search = useLocation().search;
    // Such as Disease::MONDO:0005404
    const queriedNodeId = new URLSearchParams(search).get('nodeId') || undefined;
    const queriedNodeIds = new URLSearchParams(search).get('nodeIds') || undefined;

    const [nodeIds, setNodeIds] = useState<string[] | undefined>(undefined);
    const [drawerVisible, setDrawerVisible] = useState<boolean>(false);
    const [edgeInfo, setEdgeInfo] = useState<EdgeInfo | undefined>(undefined);
    const [currentNodes, setCurrentNodes] = useState<(GraphNode | undefined)[]>([]);
    const [activatedNode, setActivatedNode] = useState<GraphNode | undefined>(undefined);
    const [activeKeys, setActiveKeys] = useState<string[]>(['0', '1', '2', '3']);
    const [relationTypeOptions, setRelationTypeOptions] = useState<OptionType[]>([]);
    const [selectedRelationTypes, setSelectedRelationTypes] = useState<string[]>([]);
    const [relationTypeDescs, setRelationTypeDescs] = useState<Record<string, string>>({});
    const [resources, setResources] = useState<OptionType[]>([]);
    const [selectedResources, setSelectedResources] = useState<string[]>([]);

    const [selectedRowKeys, setSelectedRowKeys] = useState<React.Key[]>([]);
    const [graphData, setGraphData] = useState<GraphData>({} as GraphData);
    const [tableData, setTableData] = useState<any[]>([] as any[]);
    const [loading, setLoading] = useState<boolean>(false);
    const [total, setTotal] = useState<number>(0);
    const [page, setPage] = useState<number>(1);
    const [pageSize, setPageSize] = useState<number>(30);
    const [refreshKey, setRefreshKey] = useState<number>(0);

    useEffect(() => {
        if (!isAuthenticated()) {
            logoutWithRedirect();
        } else {

            if (queriedNodeId || queriedNodeIds || props.nodeId) {
                if (queriedNodeId) {
                    setNodeIds([queriedNodeId]);
                } else {
                    setNodeIds(undefined);
                }

                if (queriedNodeIds) {
                    const ids = queriedNodeIds.split(',');
                    if (ids.length > 0) {
                        setNodeIds(ids);
                    } else {
                        setNodeIds(undefined);
                    }
                }

                if (props.nodeId) {
                    setNodeIds([props.nodeId]);
                }

                fetchRelationMetadata().then((relationStat) => {
                    const o = makeRelationTypes(relationStat)
                    let descs = {} as Record<string, string>;
                    o.forEach((item) => {
                        descs[item.value] = item.description || 'Unknown';
                    });
                    setRelationTypeDescs(descs);

                    let res = [] as OptionType[];
                    relationStat.forEach((item, index) => {
                        res.push({
                            order: 0,
                            label: item.resource,
                            value: item.resource,
                        });
                    });

                    setResources(uniqBy(res, 'value'));
                });
            }
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
        history.push('/predict-explain/knowledge-graph');
    }

    const getKnowledgesData = (
        nodeIds: string[],
        page: number,
        pageSize: number,
        relationTypes?: string[],
        resources?: string[]
    ): Promise<GraphData> => {
        return new Promise((resolve, reject) => {
            if (nodeIds && nodeIds.length > 0) {
                fetchOneStepLinkedNodes({
                    query_str: makeQueryStr(nodeIds, relationTypes, resources),
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
            width: 350,
            filters: sortBy(uniqBy(tableData.map((item) => {
                return {
                    text: item.relation_type,
                    value: item.relation_type,
                };
            }), 'value'), 'value'),
            filterMode: 'menu',
            filterSearch: true,
            filterMultiple: true,
            sorter: (a, b) => a.relation_type.localeCompare(b.relation_type),
            onFilter: (value, record) => record.relation_type.indexOf(value) === 0,
            render(text, record) {
                return (
                    <span>
                        <Tag>{text}</Tag>
                        <br />
                        {relationTypeDescs[text] || 'Unknown'}
                    </span>
                );
            }
        },
        {
            title: 'Resource',
            dataIndex: 'resource',
            key: 'resource',
            align: 'center',
            width: 100,
            // fixed: 'left',
        },
        // {
        //     title: 'PMID',
        //     dataIndex: 'pmids',
        //     align: 'center',
        //     width: 100,
        //     key: 'pmids',
        //     render: (text) => {
        //         return (
        //             <a target="_blank" href={`https://pubmed.ncbi.nlm.nih.gov/?term=${text}`}>
        //                 {text}
        //             </a>
        //         );
        //     },
        //     filters: sortBy(uniqBy(filter(tableData.map((item) => {
        //         return {
        //             text: item.pmids,
        //             value: item.pmids,
        //         };
        //     }), (item) => item.text !== ""), 'value'), 'value'),
        //     filterMode: 'menu',
        //     filterSearch: true,
        //     sorter: (a, b) => a.pmids.localeCompare(b.pmids),
        //     onFilter: (value, record) => record.pmids.indexOf(value) === 0,
        // },
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
            // width: 200,
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
                                <span className='entity-id-popover-title'>
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
                    {(record.source_id.startsWith('ENTREZ:') || record.source_id.startsWith('DrugBank:')) ?
                        <a onClick={() => { setActivatedNode(record.source_node) }}>
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
        //                 <a onClick={() => { setActivatedNode(record.source_node) }}>
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
            // width: 200,
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
                                <span className='entity-id-popover-title'>
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
                    {(record.target_id.startsWith('ENTREZ:') || record.target_id.startsWith('DrugBank:')) ?
                        <a onClick={() => { setActivatedNode(record.target_node) }}>
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
        //                 <a onClick={() => { setActivatedNode(record.target_node) }}>
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
            width: 200,
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
                        overlayClassName='popover-note'
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

    const fetchTableData = async (nodeIds: string[], page: number, pageSize: number, relationTypes?: string[], resources?: string[]) => {
        setLoading(true);
        if (nodeIds && nodeIds.length > 0) {
            fetchRelationCounts({ query_str: makeQueryStr(nodeIds, relationTypes, resources) })
                .then((response) => {
                    const n = response.map((item) => item.ncount).reduce((a, b) => a + b, 0);
                    setTotal(n);

                    if (!relationTypes || (relationTypes && relationTypes.length === 0)) {
                        const r = response.map((item) => {
                            return {
                                relation_type: item.relation_type,
                                ncount: item.ncount,
                            };
                        });
                        const mergedR = sortBy(map(groupBy(r, 'relation_type'), (group: any, key: string) => ({
                            relation_type: key,
                            ncount: sumBy(group, 'ncount'),
                        })), 'ncount').reverse();
                        setRelationTypeOptions(mergedR.map((item, index) => {
                            return {
                                order: index,
                                label: `[${item.ncount}] ${item.relation_type}`,
                                value: item.relation_type,
                            };
                        }));
                    }
                })
                .catch((error) => {
                    console.log('Get relation counts error: ', error);
                    setTotal(0);
                    setRelationTypeOptions([]);
                });
        } else {
            // Reset the component
            setTotal(0);
        }

        getKnowledgesData(nodeIds, page, pageSize, relationTypes, resources)
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

                if (edges.length > 0) {
                    const nodes = nodeIds.map((nodeId) => {
                        const entityId = nodeId.split('::')[1];
                        return response.nodes.find((node) => node.data.id === entityId);
                    });
                    setCurrentNodes(nodes);
                    setActiveKeys(getDefaultKeys(nodes).slice(-1));
                };

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
    }

    useEffect(() => {
        if (!nodeIds || !isValidNodeIds(nodeIds)) {
            return;
        }

        fetchTableData(nodeIds, page, pageSize, selectedRelationTypes, selectedResources);
    }, [nodeIds, page, pageSize, refreshKey]);

    const getRowKey = (record: GraphEdge) => {
        return record.relid || `${JSON.stringify(record)}`;
    };

    const getDefaultKeys = (currentNodes: (GraphNode | undefined)[]) => {
        return currentNodes?.map((node, index) => index.toString()).concat([`${currentNodes.length + 1}`]);
    }

    const switchCollapsePanel = (key: string | string[]) => {
        if (!Array.isArray(key)) {
            key = [key];
        }

        console.log('Switched key: ', key);
        const defaultKeys = getDefaultKeys(currentNodes);
        if (key.length === 0) {
            setActiveKeys(defaultKeys);
        } else {
            setActiveKeys(key);
        }
    }

    const getExtraButton = (index: string) => {
        return <Button type="default" icon={activeKeys.indexOf(index) < 0 ? <ArrowsAltOutlined /> : <ShrinkOutlined />}
            onClick={() => {
                switchCollapsePanel(activeKeys.filter((key) => key !== index));
            }}
        >
            {activeKeys.indexOf(index) < 0 ? 'Expand' : 'Collapse'}
        </Button>;
    }

    const getTitle = (node: GraphNode) => {
        return <span>
            {node?.nlabel} Card - {node?.data.name} [Click to show more details]
            {/* <Tooltip title="Click the panel header or `Expand` button to show more details">
                <Button type="link" style={{ marginLeft: '5px', padding: '4px 0' }}>
                    <QuestionCircleOutlined />Help
                </Button>
            </Tooltip> */}
        </span>;
    }

    return (isAuthenticated()) && (total == 0 ? (
        <Row className='knowledge-table-container'>
            <Spin spinning={loading}>
                <Empty description={
                    <>
                        <p>
                            {
                                loading ? 'Loading Knowledges' : 'No Knowledges'
                            }

                            for Your Query

                            {
                                nodeIds ? `( ${nodeIds.join(', ')} )` : ''
                            }
                        </p>
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
        <Row className='knowledge-table-wrapper'>
            <Collapse activeKey={activeKeys} ghost onChange={switchCollapsePanel}>
                {currentNodes.length > 0 && currentNodes.map((node, index) => {
                    return node?.nlabel == 'Gene' ? <Collapse.Panel header={getTitle(node)} key={index} showArrow={false}
                        extra={getExtraButton(index.toString())}
                    >
                        <NodeInfoPanel node={node} key={`${index}`} />
                    </Collapse.Panel> : null;
                })}
                <Collapse.Panel header="Knowledges" key={`${currentNodes.length + 1}`} showArrow={false}>
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
                            <Select
                                mode="multiple"
                                allowClear
                                maxTagCount={2}
                                // It's not working, I use css to limit the tag text length instead.
                                // maxTagTextLength={12}
                                style={{ width: '210px', marginRight: '10px' }}
                                size="large"
                                placeholder="Please select resources to filter."
                                defaultValue={[]}
                                onChange={(value: string[]) => {
                                    if (nodeIds) {
                                        fetchTableData(nodeIds, page, pageSize, selectedRelationTypes, value);
                                        setSelectedResources(value);

                                        // The total number of items has been changed, so we need to reset the page and page size.
                                        setPage(1);
                                        setPageSize(30);
                                    }
                                }}
                                options={resources}
                            />
                            <Select
                                mode="multiple"
                                allowClear
                                maxTagCount={2}
                                // It's not working, I use css to limit the tag text length instead.
                                // maxTagTextLength={12}
                                style={{ width: '350px', marginRight: '10px' }}
                                size="large"
                                placeholder="Please select relation types to filter."
                                defaultValue={[]}
                                onChange={(value: string[]) => {
                                    if (nodeIds) {
                                        fetchTableData(nodeIds, page, pageSize, value, selectedResources);
                                        setSelectedRelationTypes(value);

                                        // The total number of items has been changed, so we need to reset the page and page size.
                                        setPage(1);
                                        setPageSize(30);
                                    }
                                }}
                            // options={relationTypeOptions}
                            >
                                {relationTypeOptions.map((item: OptionType) => {
                                    return (
                                        <Select.Option key={item.value} value={item.value}>
                                            <div className="option-container">
                                                <div className="option-label">{item.label}</div>
                                                <div className="option-description">{item.description || relationTypeDescs[item.value] || 'Unknown'}</div>
                                            </div>
                                        </Select.Option>
                                    );
                                })}
                            </Select>
                            <Tooltip title="Download the table data as a TSV file.">
                                <Button size="large" type="default" onClick={() => {
                                    // Download as TSV file
                                    const header = columns.map((col) => col.title);
                                    const data = tableData.map((record) => {
                                        return columns.map((col: any) => {
                                            return record[col.key];
                                        });
                                    });
                                    const tsvData = [header, ...data].map((row) => row.join('\t')).join('\n');
                                    const blob = new Blob([tsvData], { type: 'text/tsv' });
                                    const url = URL.createObjectURL(blob);
                                    const a = document.createElement('a');
                                    a.href = url;
                                    a.download = `knowledges-${nodeIds?.join('-')}-${new Date().toISOString()}.tsv`;
                                    a.click();

                                    // Delete the url
                                    URL.revokeObjectURL(url);
                                }} icon={<DownloadOutlined />} />
                            </Tooltip>
                            <Button type="primary" danger size="large"
                                // disabled={selectedRowKeys.length === 0}
                                onClick={() => {
                                    if (selectedRowKeys.length === 0) {
                                        message.warning('Please select at least one row to explain.', 5);
                                        return;
                                    }
                                    explainGraph(selectedRowKeys as string[]);
                                }}>
                                Explain
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
                                showQuickJumper: false,
                                pageSizeOptions: ['10', '20', '50', '100', '300', '500'],
                                current: page,
                                pageSize: pageSize,
                                total: total || 0,
                                position: ['topLeft'],
                                showTotal: (total) => {
                                    return `Total ${total} records`;
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
                            title={activatedNode ? `Node Information - ${activatedNode.data.name}` : 'Node Information'}
                            rootStyle={{ position: 'absolute' }}
                            closable={true}
                            mask={true}
                            placement={'right'}
                            onClose={() => {
                                setActivatedNode(undefined);
                            }}
                            open={activatedNode !== undefined}
                        >
                            {
                                activatedNode ?
                                    <NodeInfoPanel node={activatedNode} /> :
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
                </Collapse.Panel>
            </Collapse>
        </Row>
    ));
};

export default KnowledgeTable;
