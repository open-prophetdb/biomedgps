import React, { useEffect, useState } from 'react';
import { Row, Layout, Col, Card, Button, Input, Menu, message } from 'antd';
import { createFromIconfontCN, LinkOutlined, LineChartOutlined, CommentOutlined } from '@ant-design/icons';
// import { ReactSVG } from 'react-svg';
import { history } from 'umi';
import type { OptionType } from 'biominer-components/dist/typings';
import { MenuItemType } from 'antd/es/menu/hooks/useItems';
import { isAuthenticated } from '@/components/util';

import './index.less';

const IconFont = createFromIconfontCN({
    scriptUrl: '//at.alicdn.com/t/c/font_4435889_2sgb9f98fdw.js',
});

type StatItem = {
    icon: string;
    key: string;
    title: string | React.ReactElement;
    stat: string;
    description?: string;
};

const HomePage: React.FC = () => {
    const [nodeOptions, setNodeOptions] = useState<OptionType[] | undefined>(undefined);
    const [menuItems, setMenuItems] = useState<MenuItemType[] | undefined>([
        {
            label: 'Predict & Explain Drugs',
            key: 'predict-explain',
            icon: <LinkOutlined />,
            onClick: () => {
                console.log('Predict & Explain Drugs');
                if (isAuthenticated()) {
                    history.push('/dashboard');
                } else {
                    message.info('Please sign in / up first.');
                }
            }
        },
        {
            label: 'Understand Disease Mechanism',
            key: 'understand-disease',
            icon: <CommentOutlined />,
            onClick: () => {
                if (isAuthenticated()) {
                    history.push('/dashboard');
                } else {
                    message.info('Please sign in / up first.');
                }
            }
        },
        {
            label: 'Analyze Omics Data',
            key: 'omics-data',
            icon: <LineChartOutlined />,
            onClick: () => {
                if (isAuthenticated()) {
                    history.push('/dashboard');
                } else {
                    message.info('Please sign in / up first.');
                }
            }
        }
    ]);

    const stats: StatItem[] = [
        {
            key: 'disease',
            icon: 'biomedgps-disease',
            title: 'Disease',
            stat: '45,362',
        },
        {
            key: 'gene',
            icon: 'biomedgps-gene',
            title: 'Gene',
            stat: '95,141',
            description: '',
        },
        {
            key: 'compound',
            icon: 'biomedgps-drug',
            title: 'Compound',
            stat: '261,905',
            description: '',
        },
        {
            key: 'knowledges',
            icon: 'biomedgps-knowledge',
            title: 'Knowledges',
            stat: '12,857,601',
            description: '',
        },
    ];

    const redirectToDashboard = (path: string) => {
        console.log('redirectToDashboard', path);
        history.push(path);
    }

    return (
        <Row className="welcome" style={{ backgroundImage: `url(${require('@/assets/background.jpg')})` }}>
            <Layout className="homepage-layout">
                <Layout.Header className="header" style={{
                    position: 'sticky',
                    top: 0,
                    zIndex: 1,
                    width: '100%',
                    display: 'flex',
                    alignItems: 'center',
                }}>
                    <Row>
                        <img src={require('@/assets/logo-home.png')} alt="logo" className="logo" />
                    </Row>
                    <Row>
                        <Menu items={menuItems} mode="horizontal" />
                        {isAuthenticated ?
                            <Button type='primary' onClick={() => { redirectToDashboard('/dashboard') }}>My Workspace</Button> :
                            <Button type='primary' onClick={() => loginWithRedirect()}>
                                Sign In / Up
                            </Button>
                        }
                    </Row>
                </Layout.Header>
                <Layout.Content className="content">
                    <Row className='content-container'>
                        <Col span={10}></Col>
                        <Col span={14}>
                            <Row className="text-content">
                                <div>
                                    <h1>AI-driven One-stop Drug Repurposing Platform</h1>
                                    <p>
                                        Unlocking the potential of existing drugs to save patients with rare and complex diseases. With a comprehensive biomedical knowledge graph and graph neural network, our platform leverages cutting-edge AI technologies, including advanced LLMs, self-supervised learning, and reinforcement learning. By deeply integrating multi-omics datasets, EHR data, and survey data, we enable effective drug repurposing for rare and complex diseases.
                                    </p>
                                </div>
                            </Row>
                            <Row className='feature-content' gutter={16}>
                                <h3>Platform Features</h3>
                                <Row>
                                    <Col span={12}>
                                        <Card
                                            hoverable
                                            cover={<img alt="example" src={require("@/assets/knowledge_graph.png")} />}
                                        >
                                            <Card.Meta title="BioMedical Knowledge Graph" description="A customized comprehensive biomedical knowledge graph for rare and complex diseases" />
                                        </Card>
                                    </Col>
                                    <Col span={12}>
                                        <Card
                                            hoverable
                                            cover={<img alt="example" src={require("@/assets/multiomics_data.png")} />}
                                        >
                                            <Card.Meta title="MultiOmics Data" description="A private multi-omics data repository for rare and complex diseases" />
                                        </Card>
                                    </Col>
                                    <Col span={12}>
                                        <Card
                                            hoverable
                                            cover={<img alt="example" src={require("@/assets/gnn.png")} />}
                                        >
                                            <Card.Meta title="AI Algorithms" description="Cutting-edge AI algorithms for drug repurposing" />
                                        </Card>
                                    </Col>
                                    <Col span={12}>
                                        <Card
                                            hoverable
                                            cover={<img alt="example" src={require("@/assets/gps-healthcare.jpg")} />}
                                        >
                                            <Card.Meta title="EHR & Survey Data" description="A private EHR and survey data repository for rare and complex diseases" />
                                        </Card>
                                    </Col>
                                </Row>
                            </Row>
                            <Row className="statistics-content" gutter={16}>
                                <h3>Statistics</h3>
                                <Row>
                                    {stats.map((item) => {
                                        return (
                                            <Col
                                                span={12}
                                                className="stat-item-container"
                                                key={item.key}
                                            >
                                                <div className="stat-item">
                                                    {typeof item.icon === 'string' ? (
                                                        <IconFont className="icon" type={item.icon}></IconFont>
                                                    ) : (
                                                        item.icon
                                                    )}
                                                    <span className="stat">{item.stat}</span>
                                                    <span className="title">{item.title}</span>
                                                    {/* <p className='desc'>{item.description}</p> */}
                                                    {/* <p className="desc">
                    {item.description
                      ? item.description?.split(';').map((item) => {
                          return <Tag key={item}>{item}</Tag>;
                        })
                      : '-'}
                  </p> */}
                                                </div>
                                            </Col>
                                        );
                                    })}
                                </Row>
                            </Row>
                        </Col>
                    </Row>
                </Layout.Content>
            </Layout>
        </Row>
    );
};

export default HomePage;
