import React, { useEffect, useState, forwardRef, useImperativeHandle } from 'react';
import { Table, Row, Tag, Space, message, Popover, Button, Input } from 'antd';
import type { ColumnsType } from 'antd/es/table';
import { KeySentence, KeySentenceTableData } from './typings';
import { deleteKeySentenceCuration, fetchKeySentenceCurationByOwner } from '@/services/swagger/KnowledgeGraph';

type KeySentenceTableProps = {
    page?: number;
    pageSize?: number;
    pageSizeOptions?: string[];
    className?: string;
    style?: React.CSSProperties;
    yScroll?: number | string;
    xScroll?: number | string;
};

const KeySentenceTable: React.FC<KeySentenceTableProps> = forwardRef((props, ref) => {
    const [data, setData] = useState<KeySentenceTableData>({} as KeySentenceTableData);
    const [loading, setLoading] = useState<boolean>(false);
    const [page, setPage] = useState<number>(props.page || 1);
    const [pageSize, setPageSize] = useState<number>(props.pageSize || 30);
    const [refreshKey, setRefreshKey] = useState<number>(0);
    const [searchText, setSearchText] = useState<string>('');

    useImperativeHandle(ref, () => ({
        downloadTable() {
            const csv = data.data.map((row) => Object.values(row).join('\t')).join('\n');
            const blob = new Blob([csv], { type: 'text/csv;charset=utf-8;' });
            const url = window.URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.style.display = 'none';
            a.href = url;
            a.download = 'key_sentences.tsv';
            document.body.appendChild(a);
            a.click();
            window.URL.revokeObjectURL(url);
            document.body.removeChild(a);
            message.success('Download successfully!');
        }
    }));

    const columns: ColumnsType<KeySentence> = [
        {
            title: 'Review',
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
            title: 'Key Sentence',
            key: 'key_sentence',
            align: 'center',
            dataIndex: 'key_sentence',
            fixed: 'left',
            width: 240,
        },
        {
            title: 'Fingerprint',
            dataIndex: 'fingerprint',
            align: 'center',
            key: 'fingerprint',
            render: (text) => {
                let link = text;
                if (text.startsWith('pmid:')) {
                    link = `https://pubmed.ncbi.nlm.nih.gov/?term=${text.split(':')[1]}`;
                } else if (text.startsWith('doi:')) {
                    link = `https://doi.org/${text.split(':')[1]}`;
                }

                return <a target="_blank" href={link}>{text}</a>;
            },
            fixed: 'left',
            ellipsis: true,
            width: 120,
        },
        {
            title: 'Description',
            dataIndex: 'description',
            key: 'description',
            align: 'center',
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
            width: 120,
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
                                                        deleteKeySentenceCuration({
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
                                                            deleteKeySentenceCuration
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
                field: 'key_sentence',
                value: `%${searchText}%`,
            }, {
                operator: 'ilike',
                field: 'description',
                value: `%${searchText}%`,
            }],
        }

        return JSON.stringify(composedQuery);
    };

    useEffect(() => {
        setLoading(true);
        fetchKeySentenceCurationByOwner({
            page: page,
            page_size: pageSize,
            query_str: makeSearchQuery(searchText)
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
                setData({} as KeySentenceTableData);
                setLoading(false);
            });
    }, [page, pageSize, refreshKey, searchText]);

    const getRowKey = (record: KeySentence) => {
        return record.id || `${JSON.stringify(record)}`;
    };

    return (
        <Row className="key-sentence-table-container">
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
                className={props.className + ' key-sentence-table'}
                style={props.style}
                size="small"
                columns={columns}
                loading={loading}
                scroll={{ x: props.xScroll || 1000, y: props.yScroll || 'calc(100vh - 240px)' }}
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
});

export default KeySentenceTable;
