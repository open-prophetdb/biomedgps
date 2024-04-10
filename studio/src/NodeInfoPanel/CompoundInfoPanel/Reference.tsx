import { Empty, Row, Col, Tag, Descriptions, Table, Spin } from "antd";
import React, { useEffect, useState } from "react";
import { CompoundInfo } from "./index.t";

import './index.less';

export interface InfoPanelProps {
    compoundInfo: CompoundInfo
}

const formatPubmedId = (pubmedId: string): string => {
    return `https://pubmed.ncbi.nlm.nih.gov/${pubmedId}`;
}

export const Reference: React.FC<InfoPanelProps> = (props) => {
    const { compoundInfo } = props;

    return (
        compoundInfo ?
            <Row className="compound-info-panel">
                <Col className="article">
                    <h2 className="title">Articles</h2>
                    <ol>
                        {
                            compoundInfo.general_references?.articles ? (compoundInfo.general_references?.articles.map((article, index) => {
                                return (
                                    <li key={index}>
                                        <span>{article.citation}</span>
                                        <a href={formatPubmedId(article.pubmed_id)} target="_blank">PubMed</a>
                                    </li>
                                )
                            })) : null
                        }
                    </ol>
                </Col>
                <Col className="link">
                    <h2 className="title">Links</h2>
                    <ol>
                        {
                            compoundInfo.general_references?.links ? (compoundInfo.general_references?.links.map((link, index) => {
                                return (
                                    <li key={index}>
                                        <a href={link.url} target="_blank">{link.title}</a>
                                    </li>
                                )
                            })) : null
                        }
                    </ol>
                </Col>
            </Row>
            : null
    );
}

export default Reference;
