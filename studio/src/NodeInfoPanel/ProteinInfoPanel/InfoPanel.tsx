import { Empty, Row, Col, Badge, Descriptions, Table, Spin } from "antd";
import React, { useEffect, useState } from "react";
import type { GeneInfo, UniProtEntry } from "../index.t";
import { isProteinCoding, fetchProteinInfo } from "./utils";
import type { DescriptionsProps } from 'antd';
import { MolStarViewer } from "..";
import { guessSpecies } from "@/components/util";
import { isEmpty } from "lodash";

import './index.less';

export interface ProteinInfoPanelProps {
    geneInfo: GeneInfo;
    proteinInfo: UniProtEntry;
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
    const background = proteinInfo?.comments?.filter((comment) => comment.commentType === 'FUNCTION');
    if (!background || (background && background.length === 0)) {
        return <div>
            <h2>Biology Background</h2>
            <Empty description="No biology background found" style={{ minHeight: '200px' }} />
        </div>
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

export const fetchAlphafoldData = async (uniprotId: string): Promise<{
    id: string;
    chain: string;
}> => {
    const response = await fetch(`https://alphafold.ebi.ac.uk/api/prediction/${uniprotId}`);
    const data = await response.json();

    return data.map((entry: any) => {
        return {
            id: entry.entryId,
            chain: `${entry.uniprotStart}-${entry.uniprotEnd}`
        }
    });
}

export const PdbInfo: React.FC<{ proteinInfo: UniProtEntry }> = ({ proteinInfo }) => {
    const [currentPdbId, setCurrentPdbId] = useState<string>('');
    const [pdbData, setPdbData] = useState<any[]>([]);

    useEffect(() => {
        const pdbInfo = proteinInfo.uniProtKBCrossReferences.filter(
            (ref) => ref.database === 'PDB'
        );
        const pdbData = pdbInfo.map((ref) => {
            return {
                key: ref.id,
                id: ref.id,
                category: ref.database,
                method: ref.properties.find((prop) => prop.key === 'Method')?.value,
                resolution: ref.properties.find((prop) => prop.key === 'Resolution')?.value,
                chain: ref.properties.find((prop) => prop.key === 'Chains')?.value,
            };
        });
        setPdbData(pdbData);
        setCurrentPdbId(pdbData[0]?.id);

        const alphafoldData = proteinInfo.uniProtKBCrossReferences.filter(
            (ref) => ref.database === 'AlphaFoldDB'
        );
        Promise.all(alphafoldData.map((ref) => fetchAlphafoldData(ref.id))).then((data) => {
            const alphafoldPdbData = data.flat().map((entry: any) => {
                return {
                    key: entry.id,
                    id: entry.id,
                    category: 'AlphaFoldDB',
                    method: 'Predicted',
                    resolution: 'Unknown',
                    chain: entry.chain,
                };
            });

            const oPdbData = [...pdbData, ...alphafoldPdbData];
            setPdbData(oPdbData);
            setCurrentPdbId(oPdbData[0]?.id);
        }).catch((err) => {
            console.error(err);
        });
    }, [proteinInfo]);

    return (
        pdbData.length === 0 ? <Empty description="No PDB found" /> :
            <Row className="pdb-info">
                <MolStarViewer pdbId={currentPdbId} dimensions={['80%', '600px']}
                    className="molstar-viewer" useInterface showControls showAxes
                />
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
    const { geneInfo, proteinInfo } = props;
    // @ts-ignore
    const [generalInfo, setGeneralInfo] = useState<DescriptionsProps['items']>([]);

    useEffect(() => {
        if (geneInfo && isProteinCoding(geneInfo)) {
            const uniprotId = geneInfo.uniprot ? geneInfo.uniprot['Swiss-Prot'] : null;
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
                    children: <a href={`https://www.genecards.org/cgi-bin/carddisp.pl?gene=${geneInfo.entrezgene}`} target="_blank">
                        {geneInfo.entrezgene}
                    </a>,
                },
                {
                    key: 'alias',
                    label: 'Alias',
                    children: geneInfo.alias ? (geneInfo.alias.join ? geneInfo.alias.join(', ') : geneInfo.alias) : null,
                },
                {
                    key: 'location',
                    label: 'Chromosome Location',
                    children: geneInfo.map_location ? geneInfo.map_location : 'Unknown',
                },
                {
                    key: 'uniport-id',
                    label: 'UniProt ID',
                    children: uniprotId ? <a href={`https://www.uniprot.org/uniprot/${uniprotId}`} target="_blank">
                        {uniprotId}
                    </a> : 'Unknown',
                },
                {
                    key: 'species',
                    label: 'Species',
                    children: guessSpecies(`${geneInfo.taxid}`)
                }
            ];

            setGeneralInfo(generalInfo);
        }
    }, [geneInfo]);

    return (
        <Row className="protein-info-panel">
            <Col className="general-information">
                {/* @ts-ignore */}
                <Descriptions title="General Information" bordered items={generalInfo} column={2} />
            </Col>
            <Col className="biology-background">
                {getBiologyBackground(proteinInfo)}
            </Col>
            <Col className="protein-snp">

            </Col>
            <Col className="protein-structure">
                <h2>Sequence</h2>
                {isEmpty(proteinInfo) ? <Empty description="No reviewed sequence found" /> :
                    <>
                        <p>{proteinInfo.sequence.value}</p>
                        <PdbInfo proteinInfo={proteinInfo} />
                    </>
                }
            </Col>
        </Row>
    );
}

export default ProteinInfoPanel;
