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
                    {
                        compoundInfo.general_references?.articles ? (compoundInfo.general_references?.articles.map((article, index) => {
                            return (
                                <div key={index}>
                                    <span>{article.citation}</span>
                                    <a href={formatPubmedId(article.pubmed_id)} target="_blank">PubMed</a>
                                </div>
                            )
                        })) : null
                    }
                </Col>
                <Col className="link">
                    {
                        compoundInfo.general_references?.links ? (compoundInfo.general_references?.links.map((link, index) => {
                            return (
                                <div key={index}>
                                    <a href={link.url} target="_blank">{link.title}</a>
                                </div>
                            )
                        })) : null
                    }
                </Col>
            </Row>
            : null
    );
}

export default Reference;
