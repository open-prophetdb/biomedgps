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
    scriptUrl: '//at.alicdn.com/t/c/font_4435889_bcnudpqk18l.js',
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
            title: 'Diseases',
            stat: '45,362',
        },
        {
            key: 'gene',
            icon: 'biomedgps-gene',
            title: 'Genes',
            stat: '95,141',
            description: '',
        },
        {
            key: 'symptom',
            icon: 'biomedgps-symptom',
            title: 'Symptoms',
            stat: '23,100',
            description: '',
        },
        // {
        //     key: 'compound',
        //     icon: 'biomedgps-drug',
        //     title: 'Compound',
        //     stat: '261,905',
        //     description: '',
        // },
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
            src: 'https://rapex.prophetdb.org/assets/examples/rapex_diagram.png',
            title: 'RAPEX Overview',
        },
        {
            src: 'https://rapex.prophetdb.org/assets/examples/curated_knowledges.png',
            title: 'Curated Knowledges',
        },
        // {
        //     src: 'https://github.com/yjcyxky/biomedgps-studio/blob/master/public/assets/disease-similarities.png?raw=true',
        //     title:
        //         'Demo2: Find similar diseases with your queried disease',
        // },
        // {
        //     src: 'https://github.com/yjcyxky/biomedgps-studio/blob/master/public/assets/drug-targets-genes.png?raw=true',
        //     title:
        //         'Demo3: Predict drugs and related genes for your queried disease',
        // },
        // {
        //     src: 'https://github.com/yjcyxky/biomedgps-studio/blob/master/public/assets/path.png?raw=true',
        //     title:
        //         'Demo4: Find potential paths between two nodes',
        // },
        // {
        //     src: 'https://github.com/yjcyxky/biomedgps/blob/dev/studio/public/README/images/step2-predict-page.png?raw=true',
        //     title:
        //         'Predict Interactions',
        // },
        // {
        //     src: 'https://github.com/yjcyxky/biomedgps/blob/dev/studio/public/README/images/step3-explain.png?raw=true',
        //     title:
        //         'Explain Your Prediction',
        // },
    ];

    return (
        <Row className="welcome">
            <Row className="box">
                <Col className="header">
                    <h1>RAPEX - Response to Air Pollution EXposure (RAPEX)</h1>
                    <h4 style={{ textAlign: 'center', fontSize: '1rem', lineHeight: '24px' }}>
                        Enter an air pollutant, gene/protein, disease, drug or symptom name to find and explain related known knowledges in RAPEX platform. If you want to predict new knowledges, please go to the <a onClick={() => { history.push('/predict-explain/predict-model'); }}>Predict Diseases/Targets</a> page. Please click the following examples to see the results.
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
                                            : 'Enter a air pollutant, gene/protein, disease, or symptom name to start...'
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
                            onSearch('Compound::MESH:D052638', 'Particular Matter')
                        }}>
                            <Tag color={guessColor("Compound")}>Compound | Particular Matter</Tag>
                        </a>
                        <a onClick={() => {
                            onSearch('Compound::MESH:D013458', 'SULFUR DIOXIDE')
                        }}>
                            <Tag color={guessColor("Compound")}>Compound | SULFUR DIOXIDE</Tag>
                        </a>
                        <a onClick={() => {
                            onSearch('Compound::MESH:D009589', 'Nitrogen Oxides')
                        }}>
                            <Tag color={guessColor("Compound")}>Compound | Nitrogen Oxides</Tag>
                        </a>
                        <a onClick={() => {
                            onSearch('Gene::ENTREZ:3569', 'IL6')
                        }}>
                            <Tag color={guessColor("Gene")}>Gene | IL6</Tag>
                        </a>
                        <a onClick={() => {
                            onSearch('Pathway::WikiPathways:WP358', 'MAPK signaling pathway')
                        }}>
                            <Tag color={guessColor("Pathway")}>Pathway | MAPK signaling pathway</Tag>
                        </a>
                        <a onClick={() => {
                            onSearch('Disease::MONDO:0005233', 'Non-small Cell Lung Carcinoma')
                        }}>
                            <Tag color={guessColor("Disease")}>Disease | Non-small Cell Lung Carcinoma</Tag>
                        </a>
                    </span>
                    <span style={{ textAlign: 'center', color: 'red', fontWeight: 'bold' }}>
                        NOTE: If you cannot find the node you are looking for, this may be due to the lack of knowledges in the current version of the platform.
                    </span>
                </Col>
                <Row className="statistics" gutter={16}>
                    <Col sm={0} md={1} xs={1} xxl={1}></Col>
                    <Col className="data-stat" sm={24} md={11} xs={11} xxl={11}>
                        <p className="desc" style={{ textAlign: 'justify' }}>
                            <span>
                            Air pollution emerged as the leading contributor to the global disease burden in 2021 and the second most significant risk factor for premature death worldwide. It is linked to severe health issues, including cancer and cardiovascular diseases. Traditional studies often isolate health outcomes without a broader, multidimensional approach, focusing solely on specific diseases. This deficiency underscores the importance of conducting integrated analyses of air pollution health impacts. Unfortunately, there is a notable absence of a comprehensive, integrated knowledge graph for detailed analysis, to better understand and mitigate the effects of air pollution.
                            </span>
                            <span>
                            To bridge this gap, we have developed a knowledge graph-based RAPEX database to systematically explore the toxicological mechanisms and health effects of air pollutants. RAPEX integrates a diverse array of air pollutants and links them with genes, proteins, pathway, diseases, and other multi-omics data, marking the first comprehensive resource exploring the intricate associations between air pollution, genes, and diseases. To enhance user engagement and understanding, we have created a user-friendly web portal (https://rapex.prophetdb.org). This portal allows users to effortlessly query, compare, analyze, or predict the potential relationships between air pollution and various biological and environmental entities. It has two uniq features:
                            </span>
                            <span>
                            1. Comprehensive data coverage. We implement ontology mapping and leverage large language models and semantic similarity techniques to integrate 10 published databases, enhancing the quality of the knowledge graph.  This approach aligns entity and relationship types, reducing entity ambiguity and relationship type fragmentation. These cleaned knowledges are incorporated into the system to establish a comprehensive knowledge graph. RAPEX houses an extensive array of biomedical knowledge, encompassing 12,857,601 knowledges that link air pollution with symptoms, genes, diseases, proteins, pathways, and metabolites etc. The resulting knowledge graph features a more balanced and diverse distribution of relationships between entities and includes phenotype databases, making the dataset suitable for representing pollutant-gene-phenotype/disease relationships. This profound integration offers deep and wide-ranging insights into the intricate interactions between air pollution and biological systems, aiding in the understanding of air pollution exposure, gene, and phenotype/disease relationships.
                            </span>
                            <span>
                            2. Database Uniqueness and Innovation. Meticulously curated and annotated knowledge from 865 rigorously selected publications, categorized into 13 entity types and 12 relationship types. Each of the 1,134 distinct knowledge points in this dataset has been annotated and reviewed by at least two independent experts, ensuring high quality and reliability.This dataset is a significant contribution to the field, offering rapid access to original research on air pollutants. It serves as an invaluable resource for validating the knowledge extraction capabilities of large language models and the predictive capabilities of air pollution exposure mechanism models. This comprehensive and expertly vetted dataset is poised to advance research and understanding in the intricate interactions between air pollution and biological systems.
                            </span>
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
