import React, { useEffect, useState } from 'react';
import { Row, Col, Spin, Tag, Select, Empty, Popover, message, Card } from 'antd';
import { history } from 'umi';
import { BookOutlined, ToolOutlined, ApiOutlined, createFromIconfontCN, TableOutlined, BarChartOutlined, CodepenOutlined, NodeIndexOutlined } from '@ant-design/icons';
// import { ReactSVG } from 'react-svg';
import { fetchEntities } from '@/services/swagger/KnowledgeGraph';
import type { OptionType, Entity, ComposeQueryItem, QueryItem } from 'biominer-components/dist/typings';
import { Carousel } from 'react-responsive-carousel';
import { filter, orderBy } from 'lodash';
import { guessColor } from '@/components/util';
import EntityCard from '@/components/EntityCard';

import 'react-responsive-carousel/lib/styles/carousel.min.css';
import './index.less';

const IconFont = createFromIconfontCN({
    scriptUrl: '//at.alicdn.com/t/c/font_4435889_2sgb9f98fdw.js',
});

export function makeQueryEntityStr(params: Partial<Entity>, order?: string[]): string {
    let query: ComposeQueryItem = {} as ComposeQueryItem;

    let label_query_item = {} as QueryItem;
    if (params.label) {
        label_query_item = {
            operator: '=',
            field: 'label',
            value: params.label,
        };
    }

    let filteredKeys = filter(Object.keys(params), (key) => key !== 'label');
    if (filteredKeys.length > 1) {
        query = {
            operator: 'or',
            items: [],
        };

        if (order) {
            // Order and filter the keys.
            filteredKeys = order.filter((key) => filteredKeys.includes(key));
        }
    } else {
        query = {
            operator: 'and',
            items: [],
        };
    }

    query['items'] = filteredKeys.map((key) => {
        return {
            operator: 'ilike',
            field: key,
            value: `%${params[key as keyof Entity]}%`,
        };
    });

    if (label_query_item.field) {
        if (query['operator'] === 'and') {
            query['items'].push(label_query_item);
        } else {
            query = {
                operator: 'and',
                items: [query, label_query_item],
            };
        }
    }

    return JSON.stringify(query);
}

let timeout: ReturnType<typeof setTimeout> | null;
export const fetchNodes = async (
    value: string,
    callback: (any: any) => void,
) => {
    // We might not get good results when the value is short than 3 characters.
    if (value.length < 3) {
        callback([]);
        return;
    }

    if (timeout) {
        clearTimeout(timeout);
        timeout = null;
    }

    // TODO: Check if the value is a valid id.

    let queryMap = {};
    let order: string[] = [];
    // If the value is a number, then maybe it is an id or xref but not for name or synonyms.
    if (value && !isNaN(Number(value))) {
        queryMap = { id: value, xrefs: value };
        order = ['id', 'xrefs', 'label'];
    } else {
        queryMap = { name: value, synonyms: value, xrefs: value, id: value };
        order = ['name', 'synonyms', 'xrefs', 'id', 'label'];
    }

    const fetchData = () => {
        fetchEntities({
            query_str: makeQueryEntityStr(queryMap, order),
            page: 1,
            page_size: 50,
            // We only want to get all valid entities.
            model_table_prefix: 'biomedgps',
        })
            .then((response) => {
                const { records } = response;
                // @ts-ignore
                const options: OptionType[] = records.map((item: Entity, index: number) => ({
                    order: index,
                    value: `${item['label']}::${item['id']}`,
                    label: <span><Tag color={guessColor(item['label'])}>{item['label']}</Tag> {`${item['id']} | ${item['name']}`}</span>,
                    description: item['description'],
                    metadata: item,
                }));
                console.log('getLabels results: ', options);
                callback(orderBy(options, ['value']));
            })
            .catch((error) => {
                if (error.response.status === 401) {
                    message.warning("Please login to see the search results.")
                } else {
                    message.warning("Cannot get search results for your query. Please try again later.")
                }
                console.log('requestNodes Error: ', error);
                callback([]);
            });
    };

    timeout = setTimeout(fetchData, 300);
};

type StatItem = {
    icon: string;
    key: string;
    title: string | React.ReactElement;
    stat: string;
    description?: string;
};

type ImageItem = {
    src: string;
    title: string;
};

const HomePage: React.FC = () => {
    const [loading, setLoading] = useState<boolean>(false);
    const [nodeOptions, setNodeOptions] = useState<OptionType[] | undefined>(undefined);

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

    const onSearch = (value: string, name?: string) => {
        console.log('Search:', value);

        if (value && name) {
            history.push(`/predict-explain/knowledge-table?nodeId=${value}&nodeName=${name}`);
            return;
        }

        const filtered = filter(nodeOptions, (item) => item.value === value);
        if (filtered.length === 0 || !filtered[0]?.metadata) {
            history.push(`/predict-explain/knowledge-table?nodeId=${value}`);
        } else {
            const metadata = filtered[0].metadata;
            history.push(`/predict-explain/knowledge-table?nodeId=${value}&nodeName=${metadata.name}`);
        }
    };

    const images: ImageItem[] = [
        {
            src: 'https://github.com/yjcyxky/biomedgps-studio/blob/master/public/assets/chatbot.png?raw=true',
            title: 'Demo1: Ask questions with chatbot',
        },
        {
            src: 'https://github.com/yjcyxky/biomedgps-studio/blob/master/public/assets/disease-similarities.png?raw=true',
            title:
                'Demo2: Find similar diseases with your queried disease',
        },
        {
            src: 'https://github.com/yjcyxky/biomedgps-studio/blob/master/public/assets/drug-targets-genes.png?raw=true',
            title:
                'Demo3: Predict drugs and related genes for your queried disease',
        },
        {
            src: 'https://github.com/yjcyxky/biomedgps-studio/blob/master/public/assets/path.png?raw=true',
            title:
                'Demo4: Find potential paths between two nodes',
        },
        {
            src: 'https://github.com/yjcyxky/biomedgps/blob/dev/studio/public/README/images/step2-predict-page.png?raw=true',
            title:
                'Predict Interactions',
        },
        {
            src: 'https://github.com/yjcyxky/biomedgps/blob/dev/studio/public/README/images/step3-explain.png?raw=true',
            title:
                'Explain Your Prediction',
        },
    ];

    const features = [
        {
            title: 'Predict Drugs',
            description: 'Predict new drug indications and understand disease mechanisms.',
            icon: <IconFont className="icon" type="biomedgps-drug" style={{ color: '#fff', fontSize: '30px' }}></IconFont>,
            onClick: () => {
                history.push('/predict-explain/predict-model?prediction_type=Compound&model_name=Disease');
            }
        },
        {
            title: 'Predict Targets',
            description: 'Predict new drug indications and understand disease mechanisms.',
            icon: <IconFont className="icon" type="biomedgps-drug" style={{ color: '#fff', fontSize: '30px' }}></IconFont>,
            onClick: () => {
                history.push('/predict-explain/predict-model?prediction_type=Gene&model_name=Disease');
            }
        },
        {
            title: 'Predict Indications',
            description: 'Predict new drug indications and understand disease mechanisms.',
            icon: <IconFont className="icon" type="biomedgps-disease" style={{ color: '#fff', fontSize: '30px' }}></IconFont>,
            onClick: () => {
                history.push('/predict-explain/predict-model?prediction_type=Disease&model_name=Compound');
            }
        },
        {
            title: 'Explain Your Findings',
            description: 'Explain your findings with our explainable AI.',
            icon: <ApiOutlined style={{ color: '#fff', fontSize: '30px' }} />,
            onClick: () => {
                history.push('/predict-explain/knowledge-graph');
            }
        },
        {
            title: 'Personalized Knowledge Graph',
            description: 'Manage your personalized knowledge graph.',
            icon: <NodeIndexOutlined style={{ color: '#fff', fontSize: '30px' }} />,
            onClick: () => {
                history.push('/knowledge-curation');
            }
        },
        {
            title: 'Statistics',
            description: 'View the statistics of the knowledge graph.',
            icon: <BarChartOutlined style={{ color: '#fff', fontSize: '30px' }} />,
            onClick: () => {
                history.push('/statistics');
            }
        },
        {
            title: 'ME/CFS & LongCOVID',
            description: 'Explore the ME/CFS & LongCOVID knowledge graph.',
            icon: <TableOutlined style={{ color: '#fff', fontSize: '30px' }} />,
            onClick: () => {
                history.push('/mecfs-longcovid');
            }
        },
        {
            title: 'PTSD',
            description: 'Explore the PTSD knowledge graph.',
            icon: <TableOutlined style={{ color: '#fff', fontSize: '30px' }} />,
            onClick: () => {
                history.push('/predict-explain/knowledge-table?nodeIds=Disease::MONDO:0005146');
            }
        }
    ];

    const FeatureCard = (props: { title: string, description: string, icon: React.ReactNode, onClick: () => void }) => {
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
                            {icon}
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
        <Row className="dashboard">
            <Row className="box">
                <Row className='first-row'>
                    <Col className='left-col'>
                        {/* 
                            <h3>Network Medicine Platform</h3>
                            <p>
                                Network Medicine for Disease Mechanism and Treatment Based on AI and knowledge graph.
                            </p> 
                        */}
                        <img src={require('@/assets/knowledge_graph_diagram.png')} alt="logo" />
                    </Col>
                    <Col className="right-col">
                        {/* <img src={require('@/assets/logo-white.png')} alt="logo" height="80" /> */}
                        <h4 style={{ fontSize: '1rem', lineHeight: '24px' }}>
                            Enter a gene/protein, disease, drug or symptom name to find and explain related known knowledges in our platform.
                            <br />
                            If you want to predict new knowledges, please go to the <a onClick={() => { history.push('/predict-explain/predict-model'); }}>Predict Drug/Target</a> page.
                            <br />
                            Please click the following examples to see the results.
                        </h4>
                        <Select
                            showSearch
                            allowClear
                            size="large"
                            style={{ width: '100%' }}
                            getPopupContainer={(triggerNode) => {
                                return triggerNode.parentNode;
                            }}
                            loading={loading}
                            defaultActiveFirstOption={false}
                            placeholder="Enter a gene/protein, disease, drug or symptom name to start..."
                            onSearch={(value) => {
                                setLoading(true);
                                fetchNodes(value, setNodeOptions).finally(() => {
                                    setLoading(false);
                                });
                            }}
                            filterOption={false}
                            onSelect={(value, options) => {
                                onSearch(value);
                            }}
                            notFoundContent={
                                <Empty
                                    description={
                                        loading
                                            ? 'Searching...'
                                            : nodeOptions !== undefined
                                                ? 'Not Found or Too Short Input'
                                                : 'Enter a gene/protein, disease, drug or symptom name to start...'
                                    }
                                />
                            }
                        >
                            {nodeOptions &&
                                nodeOptions.map((option: any) => (
                                    <Select.Option key={option.value} value={option.value} disabled={option.disabled}>
                                        {option.metadata ? (
                                            <Popover
                                                mouseEnterDelay={0.5}
                                                placement="rightTop"
                                                title={option.label}
                                                content={EntityCard(option.metadata)}
                                                trigger="hover"
                                                getPopupContainer={(triggeredNode: any) => document.body}
                                                overlayClassName="entity-id-popover"
                                                autoAdjustOverflow={false}
                                                destroyTooltipOnHide={true}
                                                zIndex={1500}
                                            >
                                                {option.label}
                                            </Popover>
                                        ) : (
                                            option.label
                                        )}
                                    </Select.Option>
                                ))}
                        </Select>
                        <span className="desc">
                            <h4 style={{ marginBottom: '10px' }}>Examples:</h4>
                            <a onClick={() => {
                                onSearch('Gene::ENTREZ:3569', 'IL6')
                            }}>
                                <Tag color={guessColor("Gene")}>Gene | IL6</Tag>
                            </a>
                            <a onClick={() => {
                                onSearch('Compound::DrugBank:DB00028', 'Human immunoglobulin G')
                            }}>
                                <Tag color={guessColor("Gene")}>Gene | Human immunoglobulin G</Tag>
                            </a>
                            <a onClick={() => {
                                onSearch('Pathway::WikiPathways:WP1742', 'TP53 Network')
                            }}>
                                <Tag color={guessColor("Pathway")}>Pathway | TP53 Network</Tag>
                            </a>
                            <a onClick={() => {
                                onSearch('Disease::MONDO:0005404', 'ME/CFS')
                            }}>
                                <Tag color={guessColor("Disease")}>Disease | Chronic Fatigue Syndrome</Tag>
                            </a>
                            <a onClick={() => {
                                onSearch('Disease::MONDO:0100233', 'LongCOVID')
                            }}>
                                <Tag color={guessColor("Disease")}>Disease | LongCOVID</Tag>
                            </a>
                            <a onClick={() => {
                                onSearch('Symptom::MESH:D005221', 'Fatigue')
                            }}>
                                <Tag color={guessColor("Symptom")}>Symptom | Fatigue</Tag>
                            </a>
                        </span>
                        <span className='note'>
                            NOTE: If you cannot find the node you are looking for, this may be due to the lack of knowledges in the current version of the platform.
                            <br />
                            Please give us feedback or check the <a href={`https://${window.location.host}/#/about`}>About</a> page for more information.
                        </span>
                    </Col>
                </Row>
                <Row className='second-row'>
                    <Col className='title'>
                        <h3>Quick Start</h3>
                    </Col>
                    <Col className='content'>
                        {
                            features.map((feature) => {
                                return (
                                    <FeatureCard title={feature.title} description={feature.description} icon={feature.icon} key={feature.title} onClick={feature.onClick} />
                                );
                            })
                        }
                    </Col>
                </Row>
                <Row className="statistics" gutter={16} style={{ display: 'none' }}>
                    <Row style={{ width: '80%', maxWidth: '1800px', margin: '0 auto' }}>
                        <Col className="data-stat">
                            <p className="desc" style={{ textAlign: 'justify' }}>
                                A platform with biomedical knowledge graph and graph neural network for drug repurposing and disease mechanism.
                                <br />
                                <br />
                                The Network Medicine Platform, BioMedGPS, integrates a biomedical knowledge graph, multi-omics data, and deep learning models, aiming to unravel the molecular mechanisms of diseases and facilitate drug repurposing. It features a predictive module for discovering new drug indications and understanding disease mechanisms, alongside an explanatory module offering a knowledge graph studio and graph neural network analysis. The platform supports custom data sources, models, and omics datasets, enhanced by large language models for dynamic querying. Demonstrations showcase its capabilities in drug prediction, disease similarity analysis, and graphical pathfinding.
                                <br />
                                <br />
                                Its unique integration enables precise prediction of drug efficacy and discovery of novel drug indications, offering a faster, cost-effective alternative to traditional drug development. By harnessing the power of graph neural networks and large language models, BioMedGPS provides deep insights into the complex biological networks underlying diseases, facilitating breakthroughs in personalized medicine and therapeutic strategies. This platform stands out by allowing customization across data sources, models, and omics datasets, ensuring versatility and applicability across a wide range of biomedical research areas.
                                <br />
                                <br />
                                More resources about the platform can be found in the <a href={`https://${window.location.host}/#/about`}>About</a> page.
                            </p>
                        </Col>
                        <Col className="image-container">
                            <Carousel autoPlay dynamicHeight={false} infiniteLoop showThumbs={false}>
                                {images.map((item: ImageItem) => {
                                    return (
                                        <div key={item.title}>
                                            <img src={item.src} />
                                            <p className="legend">{item.title}</p>
                                        </div>
                                    );
                                })}
                            </Carousel>
                        </Col>
                    </Row>
                </Row>
                <Row className="text-statistics">
                    {stats.map((item) => {
                        return (
                            <Col
                                xxl={6}
                                xl={6}
                                lg={6}
                                md={12}
                                sm={24}
                                xs={24}
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
        </Row>
    );
};

export default HomePage;
