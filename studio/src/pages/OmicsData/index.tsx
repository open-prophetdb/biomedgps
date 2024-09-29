import React, { useState, useEffect } from 'react';
import { Row, Col, Menu, Button, Empty, message } from 'antd';
import { MenuUnfoldOutlined, MenuFoldOutlined, DatabaseOutlined, HistoryOutlined, AppstoreOutlined, ClockCircleOutlined } from '@ant-design/icons';
import Workflow from './Workflow';
import { Workflow as WorkflowData, TaskHistory as TaskHistoryData } from '../../StatEngine/components/WorkflowList/data';
import TaskHistory from './TaskHistory';
import StatEngine from '../../StatEngine';

import './index.less';
import { fetchWorkflows } from '@/services/swagger/KnowledgeGraph';

export type OmicsDataProps = {

};

const OmicsData: React.FC<OmicsDataProps> = (props) => {
    const defaultMenuItems: any[] = [
        {
            label: 'Datasets',
            key: 'datasets',
            icon: <DatabaseOutlined />,
            onClick: () => {
                setMenuKey('datasets');
                setMenuItems(defaultMenuItems);
            }
        },
        {
            label: 'Workflows',
            key: 'workflows',
            icon: <AppstoreOutlined />,
            onClick: () => {
                setMenuKey('workflows');
                setMenuItems(defaultMenuItems);
            }
        },
        {
            label: 'Task History',
            key: 'task-history',
            icon: <HistoryOutlined />,
            onClick: () => {
                setMenuKey('task-history');
                setMenuItems(defaultMenuItems);
            }
        },
        {
            type: 'divider',
        }
    ];

    const [menuItems, setMenuItems] = useState<any[]>(defaultMenuItems);
    const [menuKey, setMenuKey] = useState<string>('datasets');
    const [workflows, setWorkflows] = useState<WorkflowData[]>([]);
    const [collapsed, setCollapsed] = useState<boolean>(false);

    useEffect(() => {
        fetchWorkflows({ page: 1, page_size: 100 }).then((workflows) => {
            setWorkflows(workflows.records);
        }).catch((err) => {
            console.log('error: ', err);
        });
    }, []);

    const onWorkflowClick = (workflow: WorkflowData) => {
        console.log(workflow);
        setMenuItems([
            ...defaultMenuItems,
            {
                label: `Workflow: ${workflow.name}`,
                key: workflow.id,
                icon: <AppstoreOutlined />,
                danger: true,
                data: workflow,
            }
        ])

        setMenuKey(workflow.id);
        setCollapsed(true);
    };

    const onTaskHistoryClick = async (taskHistory: TaskHistoryData) => {
        console.log('onTaskHistoryClick: ', taskHistory);

        if (!workflows.length || !taskHistory.workflow_id || workflows.find((workflow) => workflow.id === taskHistory.workflow_id) === undefined) {
            message.error('No workflows found');
            return;
        }

        setMenuItems([
            ...defaultMenuItems,
            {
                label: `Task: ${taskHistory.task_name}`,
                key: taskHistory.task_id,
                icon: <ClockCircleOutlined />,
                danger: true,
                data: taskHistory,
            }
        ])

        setMenuKey(taskHistory.task_id);
        setCollapsed(true);
    };

    const whichPanel = () => {
        if (menuKey === 'datasets') {
            return <Empty className='empty-omics-data-container' description='Datasets panel is coming soon' />;
        } else if (menuKey === 'workflows') {
            return <Workflow className='workflow-container' onWorkflowClick={onWorkflowClick} />;
        } else if (menuKey === 'task-history') {
            return <TaskHistory className='task-history-container' onTaskHistoryClick={onTaskHistoryClick} />;
        } else {
            const data = menuItems.find((item) => item.key === menuKey)?.data;
            console.log("whichPanel in OmicsData: ", data, menuKey);

            if (data.task_id) {
                const workflow = workflows.find((workflow) => workflow.id === data.workflow_id);
                if (!workflow) {
                    message.error('No workflow found');
                    return <Empty className='empty-omics-data-container' description='No workflow with this task' />;
                }

                // It's a task
                return <StatEngine workflow={workflow} task={data} workflowId={data.workflow_id} />;
            } else {
                // It's a workflow
                return <StatEngine workflow={data} workflowId={data.id} />;
            }
            // return <Empty className='empty-omics-data-container' />;
        }
    };

    return <Row className='omics-data-wrapper'>
        <Col className='menu-panel'>
            <Menu
                openKeys={[menuKey]}
                selectedKeys={[menuKey]}
                mode="inline"
                theme="light"
                items={menuItems}
                inlineCollapsed={collapsed}
            />
            <Button type="primary" onClick={() => { setCollapsed(!collapsed) }} style={{ marginBottom: 16 }}>
                {collapsed ? <MenuUnfoldOutlined /> : <MenuFoldOutlined />}
            </Button>
        </Col>
        <Row className={`${collapsed ? 'collapsed-' : ''}omics-data-container`}>
            {whichPanel()}
        </Row>
    </Row>
};

export default OmicsData;
