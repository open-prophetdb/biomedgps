import React, { useEffect, useState, forwardRef, useImperativeHandle } from 'react';
import { history } from 'umi';
import { Table, Row, Tag, Space, message, Popover, Button, Input } from 'antd';
import type { ColumnsType } from 'antd/es/table';
import { GraphEdge, GraphTableData } from './typings';
import type { GraphData } from 'biominer-components/dist/typings';
import { pushGraphDataToLocalStorage } from 'biominer-components/dist/KnowledgeGraph/utils';
import { deleteCuratedKnowledge, fetchCuratedGraph, fetchCuratedKnowledges, fetchCuratedKnowledgesByOwner } from '@/services/swagger/KnowledgeGraph';

import './KnowledgeTable.less';

type GraphTableProps = {
    page?: number;
    pageSize?: number;
    pageSizeOptions?: string[];
    className?: string;
    style?: React.CSSProperties;
    yScroll?: number | string;
    xScroll?: number | string;
};

const GraphTable: React.FC<GraphTableProps> = forwardRef((props, ref) => {
    const [graphData, setGraphData] = useState<GraphData>({} as GraphData);
    const [tableData, setTableData] = useState<GraphEdge[]>([]);
    const [loading, setLoading] = useState<boolean>(false);
    const [page, setPage] = useState<number>(props.page || 1);
    const [pageSize, setPageSize] = useState<number>(props.pageSize || 30);
    const [refreshKey, setRefreshKey] = useState<number>(0);
    const [selectedRowKeys, setSelectedRowKeys] = useState<React.Key[]>([]);
    const [total, setTotal] = useState<number>(0);
    const [searchText, setSearchText] = useState<string>('');

    useImperativeHandle(ref, () => ({
        downloadTable() {
            if (!tableData || !tableData.length) {
                message.error('No data to export');
                return;
            }

            const jsonString = JSON.stringify(tableData, null, 2);
            const blob = new Blob([jsonString], { type: 'application/json;charset=utf-8;' });
            const url = window.URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.style.display = 'none';
            a.href = url;
            a.download = 'knowledges.json';
            document.body.appendChild(a);
            a.click();
            window.URL.revokeObjectURL(url);
            document.body.removeChild(a);

            message.success('Download successfully!');
        }
    }));

    const columns: ColumnsType<GraphEdge> = [
        {
            title: 'Relation Type',
            key: 'relation_type',
            align: 'center',
            dataIndex: 'relation_type',
            fixed: 'left',
            width: 240,
        },
        {
            title: 'Source Name',
            dataIndex: 'source_name',
            key: 'source_name',
            align: 'center',
            width: 200,
        },
        {
            title: 'Source ID',
            dataIndex: 'source_id',
            align: 'center',
            key: 'source_id',
            width: 150,
        },
        {
            title: 'Source Type',
            dataIndex: 'source_type',
            align: 'center',
            key: 'source_type',
            width: 120,
        },
        {
            title: 'Target Name',
            dataIndex: 'target_name',
            align: 'center',
            key: 'target_name',
            width: 200,
        },
        {
            title: 'Target ID',
            dataIndex: 'target_id',
            align: 'center',
            key: 'target_id',
            width: 150,
        },
        {
            title: 'Target Type',
            dataIndex: 'target_type',
            align: 'center',
            key: 'target_type',
            width: 120,
        },
        {
            title: 'Curator',
            dataIndex: 'curator',
            align: 'center',
            key: 'curator',
            width: 120,
        },
        {
            title: 'Created Time',
            key: 'created_at',
            align: 'center',
            dataIndex: 'created_at',
            render: (text) => {
                return new Date(text).toLocaleString();
            },
            width: 120,
        },
        {
            title: 'Actions',
            key: 'actions',
            align: 'center',
            fixed: 'left',
            width: 120,
            render: (text, record) => {
                return (
                    <Space>
                        <div>
                            <Popover
                                content={
                                    <div>
                                        <p style={{ marginBottom: '5px' }}>Are you sure to delete this knowledge?</p>
                                        <p style={{ display: 'flex', justifyContent: 'flex-end', marginBottom: '0' }}>
                                            <Button
                                                danger
                                                size="small"
                                                onClick={() => {
                                                    if (
                                                        record.id !== undefined &&
                                                        record.id >= 0
                                                    ) {
                                                        deleteCuratedKnowledge({
                                                            id: record.id
                                                        })
                                                            .then((response: any) => {
                                                                message.success('Delete knowledge successfully!');
                                                                setRefreshKey(refreshKey + 1);
                                                            })
                                                            .catch((error: any) => {
                                                                console.log('Delete knowledge error: ', error);
                                                                message.error('Delete knowledge failed!');
                                                            });
                                                    } else {
                                                        message.error('Delete knowledge failed!');
                                                        console.log(
                                                            'Delete knowledge error: ',
                                                            record,
                                                            deleteCuratedKnowledge
                                                        );
                                                    }
                                                }}
                                            >
                                                Confirm
                                            </Button>
                                        </p>
                                    </div>
                                }
                                title="Comfirm"
                            >
                                <Button danger size="small">
                                    Delete
                                </Button>
                            </Popover>
                        </div>
                    </Space>
                );
            },
        },
    ];

    const makeSearchQuery = (searchText: string) => {
        if (!searchText) {
            return undefined;
        }

        const composedQuery = {
            operator: 'or',
            items: [{
                operator: 'ilike',
                field: 'source_name',
                value: `%${searchText}%`,
            }, {
                operator: 'ilike',
                field: 'target_name',
                value: `%${searchText}%`,
            }, {
                operator: 'ilike',
                field: 'relation_type',
                value: `%${searchText}%`,
            }, {
                operator: 'ilike',
                field: 'curator',
                value: `%${searchText}%`,
            }],
        }

        return JSON.stringify(composedQuery);
    };

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

    useEffect(() => {
        setLoading(true);
        fetchCuratedKnowledgesByOwner({
            page: page,
            page_size: pageSize,
            query_str: makeSearchQuery(searchText)
        }).then((response) => {
            setTotal(response.total);
            setTableData(response.records);
            setLoading(false);
        }).catch((error) => {
            console.log('Get knowledges error: ', error);
            setTotal(0);
            setLoading(false);
        });

        // Cannot get all curated knowledges by fetchCuratedGraph, because the backend does not support to query all curated knowledges for admin.
        // TODO: How to explain the selected knowledges in the graph?
        // fetchCuratedGraph({
        //     page: page,
        //     page_size: pageSize,
        //     strict_mode: true,
        //     project_id: '-1',
        //     organization_id: '-1',
        // })
        //     .then((response) => {
        //         setGraphData(response);
        //         setLoading(false);
        //         const edges = response.edges.map((item) => {
        //             return {
        //                 // We need it for the row selection
        //                 ...item,
        //                 ...item.data,
        //             }
        //         });

        //         let tableData = edges.map((item) => {
        //             const newItem: any = { ...item };
        //             const sourceName = response.nodes.find((node) => node.data.id === item.source_id)?.data.name;
        //             const targetName = response.nodes.find((node) => node.data.id === item.target_id)?.data.name;
        //             newItem.source_name = sourceName;
        //             newItem.source_node = response.nodes.find((node) => node.data.id === item.source_id);
        //             newItem.target_name = targetName;
        //             newItem.target_node = response.nodes.find((node) => node.data.id === item.target_id);

        //             return newItem;
        //         })
        //         setTableData(tableData);
        //         setLoading(false);
        //     })
        //     .catch((error) => {
        //         console.log('Get knowledges error: ', error);
        //         setGraphData({} as GraphData);
        //         setTableData([]);
        //         setLoading(false);
        //     });
    }, [page, pageSize, refreshKey, searchText]);

    const getRowKey = (record: GraphEdge) => {
        // return `${record.source_id}-${record.target_id}-${record.relation_type}-${record.pmid}-${record.curator}`;
        return record.id || `${JSON.stringify(record)}`;
    };

    return (
        <Row className="graph-table-container">
            <Input.Search
                placeholder="Search by Keywords [AI Search Functionality Coming Soon]"
                allowClear
                enterButton="Search"
                size="middle"
                onSearch={(value) => {
                    setSearchText(value);
                }}
                style={{ marginBottom: '10px' }}
            />
            <Table
                className={props.className ? props.className + ' graph-table' : 'graph-table'}
                style={props.style}
                size="small"
                columns={columns}
                loading={loading}
                scroll={{ x: props.xScroll || 1000, y: props.yScroll || 'calc(100vh - 280px)' }}
                dataSource={tableData}
                // TODO: How to explain the selected knowledges in the graph?
                // rowSelection={{
                //     selectedRowKeys,
                //     onChange: onSelectChange,
                // }}
                rowKey={(record) => getRowKey(record)}
                expandable={{
                    expandedRowRender: (record) => (
                        <p style={{ margin: 0 }}>
                            <Tag>Key Sentence</Tag> {record.key_sentence || 'No Key Sentence'}
                            <br />
                            <Tag>Curator</Tag> {record.curator || 'Unknown'}
                        </p>
                    ),
                }}
                pagination={{
                    showSizeChanger: true,
                    showQuickJumper: true,
                    pageSizeOptions: props.pageSizeOptions || ['10', '20', '50', '100', '300', '500', '1000'],
                    current: page,
                    pageSize: pageSize,
                    total: total || 0,
                    position: ['bottomRight'],
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
});

export default GraphTable;
