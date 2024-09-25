import React, { useEffect, useState } from 'react';
import { Table, Row, Tag, Space, message, Popover, Button } from 'antd';
import type { ColumnsType } from 'antd/es/table';
import { EntityCuration, EntityTableData } from './typings';
import { deleteEntityCuration, fetchEntityCurationByOwner } from '@/services/swagger/KnowledgeGraph';

type KeySentenceTableProps = {
    page?: number;
    pageSize?: number;
    pageSizeOptions?: string[];
    className?: string;
    style?: React.CSSProperties;
    yScroll?: number | string;
    xScroll?: number | string;
};

const KeySentenceTable: React.FC<KeySentenceTableProps> = (props) => {
    const [data, setData] = useState<EntityTableData>({} as EntityTableData);
    const [loading, setLoading] = useState<boolean>(false);
    const [page, setPage] = useState<number>(props.page || 1);
    const [pageSize, setPageSize] = useState<number>(props.pageSize || 30);
    const [refreshKey, setRefreshKey] = useState<number>(0);

    const columns: ColumnsType<EntityCuration> = [
        {
            title: 'Webpage',
            key: 'webpage',
            align: 'center',
            dataIndex: 'webpage',
            fixed: 'left',
            width: 120,
            render: (text) => {
                return <a target="_blank" href={text}>Review in Webpage</a>;
            },
        },
        {
            title: 'Entity ID',
            dataIndex: 'entity_id',
            key: 'entity_id',
            align: 'center',
            width: 100,
        },
        {
            title: 'Entity Type',
            dataIndex: 'entity_type',
            key: 'entity_type',
            align: 'center',
            width: 50,
        },
        {
            title: 'Entity Name',
            key: 'entity_name',
            align: 'center',
            dataIndex: 'entity_name',
            fixed: 'left',
            width: 100,
        },
        {
            title: 'Fingerprint',
            dataIndex: 'fingerprint',
            align: 'center',
            key: 'fingerprint',
            render: (text) => {
                const getText = (text: string) => {
                    if (text.startsWith('http')) {
                        return text;
                    } else {
                        return `${text.split(':')[0].toUpperCase()} | ${text.split(':')[1]}`;
                    }
                };

                let link = text;
                if (text.startsWith('pmid:')) {
                    link = `https://pubmed.ncbi.nlm.nih.gov/?term=${text.split(':')[1]}`;
                } else if (text.startsWith('doi:')) {
                    link = `https://doi.org/${text.split(':')[1]}`;
                }

                return <a target="_blank" href={link}>
                    {getText(text)}
                </a>;
            },
            fixed: 'left',
            ellipsis: true,
            width: 150,
        },
        {
            title: 'Created Time',
            key: 'created_at',
            align: 'center',
            dataIndex: 'created_at',
            render: (text) => {
                return new Date(text).toLocaleString();
            },
            width: 150,
        },
        {
            title: 'Actions',
            key: 'actions',
            align: 'center',
            fixed: 'right',
            width: 50,
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
                                                        deleteEntityCuration({
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
                                                            deleteEntityCuration
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

    useEffect(() => {
        setLoading(true);
        fetchEntityCurationByOwner({
            page: page,
            page_size: pageSize,
        })
            .then((response) => {
                setData({
                    data: response.records.map((record: any) => ({
                        ...record,
                        webpage: record.annotation.uri
                    })),
                    total: response.total,
                    page: response.page,
                    pageSize: response.page_size,
                });
                setLoading(false);
            })
            .catch((error) => {
                console.log('Get knowledges error: ', error);
                setData({} as EntityTableData);
                setLoading(false);
            });
    }, [page, pageSize, refreshKey]);

    const getRowKey = (record: EntityCuration) => {
        return record.id || `${JSON.stringify(record)}`;
    };

    return (
        <Row className="key-sentence-table-container">
            <Table
                className={props.className ? props.className + ' key-sentence-table' : 'key-sentence-table'}
                style={props.style}
                size="small"
                columns={columns}
                loading={loading}
                scroll={{ x: props.xScroll || 1000, y: props.yScroll || 'calc(100vh - 280px)' }}
                dataSource={data.data}
                rowKey={(record) => getRowKey(record)}
                pagination={{
                    showSizeChanger: true,
                    showQuickJumper: true,
                    pageSizeOptions: props.pageSizeOptions || ['10', '20', '50', '100', '300', '500', '1000'],
                    current: page,
                    pageSize: pageSize,
                    total: data.total || 0,
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
};

export default KeySentenceTable;
