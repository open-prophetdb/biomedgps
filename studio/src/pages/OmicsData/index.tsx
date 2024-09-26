import React, { useState } from 'react';
import { Row, Col, Menu, Button, Empty } from 'antd';
import { MenuUnfoldOutlined, MenuFoldOutlined, TableOutlined, HistoryOutlined, AppstoreOutlined, ClockCircleOutlined } from '@ant-design/icons';
import Workflow from './Workflow';
import { Workflow as WorkflowData, TaskHistory as TaskHistoryData } from './typings.t';
import TaskHistory from './TaskHistory';

import './index.less';

export type OmicsDataProps = {

};

const OmicsData: React.FC<OmicsDataProps> = (props) => {
    const defaultMenuItems: any[] = [
        {
            label: 'Workflows',
            key: 'workflows',
            icon: <TableOutlined />,
            onClick: () => {
                setMenuKey('workflows');
            }
        },
        {
            label: 'Task History',
            key: 'task-history',
            icon: <HistoryOutlined />,
            onClick: () => {
                setMenuKey('task-history');
            }
        },
        {
            type: 'divider',
        }
    ];

    const [menuItems, setMenuItems] = useState<any[]>(defaultMenuItems);
    const [menuKey, setMenuKey] = useState<string>('workflows');
    const [collapsed, setCollapsed] = useState<boolean>(false);

    const onWorkflowClick = (workflow: WorkflowData) => {
        console.log(workflow);
        setMenuItems([
            ...defaultMenuItems,
            {
                label: `Workflow: ${workflow.name}`,
                key: workflow.id,
                icon: <AppstoreOutlined />,
                danger: true,
            }
        ])

        setMenuKey(workflow.id);
        setCollapsed(true);
    };

    const onTaskHistoryClick = (taskHistory: TaskHistoryData) => {
        console.log(taskHistory);
        setMenuItems([
            ...defaultMenuItems,
            {
                label: `Task: ${taskHistory.task_name}`,
                key: taskHistory.id,
                icon: <ClockCircleOutlined />,
                danger: true,
            }
        ])

        setMenuKey(taskHistory.id);
        setCollapsed(true);
    };

    const whichPanel = () => {
        if (menuKey === 'workflows') {
            return <Workflow className='workflow-container' onWorkflowClick={onWorkflowClick} />;
        } else if (menuKey === 'task-history') {
            return <TaskHistory className='task-history-container' onTaskHistoryClick={onTaskHistoryClick} />;
        } else {
            return <Empty className='empty-omics-data-container' />;
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
