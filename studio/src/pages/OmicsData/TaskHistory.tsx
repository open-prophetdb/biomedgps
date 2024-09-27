import React, { useEffect, useState, forwardRef, useImperativeHandle } from 'react';
import { Table, Row, Tag, Space, message, Popover, Button, Input } from 'antd';
import type { ColumnsType } from 'antd/es/table';
import { TaskHistory, TaskHistoryTableData } from '../../StatEngine/components/WorkflowList/data';
import { fetchTasks } from '@/services/swagger/KnowledgeGraph';

type TaskHistoryTableProps = {
    page?: number;
    pageSize?: number;
    pageSizeOptions?: string[];
    className?: string;
    style?: React.CSSProperties;
    yScroll?: number | string;
    xScroll?: number | string;
    refreshKey?: number;
    onTaskHistoryClick?: (taskHistory: TaskHistory) => void;
};

const isFinished = (taskHistory: TaskHistory) => {
    return taskHistory.status === 'Completed';
};

const TaskHistoryTable: React.FC<TaskHistoryTableProps> = forwardRef((props, ref) => {
    const [data, setData] = useState<TaskHistoryTableData>({} as TaskHistoryTableData);
    const [loading, setLoading] = useState<boolean>(false);
    const [page, setPage] = useState<number>(props.page || 1);
    const [pageSize, setPageSize] = useState<number>(props.pageSize || 30);

    const columns: ColumnsType<TaskHistory> = [
        {
            title: 'Task Name',
            key: 'task_name',
            align: 'center',
            dataIndex: 'task_name',
            fixed: 'left',
            width: 150,
        },
        {
            title: 'Description',
            dataIndex: 'key_sentence',
            fixed: 'left',
            ellipsis: true,
            width: 'auto',
        },
        {
            title: 'Workflow ID',
            dataIndex: 'workflow_id',
            align: 'center',
            key: 'workflow_id',
            ellipsis: true,
            width: 150,
        },
        {
            title: 'Labels',
            dataIndex: 'labels',
            key: 'labels',
            align: 'center',
            width: 150,
        },
        {
            title: 'Submitted Time',
            key: 'submitted_time',
            align: 'center',
            dataIndex: 'submitted_time',
            render: (text) => {
                return new Date(text).toLocaleString();
            },
            width: 200,
        },
        {
            title: 'Started Time',
            key: 'started_time',
            align: 'center',
            dataIndex: 'started_time',
            render: (text) => {
                return new Date(text).toLocaleString();
            },
            width: 200,
        },
        {
            title: 'Finished Time',
            key: 'finished_time',
            align: 'center',
            dataIndex: 'finished_time',
            render: (text) => {
                return new Date(text).toLocaleString();
            },
            width: 200,
        },
        {
            title: 'Status',
            dataIndex: 'status',
            align: 'center',
            width: 120,
        },
        {
            title: 'Actions',
            key: 'actions',
            align: 'center',
            fixed: 'right',
            width: 220,
            render: (text, record) => {
                return <Button.Group>
                    <Button type="primary" onClick={() => { props.onTaskHistoryClick?.(record) }}>View</Button>
                    <Button disabled={!isFinished(record)}>Log</Button>
                    <Button danger disabled>Delete</Button>
                </Button.Group>
            },
        },
    ];

    useEffect(() => {
        setLoading(true);
        fetchTasks({
            page: page,
            page_size: pageSize
        })
            .then((response) => {
                setData({
                    data: response.records,
                    total: response.total,
                    page: response.page,
                    pageSize: response.page_size,
                });
                setLoading(false);
            })
            .catch((error) => {
                console.log('Get tasks error: ', error);
                setData({} as TaskHistoryTableData);
                setLoading(false);
            });
    }, [page, pageSize, props.refreshKey]);

    const getRowKey = (record: TaskHistory) => {
        return record.id || `${JSON.stringify(record)}`;
    };

    return (
        <Row className="task-history-table-container">
            <h3>Task History for Omics Data</h3>
            <Table
                className={props.className ? props.className + ' task-history-table' : 'task-history-table'}
                style={props.style}
                size="middle"
                bordered={true}
                columns={columns}
                loading={loading}
                scroll={{ x: props.xScroll || 1000, y: props.yScroll || 'calc(100vh - 220px)' }}
                dataSource={data.data}
                rowKey={(record) => getRowKey(record)}
                pagination={{
                    showSizeChanger: true,
                    showQuickJumper: true,
                    pageSizeOptions: props.pageSizeOptions || ['10', '20', '50', '100'],
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
            />
        </Row>
    );
});

export default TaskHistoryTable;
