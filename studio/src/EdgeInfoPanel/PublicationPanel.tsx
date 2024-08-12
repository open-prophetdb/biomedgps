import React, { useEffect, useState } from 'react';
import { MarkdownViewer } from 'biominer-components';
import RehypeRaw from 'rehype-raw';
import { Button, List, message, Row, Col, Tag } from 'antd';
import { FileProtectOutlined } from '@ant-design/icons';
import type { Publication, PublicationDetail } from 'biominer-components/dist/typings';
import PublicationDesc from './PublicationDesc';
import { fetchPublication, fetchPublications, fetchPublicationsSummary, answerQuestionWithPublications } from '@/services/swagger/KnowledgeGraph';

import './index.less';

export type PublicationPanelProps = {
    queryStr: string;
    startNode?: string;
    endNode?: string;
};

const PublicationPanel: React.FC<PublicationPanelProps> = (props) => {
    const [page, setPage] = useState<number>(0);
    const [total, setTotal] = useState<number>(0);
    const [pageSize, setPageSize] = useState<number>(10);
    const [loading, setLoading] = useState<boolean>(false);
    const [publicationMap, setPublicationMap] = useState<Record<string, PublicationDetail>>({});
    const [abstractMap, setAbstractMap] = useState<Record<string, string>>({});
    const [searchId, setSearchId] = useState<string>('');
    const [publicationSummary, setPublicationSummary] = useState<string>('Loading...');
    const [generating, setGenerating] = useState<boolean>(false);

    const showAbstract = async (doc_id: string): Promise<PublicationDetail> => {
        console.log('Show Abstract: ', doc_id);
        return new Promise((resolve, reject) => {
            fetchPublication({ id: doc_id }).then((publication) => {
                console.log('Publication: ', publication);
                setPublicationMap({
                    ...publicationMap,
                    [doc_id]: publication
                })
                resolve(publication);
            }).catch((error) => {
                console.error('Error: ', error);
                reject(error);
            });
        });
    };

    useEffect(() => {
        if (!props.queryStr) {
            return;
        }

        setLoading(true);
        fetchPublications(
            {
                query_str: props.queryStr,
                page: 0,
                page_size: 10
            }).then((data) => {
                setSearchId(data.search_id || '');
                if (data.search_id) {
                    fetchPublicationSummary(data.search_id);
                }

                let publicationMap: Record<string, PublicationDetail> = {};
                data.records.forEach((publication) => {
                    publicationMap[publication.doc_id] = publication;
                });
                setPublicationMap(publicationMap);
                setPage(data.page);
                setTotal(data.total);
                setPageSize(data.page_size);
            }).catch((error) => {
                console.error('Error: ', error);
                message.error('Failed to fetch publications');
            }).finally(() => {
                setLoading(false);
            });
    }, [props.queryStr, page, pageSize]);

    const loadAbstractsAndAnswer = async (docIds: string[]) => {
        const tempAbstractMap: Record<string, string> = {};
        for (let i = 0; i < docIds.length; i++) {
            const docId = docIds[i];
            if (!publicationMap[docId].article_abstract) {
                const msg = `Load ${i} publication...`;
                setPublicationSummary(msg);
                await showAbstract(docId).then((publication) => {
                    tempAbstractMap[docId] = publication.article_abstract || '';
                }).catch((error) => {
                    setGenerating(false);
                    console.error('Error: ', error);
                });
                setTimeout(() => console.log(msg), 200 * i)
            }
        }

        setAbstractMap(tempAbstractMap);
        setPublicationSummary('Publications loaded, answering question...');

        answerQuestionWithPublications(
            {
                question: props.queryStr,
            },
            // @ts-ignore Don't need to care about this warning, just because the authors field is not defined as a string[].
            Object.values(publicationMap)
        ).then((response) => {
            console.log('Answer: ', response);
            setPublicationSummary(response.summary);
        }).catch((error) => {
            setGenerating(false);
            console.error('Error: ', error);
            setPublicationSummary('Failed to answer question, because of the following error: ' + error);
        });
    }

    const showPublication = async (publication: PublicationDetail) => {
        console.log('Show Publication: ', publication);
        if (publication) {
            console.log('Publication Map: ', publicationMap);
            const link = publication?.provider_url;
            const doi_link = "https://doi.org/" + publication?.doi;

            if (publication?.doi) {
                window.open(doi_link, '_blank');
            } else if (link) {
                window.open(link, '_blank');
            } else {
                message.warning('No link available for this publication');
            }
        }
    };

    const fetchPublicationSummary = async (searchId: string) => {
        const response = await fetchPublicationsSummary({
            search_id: searchId
        })

        if (response && response.summary) {
            const summary = response.summary;
            setPublicationSummary(summary);
        }
    }

    const onClickPublication = (item: Publication) => {
        if (publicationMap[item.doc_id]) {
            showPublication(publicationMap[item.doc_id])
        } else {
            showAbstract(item.doc_id).then((publication) => {
                showPublication(publication);
            }).catch((error) => {
                message.error('Failed to fetch publication details');
            });
        }
    }

    return (
        <Row className='publication-panel'>
            <Tag className='publication-tag'>Summary</Tag>
            <Col className='publication-panel-header'>
                <span>
                    <Tag>Question</Tag>
                    {props.queryStr} <Button type="primary" onClick={() => {
                        setGenerating(true);
                        const docIds = Object.keys(publicationMap);
                        if (docIds.length > 0) {
                            loadAbstractsAndAnswer(docIds);
                        }
                    }} disabled={generating || Object.keys(publicationMap).length == 0} size='small'>
                        Generate Detailed Answer
                    </Button>
                </span>
                <p>
                    {/* <Tag>Answer by AI</Tag> */}
                    <MarkdownViewer markdown={publicationSummary} rehypePlugins={[RehypeRaw]} />
                </p>
            </Col>

            <Tag className='publication-tag'>References</Tag>
            <Col className='publication-panel-content'>
                <List
                    loading={loading}
                    itemLayout="horizontal"
                    rowKey={'doc_id'}
                    dataSource={Object.values(publicationMap)}
                    size="large"
                    pagination={{
                        disabled: false,
                        position: 'top',
                        current: page,
                        total: total,
                        pageSize: pageSize,
                        onChange: (page: number, pageSize: number) => {
                            setPage(page);
                            setPageSize(pageSize);
                        }
                    }}
                    renderItem={(item, index) => (
                        <List.Item>
                            <List.Item.Meta
                                avatar={<FileProtectOutlined />}
                                title={<a onClick={(e) => { onClickPublication(item); }}>{item.title}</a>}
                                description={
                                    <PublicationDesc publication={item} abstract={abstractMap[item.doc_id]}
                                        showAbstract={showAbstract} startNode={props.startNode} endNode={props.endNode}
                                        showPublication={(publication) => onClickPublication(publication)}
                                    />
                                }
                            />
                        </List.Item>
                    )}
                />
            </Col>
        </Row>
    );
};

export default PublicationPanel;
