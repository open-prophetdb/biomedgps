import React, { useEffect, useState } from 'react';
import { history } from 'umi';
import { Table, Row, Tag, Space, message, Popover, Button } from 'antd';
import type { ColumnsType } from 'antd/es/table';
import { useAuth0 } from "@auth0/auth0-react";
import { fetchOneStepLinkedNodes, fetchRelationCounts } from '@/services/swagger/KnowledgeGraph';
import type { ComposeQueryItem, QueryItem, GraphData, GraphEdge, GraphNode } from 'biominer-components/dist/typings';
import { guessLink } from 'biominer-components/dist/utils';
import { sortBy, filter, uniqBy } from 'lodash';
import { pushGraphDataToLocalStorage } from 'biominer-components/dist/KnowledgeGraph/utils';

import './index.less';

export type GraphTableData = {
    data: GraphEdge[];
    total: number;
    page: number;
    pageSize: number;
};

const makeQueryStr = (): string => {
    const fsource_query: ComposeQueryItem = {
        operator: 'and',
        items: [
            {
                field: 'source_type',
                operator: '=',
                value: "Disease",
            },
            {
                field: 'source_id',
                operator: '=',
                value: "MONDO:0005404",
            },
        ],
    };

    const ssource_query: ComposeQueryItem = {
        operator: 'and',
        items: [
            {
                field: 'source_type',
                operator: '=',
                value: "Disease",
            },
            {
                field: 'source_id',
                operator: '=',
                value: "MONDO:0100233",
            },
        ],
    };

    const ftarget_query: ComposeQueryItem = {
        operator: 'and',
        items: [
            {
                field: 'target_type',
                operator: '=',
                value: "Disease",
            },
            {
                field: 'target_id',
                operator: '=',
                value: "MONDO:0005404",
            },
        ],
    };

    const starget_query: ComposeQueryItem = {
        operator: 'and',
        items: [
            {
                field: 'target_type',
                operator: '=',
                value: "Disease",
            },
            {
                field: 'target_id',
                operator: '=',
                value: "MONDO:0100233",
            },
        ],
    };

    let query: ComposeQueryItem = {
        operator: 'or',
        items: [fsource_query, ftarget_query, ssource_query, starget_query],
    };

    return JSON.stringify(query);
}

const KnowledgeTable: React.FC = (props) => {
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
        fetchRelationCounts({ query_str: makeQueryStr() })
            .then((response) => {
                const n = response.map((item) => item.ncount).reduce((a, b) => a + b, 0);
                setTotal(n);
            })
            .catch((error) => {
                console.log('Get relation counts error: ', error);
                setTotal(0);
            });
    }, []);

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

    const getKnowledgesData = (page: number, pageSize: number): Promise<GraphData> => {
        return new Promise((resolve, reject) => {
            fetchOneStepLinkedNodes({
                query_str: makeQueryStr(),
                page_size: pageSize,
                page: page
            })
                .then((response) => {
                    resolve(response);
                })
                .catch((error) => {
                    reject(error);
                });
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
        },
        {
            title: 'Source ID',
            dataIndex: 'source_id',
            align: 'center',
            key: 'source_id',
            render: (text) => {
                return (
                    <a target="_blank" href={guessLink(text)}>
                        {text}
                    </a>
                );
            }
        },
        {
            title: 'Source Type',
            dataIndex: 'source_type',
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
        },
        {
            title: 'Target Name',
            dataIndex: 'target_name',
            align: 'center',
            key: 'target_name',
            width: 300,
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
        },
        {
            title: 'Target ID',
            dataIndex: 'target_id',
            align: 'center',
            key: 'target_id',
            render: (text) => {
                return (
                    <a target="_blank" href={guessLink(text)}>
                        {text}
                    </a>
                );
            }
        },
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
        },
        {
            title: 'Score',
            dataIndex: 'score',
            align: 'center',
            key: 'score',
            sorter: (a, b) => a.score - b.score,
        }
    ];

    useEffect(() => {
        setLoading(true);
        getKnowledgesData(page, pageSize)
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
                    newItem.target_name = targetName;

                    return newItem;
                })
                setTableData(tableData);
            })
            .catch((error) => {
                console.log('Get knowledges error: ', error);
                setGraphData({} as GraphData);
                setLoading(false);
            });
    }, [page, pageSize, refreshKey]);

    const getRowKey = (record: GraphEdge) => {
        return record.relid || `${JSON.stringify(record)}`;
    };

    return (
        <Row className="knowledge-table-container">
            <div className='button-container'>
                <Button type="primary" danger size="small" disabled={selectedRowKeys.length === 0}
                    onClick={() => explainGraph(selectedRowKeys as string[])}>
                    Explain
                </Button>
                <span>
                    Selected {selectedRowKeys.length} items [You can select several items by clicking on the checkboxes and explain them together]
                </span>
            </div>
            <Table
                className={'graph-table'}
                style={{ width: '100%', height: '100%' }}
                size="small"
                columns={columns}
                loading={loading}
                scroll={{ x: 1000, y: 'calc(100vh - 140px)' }}
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
                    position: ['topRight'],
                    showTotal: (total) => {
                        return `Total ${total} items`;
                    },
                }}
                onChange={(pagination) => {
                    setPage(pagination.current || 1);
                    setPageSize(pagination.pageSize || 10);
                }}
            ></Table>
        </Row>
    );
};

export default KnowledgeTable;
