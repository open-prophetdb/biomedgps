import React, { useEffect, useState } from 'react';
import { Row, Col, Spin, Tag, Select, Empty, Popover, message } from 'antd';
import { history } from 'umi';
import { BookOutlined, ToolOutlined, createFromIconfontCN } from '@ant-design/icons';
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
            stat: '30,662',
        },
        {
            key: 'gene',
            icon: 'biomedgps-gene',
            title: 'Gene',
            stat: '105,382',
            description: '',
        },
        {
            key: 'compound',
            icon: 'biomedgps-drug',
            title: 'Compound',
            stat: '267,789',
            description: '',
        },
        {
            key: 'knowledges',
            icon: 'biomedgps-knowledge',
            title: 'Knowledges',
            stat: '5,810,160',
            description: '',
        },
    ];

    const onSearch = (value: string, name?: string) => {
        console.log('Search:', value);

        if (value && name) {
            history.push(`/knowledge-table?nodeId=${value}&nodeName=${name}`);
            return;
        }

        const filtered = filter(nodeOptions, (item) => item.value === value);
        if (filtered.length === 0 || !filtered[0]?.metadata) {
            history.push(`/knowledge-table?nodeId=${value}`);
        } else {
            const metadata = filtered[0].metadata;
            history.push(`/knowledge-table?nodeId=${value}&nodeName=${metadata.name}`);
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

    return (
        <Row className="welcome">
            <Row className="box">
                <Col className="header">
                    <h4 style={{ textAlign: 'center', fontSize: '16px', lineHeight: '24px' }}>
                        Enter a gene/protein, disease, drug or symptom name to find and explain related known knowledges in our platform.
                        <br />
                        If you want to predict new knowledges, please go to the <a onClick={() => { history.push('/predict-model'); }}>Predict Drug/Target</a> page.
                        <br />
                        Please click the following examples to see the results.
                    </h4>
                    <Select
                        showSearch
                        allowClear
                        size="large"
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
                        Examples: {' '}
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
                    <span style={{ textAlign: 'center', color: 'red', fontWeight: 'bold' }}>
                        NOTE: If you cannot find the node you are looking for, this may be due to the lack of knowledges in the current version of the platform.
                        <br />
                        Please give us feedback or check the <a href={`https://${window.location.host}/#/about`}>About</a> page for more information.
                    </span>
                </Col>
                <Row className="statistics" gutter={16}>
                    <Col sm={0} md={1} xs={1} xxl={1}></Col>
                    <Col className="data-stat" sm={24} md={11} xs={11} xxl={11}>
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
                    <Col sm={0} md={1} xs={1} xxl={1}></Col>
                    <Col className="image-container" sm={24} md={10} xs={10} xxl={10}>
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
                    <Col sm={0} md={1} xs={1} xxl={1}></Col>
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
