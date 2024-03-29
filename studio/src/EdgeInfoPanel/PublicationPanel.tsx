import React, { useEffect, useState } from 'react';
import { Button, List, message } from 'antd';
import { FileProtectOutlined } from '@ant-design/icons';
import type { Publication, PublicationDetail } from 'biominer-components/dist/typings';
import PublicationDesc from './PublicationDesc';
import { fetchPublication, fetchPublications } from '@/services/swagger/KnowledgeGraph';

import './index.less';

export type PublicationPanelProps = {
    queryStr: string;
};

const PublicationPanel: React.FC<PublicationPanelProps> = (props) => {
    const [publications, setPublications] = useState<Publication[]>([]);
    const [page, setPage] = useState<number>(0);
    const [total, setTotal] = useState<number>(0);
    const [pageSize, setPageSize] = useState<number>(10);
    const [loading, setLoading] = useState<boolean>(false);
    const [publicationMap, setPublicationMap] = useState<Record<string, PublicationDetail>>({});

    const showAbstract = (doc_id: string): Promise<PublicationDetail> => {
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
            }).then((publications) => {
                setPublications(publications.records);
                setPage(publications.page);
                setTotal(publications.total);
                setPageSize(publications.page_size);
            }).catch((error) => {
                console.error('Error: ', error);
                message.error('Failed to fetch publications');
            }).finally(() => {
                setLoading(false);
            });
    }, [props.queryStr, page, pageSize]);

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
        <>
            <div className='publication-panel-header'>
                <h3>
                    Top 10 Relevant Publications [<span>Keywords: {props.queryStr.split('#').join(', ')}</span>]
                </h3>
            </div>
            <List
                loading={loading}
                itemLayout="horizontal"
                rowKey={'doc_id'}
                dataSource={publications}
                size="large"
                pagination={{
                    disabled: true,
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
                                <PublicationDesc publication={item}
                                    showAbstract={showAbstract} queryStr={props.queryStr}
                                    showPublication={(publication) => onClickPublication(publication)}
                                />
                            }
                        />
                    </List.Item>
                )}
            />
        </>
    );
};

export default PublicationPanel;
