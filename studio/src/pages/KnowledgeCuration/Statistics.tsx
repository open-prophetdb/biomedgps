import React, { useEffect, useState } from 'react';
import { Card, Row, Col, Statistic, Table, Progress, Empty, Spin, message } from 'antd';
import { 
    UserOutlined, 
    FileTextOutlined, 
    LinkOutlined, 
    CalendarOutlined,
    BarChartOutlined,
    PieChartOutlined
} from '@ant-design/icons';
import { fetchCurationStatistics } from '@/services/swagger/KnowledgeGraph';
// @ts-ignore
import Plot from 'react-plotly.js';

import './Statistics.less';

const Statistics: React.FC = () => {
    const [loading, setLoading] = useState(true);
    const [stats, setStats] = useState<swagger.CurationStatistics>({
        total_knowledges: 0,
        total_entities: 0,
        total_key_sentences: 0,
        total_curators: 0,
        recent_activity_30_days: 0,
        recent_activity_60_days: 0,
        recent_activity_90_days: 0,
        recent_activity_180_days: 0,
        top_curators: [],
        top_relation_types: [],
        top_entity_types: [],
        monthly_trend: [],
        curator_activity: [],
        monthly_curator_trend: []
    });

    useEffect(() => {
        fetchStatistics();
    }, []);

    const generateLastMonths = (n: number): string[] => {
        const results: string[] = [];
        const base = new Date();
        // Use first day of month to avoid DST shifts
        base.setDate(1);
        for (let i = n - 1; i >= 0; i--) {
            const d = new Date(base.getFullYear(), base.getMonth() - i, 1);
            const key = `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}`;
            results.push(key);
        }
        return results;
    };

    const fetchStatistics = async () => {
        try {
            setLoading(true);
            
            // 获取所有数据
            const statsRes = await fetchCurationStatistics({});

            setStats({
                total_knowledges: statsRes.total_knowledges,
                total_entities: statsRes.total_entities,
                total_key_sentences: statsRes.total_key_sentences,
                total_curators: statsRes.total_curators,
                recent_activity_30_days: statsRes.recent_activity_30_days,
                recent_activity_60_days: statsRes.recent_activity_60_days,
                recent_activity_90_days: statsRes.recent_activity_90_days,
                recent_activity_180_days: statsRes.recent_activity_180_days,
                top_curators: statsRes.top_curators,
                top_relation_types: statsRes.top_relation_types,
                top_entity_types: statsRes.top_entity_types,
                monthly_trend: statsRes.monthly_trend,
                curator_activity: statsRes.curator_activity,
                monthly_curator_trend: statsRes.monthly_curator_trend
            });

        } catch (error) {
            console.error('Failed to fetch statistics:', error);
            message.error('Failed to load statistics');
        } finally {
            setLoading(false);
        }
    };

    const curatorActivityColumns = [
        {
            title: 'Curator',
            dataIndex: 'curator',
            key: 'curator',
            render: (text: string) => <span style={{ fontWeight: 'bold' }}>{text}</span>
        },
        {
            title: 'Knowledges',
            dataIndex: 'knowledges',
            key: 'knowledges',
            render: (value: number) => <span style={{ color: '#1890ff' }}>{value}</span>
        },
        {
            title: 'Entities',
            dataIndex: 'entities',
            key: 'entities',
            render: (value: number) => <span style={{ color: '#52c41a' }}>{value}</span>
        },
        {
            title: 'Key Sentences',
            dataIndex: 'sentences',
            key: 'sentences',
            render: (value: number) => <span style={{ color: '#fa8c16' }}>{value}</span>
        },
        {
            title: 'Total',
            key: 'total',
            render: (record: any) => (
                <span style={{ fontWeight: 'bold', color: '#722ed1' }}>
                    {record.knowledges + record.entities + record.sentences}
                </span>
            )
        }
    ];

    const topCuratorsColumns = [
        {
            title: 'Rank',
            key: 'rank',
            render: (_: any, __: any, index: number) => (
                <span style={{ 
                    fontWeight: 'bold', 
                    color: index < 3 ? '#f5222d' : '#666' 
                }}>
                    #{index + 1}
                </span>
            )
        },
        {
            title: 'Curator',
            dataIndex: 'curator',
            key: 'curator'
        },
        {
            title: 'Count',
            dataIndex: 'count',
            key: 'count',
            render: (value: number) => (
                <span style={{ fontWeight: 'bold', color: '#1890ff' }}>
                    {value}
                </span>
            )
        },
        {
            title: 'Percentage',
            key: 'percentage',
            render: (record: any) => {
                const percentage = ((record.count / stats.total_knowledges) * 100).toFixed(1);
                return (
                    <Progress 
                        percent={parseFloat(percentage)} 
                        size="small" 
                        showInfo={false}
                        strokeColor="#1890ff"
                    />
                );
            }
        }
    ];

    if (loading) {
        return (
            <div style={{ textAlign: 'center', padding: '50px' }}>
                <Spin size="large" />
                <div style={{ marginTop: '20px' }}>Loading statistics...</div>
            </div>
        );
    }

    // 构造 Plotly 数据（总数柱，hover 展示按人分解）
    const months = stats.monthly_curator_trend.map(m => m.month);
    const totals = stats.monthly_curator_trend.map(m => m.total);
    const hoverTexts = stats.monthly_curator_trend.map(item => {
        const entries = Object.entries(item.per_curator).sort((a, b) => (b[1] as number) - (a[1] as number));
        const top = entries.slice(0, 10);
        const others = entries.slice(10).reduce((sum, [, v]) => sum + (v as number), 0);
        const lines = top.map(([c, v]) => `${c}: ${v}`);
        if (others > 0) {
            lines.push(`Others: ${others}`);
        }
        return lines.join('<br>');
    });

    return (
        <div className="knowledge-curation-statistics">
            <Row gutter={[16, 16]}>
                {/* 基础统计卡片 */}
                <Col xs={24} sm={12} lg={6}>
                    <Card>
                        <Statistic
                            title="Total Knowledges"
                            value={stats.total_knowledges}
                            prefix={<FileTextOutlined />}
                            valueStyle={{ color: '#1890ff' }}
                        />
                    </Card>
                </Col>
                <Col xs={24} sm={12} lg={6}>
                    <Card>
                        <Statistic
                            title="Total Entities"
                            value={stats.total_entities}
                            prefix={<LinkOutlined />}
                            valueStyle={{ color: '#52c41a' }}
                        />
                    </Card>
                </Col>
                <Col xs={24} sm={12} lg={6}>
                    <Card>
                        <Statistic
                            title="Total Key Sentences"
                            value={stats.total_key_sentences}
                            prefix={<BarChartOutlined />}
                            valueStyle={{ color: '#fa8c16' }}
                        />
                    </Card>
                </Col>
                <Col xs={24} sm={12} lg={6}>
                    <Card>
                        <Statistic
                            title="Active Curators"
                            value={stats.total_curators}
                            prefix={<UserOutlined />}
                            valueStyle={{ color: '#722ed1' }}
                        />
                    </Card>
                </Col>

                {/* 最近活动 */}
                <Col xs={24} sm={12} lg={6}>
                    <Card>
                        <Statistic
                            title="Recent Activity (30 days)"
                            value={stats.recent_activity_30_days}
                            prefix={<CalendarOutlined />}
                            valueStyle={{ color: '#f5222d' }}
                        />
                    </Card>
                </Col>
                <Col xs={24} sm={12} lg={6}>
                    <Card>
                        <Statistic
                            title="Recent Activity (60 days)"
                            value={stats.recent_activity_60_days}
                            prefix={<CalendarOutlined />}
                            valueStyle={{ color: '#f5222d' }}
                        />
                    </Card>
                </Col>

                <Col xs={24} sm={12} lg={6}>
                    <Card>
                        <Statistic
                            title="Recent Activity (90 days)"
                            value={stats.recent_activity_90_days}
                            prefix={<CalendarOutlined />}
                            valueStyle={{ color: '#f5222d' }}
                        />
                    </Card>
                </Col>

                <Col xs={24} sm={12} lg={6}>
                    <Card>
                        <Statistic
                            title="Recent Activity (180 days)"
                            value={stats.recent_activity_180_days}
                            prefix={<CalendarOutlined />}
                            valueStyle={{ color: '#f5222d' }}
                        />
                    </Card>
                </Col>

                {/* Top Curators */}
                <Col xs={24} lg={12}>
                    <Card title="Top Curators" extra={<PieChartOutlined />}>
                        <Table
                            dataSource={stats.top_curators}
                            columns={topCuratorsColumns}
                            pagination={false}
                            size="small"
                            rowKey="curator"
                            scroll={{ y: 300 }}
                        />
                    </Card>
                </Col>

                {/* Top Relation Types */}
                <Col xs={24} lg={12}>
                    <Card title="Top Relation Types">
                        <Table
                            dataSource={stats.top_relation_types}
                            columns={[
                                {
                                    title: 'Relation Type',
                                    dataIndex: 'relation_type',
                                    key: 'relation_type'
                                },
                                {
                                    title: 'Count',
                                    dataIndex: 'count',
                                    key: 'count',
                                    render: (value: number) => (
                                        <span style={{ fontWeight: 'bold', color: '#1890ff' }}>
                                            {value}
                                        </span>
                                    )
                                }
                            ]}
                            pagination={false}
                            size="small"
                            rowKey="relation_type"
                            scroll={{ y: 300 }}
                        />
                    </Card>
                </Col>

                {/* Top Entity Types */}
                <Col xs={24} lg={12}>
                    <Card title="Top Entity Types">
                        <Table
                            dataSource={stats.top_entity_types}
                            columns={[
                                {
                                    title: 'Entity Type',
                                    dataIndex: 'entity_type',
                                    key: 'entity_type'
                                },
                                {
                                    title: 'Count',
                                    dataIndex: 'count',
                                    key: 'count',
                                    render: (value: number) => (
                                        <span style={{ fontWeight: 'bold', color: '#52c41a' }}>
                                            {value}
                                        </span>
                                    )
                                }
                            ]}
                            pagination={false}
                            size="small"
                            rowKey="entity_type"
                            scroll={{ y: 300 }}
                        />
                    </Card>
                </Col>

                {/* Monthly Trend */}
                <Col xs={24} lg={12}>
                    <Card title="Monthly Activity Trend">
                        <Plot
                            data={[
                                {
                                    type: 'bar',
                                    x: months,
                                    y: totals,
                                    customdata: hoverTexts,
                                    hovertemplate: '%{x}<br>Total: %{y}<br>%{customdata}<extra></extra>',
                                    textposition: 'none',
                                    marker: { color: '#1890ff' }
                                } as any
                            ]}
                            layout={{
                                autosize: true,
                                height: 340,
                                margin: { l: 40, r: 10, t: 10, b: 60 },
                                xaxis: { tickangle: -45, tickmode: 'array', tickvals: months, ticktext: months },
                                yaxis: { title: 'Count' },
                                hovermode: 'x unified',
                                transition: { duration: 300 }
                            }}
                            config={{ displayModeBar: false, responsive: true }}
                            style={{ width: '100%' }}
                        />
                    </Card>
                </Col>

                {/* Curator Activity Details */}
                <Col xs={24}>
                    <Card title="Curator Activity Details">
                        <Table
                            dataSource={stats.curator_activity}
                            columns={curatorActivityColumns}
                            pagination={{ pageSize: 10 }}
                            size="small"
                            rowKey="curator"
                            scroll={{ y: 300 }}
                        />
                    </Card>
                </Col>
            </Row>
        </div>
    );
};

export default Statistics;
