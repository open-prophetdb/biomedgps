import { Empty, Row, Col, Badge, Descriptions, Table, Spin } from "antd";
import React, { useEffect, useState } from "react";
import type { GeneInfo, UniProtEntry } from "./index.t";
import { isProteinCoding, fetchProteinInfo } from "./utils";
import type { DescriptionsProps } from 'antd';
import { MolStarViewer } from "..";

import './index.less';

export interface ProteinInfoPanelProps {
    geneInfo?: GeneInfo;
}

function PubMedLinks(text: string) {
    const parts = text.split(/(PubMed:\d+)/);

    const components = parts.map((part, index) => {
        if (/PubMed:\d+/.test(part)) {
            // @ts-ignore, Extract the PubMed ID from the part
            const pubMedId = part.match(/\d+/)[0];
            return <a key={index} href={`https://www.ncbi.nlm.nih.gov/pubmed/${pubMedId}`} target="_blank">{part}</a>;
        } else {
            return <span key={index}>{part}</span>;
        }
    });

    return <div>{components}</div>;
}

export const getBiologyBackground = (proteinInfo: UniProtEntry): React.ReactNode => {
    const background = proteinInfo.comments.filter((comment) => comment.commentType === 'FUNCTION');
    if (background.length === 0) {
        return <Empty description="No biology background found" />;
    } else {
        return (
            <div>
                <h2>Biology Background</h2>
                {background.map((comment, index) => (
                    <div key={index}>
                        {comment.texts.map((text, index) => (
                            <p key={index}>
                                {PubMedLinks(text.value)}
                            </p>
                        ))}
                    </div>
                ))}
            </div>
        );
    }
}

export const PdbInfo: React.FC<{ proteinInfo: UniProtEntry }> = ({ proteinInfo }) => {
    const [currentPdbId, setCurrentPdbId] = useState<string>('');
    const [pdbData, setPdbData] = useState<any[]>([]);

    useEffect(() => {
        const pdbInfo = proteinInfo.uniProtKBCrossReferences.filter((ref) => ref.database === 'PDB');
        const pdbData = pdbInfo.map((ref) => {
            return {
                key: ref.id,
                id: ref.id,
                category: 'PDB',
                method: ref.properties.find((prop) => prop.key === 'Method')?.value,
                resolution: ref.properties.find((prop) => prop.key === 'Resolution')?.value,
                chain: ref.properties.find((prop) => prop.key === 'Chains')?.value,
            };
        });

        setPdbData(pdbData);
        setCurrentPdbId(pdbData[0]?.id);
    }, [proteinInfo]);


    return (
        pdbData.length === 0 ? <Empty description="No PDB found" /> :
            <Row className="pdb-info">
                <MolStarViewer pdbId={currentPdbId} dimensions={['80%', '500px']}
                    className="molstar-viewer" useInterface showControls showAxes />
                <Table dataSource={pdbData} columns={[
                    {
                        title: 'Database',
                        dataIndex: 'category',
                        key: 'category',
                    },
                    {
                        title: 'ID',
                        dataIndex: 'id',
                        key: 'id',
                    },
                    {
                        title: 'Method',
                        dataIndex: 'method',
                        key: 'method',
                    },
                    {
                        title: 'Resolution',
                        dataIndex: 'resolution',
                        key: 'resolution',
                    },
                    {
                        title: 'Chain',
                        dataIndex: 'chain',
                        key: 'chain',
                    }
                ]} onRow={(row) => {
                    return {
                        onClick: (event) => {
                            setCurrentPdbId(row.id);
                        }
                    };
                }} pagination={{
                    pageSize: 5,
                }} />
            </Row>
    );
}

export const ProteinInfoPanel: React.FC<ProteinInfoPanelProps> = (props) => {
    const { geneInfo } = props;
    const [proteinInfo, setProteinInfo] = useState<UniProtEntry | null>(null);
    // @ts-ignore
    const [generalInfo, setGeneralInfo] = useState<DescriptionsProps['items']>([]);
    const [loading, setLoading] = useState<boolean>(false);

    useEffect(() => {
        if (geneInfo && isProteinCoding(geneInfo)) {
            setLoading(true);
            const uniprotId = geneInfo.uniprot ? geneInfo.uniprot['Swiss-Prot'] : null;
            if (!uniprotId) {
                setProteinInfo(null);
                return;
            }

            fetchProteinInfo(uniprotId).then((resp: UniProtEntry) => {
                setProteinInfo(resp);
                setLoading(false);
            }).catch((err: any) => {
                console.error(err);
                setProteinInfo(null);
                setLoading(false);
            });

            // @ts-ignore
            const generalInfo: DescriptionsProps['items'] = [
                {
                    key: 'official-gene-symbol',
                    label: 'Official Gene Symbol',
                    children: geneInfo.symbol,
                },
                {
                    key: 'official-full-name',
                    label: 'Official Full Name',
                    children: geneInfo.name,
                },
                {
                    key: 'ncbi-gene-id',
                    label: 'NCBI Gene ID',
                    children: geneInfo.entrezgene,
                },
                {
                    key: 'alias',
                    label: 'Alias',
                    children: geneInfo.alias ? geneInfo.alias.join(', ') : null,
                },
                {
                    key: 'location',
                    label: 'Chromosome Location',
                    children: geneInfo.map_location ? geneInfo.map_location : 'Unknown',
                }
            ];

            setGeneralInfo(generalInfo);
        }
    }, [geneInfo]);

    return (
        proteinInfo ? (
            <Row className="protein-info-panel">
                <Col className="general-information">
                    {/* @ts-ignore */}
                    <Descriptions title="General Information" bordered items={generalInfo} column={2} />
                </Col>
                <Col className="biology-background">
                    {proteinInfo ? (
                        getBiologyBackground(proteinInfo)
                    ) : (
                        <Empty description="No protein found" />
                    )}
                </Col>
                <Col className="protein-snp">

                </Col>
                <Col className="protein-structure">
                    <h2>Sequence</h2>
                    <p>{proteinInfo.sequence.value}</p>
                    <PdbInfo proteinInfo={proteinInfo} />
                </Col>
            </Row>
        ) : (
            <Spin spinning={loading}><Empty description="No gene found" /></Spin>
        )
    );
}

export default ProteinInfoPanel;
