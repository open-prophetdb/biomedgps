import React, { useState, useEffect } from 'react';
import { Card, Row, Col, Pagination, Empty, Input } from 'antd';
import { Workflow, WorkflowTableData } from '../../StatEngine/components/WorkflowList/data';
import { fetchWorkflows } from '@/services/swagger/KnowledgeGraph';
import { AppstoreOutlined } from '@ant-design/icons';

type WorkflowProps = {
    page?: number;
    pageSize?: number;
    refreshKey?: number;
    className?: string;
    onWorkflowClick?: (workflow: Workflow) => void;
};

type IconType = {
    src: string;
    type: string;
    sizes: string;
}

const WorkflowPanel: React.FC<WorkflowProps> = (props) => {
    const [workflowTableData, setWorkflowTableData] = useState<WorkflowTableData>({} as WorkflowTableData);
    const [page, setPage] = useState<number>(props.page || 1);
    const [pageSize, setPageSize] = useState<number>(props.pageSize || 30);
    const [loading, setLoading] = useState<boolean>(false);
    const [searchStr, setSearchStr] = useState<string>('');

    useEffect(() => {
        let params: any = {
            page: page,
            page_size: pageSize,
        };

        if (searchStr && searchStr.trim() !== '') {
            params["query_str"] = JSON.stringify({
                field: 'name',
                value: `%${searchStr}%`,
                operator: 'ilike'
            })
        }

        setLoading(true);
        fetchWorkflows(params).then((data) => {
            setLoading(false);
            setWorkflowTableData({
                data: data.records,
                total: data.total,
                page: data.page,
                pageSize: data.page_size,
            });
        }).catch((err) => {
            console.log('error: ', err);
            setWorkflowTableData({} as WorkflowTableData);
            setLoading(false);
        });
    }, [page, pageSize, props.refreshKey, searchStr]);

    const WorkflowCard = (props: { title: string, description: string, icon: IconType | null, onClick: () => void }) => {
        const { title, description, icon, onClick } = props;

        return (
            <Card bordered={false} style={{ textAlign: 'left' }} className='feature-card' onClick={onClick}>
                <Row style={{ display: 'flex', flexDirection: 'row', alignItems: 'flex-start', flexWrap: 'nowrap' }}>
                    <Col style={{ marginRight: '20px' }}>
                        <div style={{
                            backgroundColor: '#59aaff',
                            marginTop: '5px',
                            width: '50px',
                            height: '50px',
                            display: 'flex',
                            justifyContent: 'center',
                            alignItems: 'center',
                            borderRadius: '5px',
                        }}>
                            {icon ? <img src={icon.src} alt={icon.type || ''} width={'36px'} height={'36px'} /> : <AppstoreOutlined style={{ color: '#fff' }} />}
                        </div>
                    </Col>
                    <Col>
                        <span style={{ fontSize: '1rem', fontWeight: '500' }}>{title}</span>
                        <p style={{ fontSize: '0.9rem', marginBottom: '0px' }}>{description}</p>
                    </Col>
                </Row>
            </Card>
        );
    };

    return (
        <Row className={`workflow-cards ${props.className || ''}`}>
            <Col className='workflow-cards-title'>
                <Input.Search placeholder="Search workflows" enterButton="Search" size="large" loading={loading}
                    allowClear
                    onSearch={(value: string) => {
                        setSearchStr(value);
                        setPage(page);
                        setPageSize(pageSize);
                    }} />
            </Col>
            <Col className='workflow-cards-content'>
                {
                    workflowTableData.total > 0 ? workflowTableData.data.map((workflow: Workflow) => {
                        return (
                            <WorkflowCard title={workflow.name} description={workflow.description || ''} icon={workflow.icons || null} key={workflow.name} onClick={() => {
                                props.onWorkflowClick && props.onWorkflowClick(workflow);
                             }} />
                        );
                    }) : <Empty description='No workflows found' />
                }
            </Col>
            <Col className='workflow-cards-pagination'>
                <Pagination
                    total={workflowTableData.total}
                    pageSize={pageSize}
                    current={page}
                    onChange={(page: number, pageSize: number) => {
                        setPage(page);
                        setPageSize(pageSize);
                    }}
                    showSizeChanger={true}
                    showQuickJumper={true}
                    showTotal={(total: number) => `Total ${total} items`}
                />
            </Col>
        </Row>
    );
};

export default WorkflowPanel;