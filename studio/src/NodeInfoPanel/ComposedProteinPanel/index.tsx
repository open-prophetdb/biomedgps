import { Empty, Tabs } from "antd";
import React, { useEffect, useState } from "react";
import { fetchMyGeneInfo, isProteinCoding, fetchProteinInfo } from "../ProteinInfoPanel/utils";
import type { GeneInfo, UniProtEntry } from "../index.t";
import ProteinInfoPanel from "../ProteinInfoPanel";
import AlignmentViewer from "../AlignmentViewer";
import GeneInfoPanel from "../GeneInfoPanel";
import { guessSpecies, isExpectedSpecies, expectedOrder, guessSpeciesAbbr } from '@/components/util';

import "./index.less";

type ComposedProteinPanel = {
    geneInfo: GeneInfo;
}

const fetchProteinInfoByGeneInfo = async (geneInfo: GeneInfo): Promise<UniProtEntry> => {
    const uniprotId = geneInfo.uniprot ? geneInfo.uniprot['Swiss-Prot'] : null;
    if (!uniprotId) {
        return {} as UniProtEntry;
    }

    return fetchProteinInfo(uniprotId);
}

const ComposedProteinPanel: React.FC<ComposedProteinPanel> = (props) => {
    const { geneInfo } = props;
    const [items, setItems] = useState<any[]>([]);
    const [allGeneInfos, setAllGeneInfos] = useState<{
        taxid: number;
        geneInfo: GeneInfo;
        species: string;
        abbr: string;
    }[]>([]);
    const [allProteinInfos, setAllProteinInfos] = useState<Record<string, {
        proteinInfo: UniProtEntry;
        geneInfo: GeneInfo;
    }>>({});

    useEffect(() => {
        const init = async () => {
            if (!geneInfo) {
                return;
            }

            const proteinInfo = await fetchProteinInfoByGeneInfo(geneInfo);
            setItems([
                {
                    label: guessSpeciesAbbr(`${geneInfo.taxid}`),
                    key: geneInfo.taxid,
                    children: isProteinCoding(geneInfo) ?
                        < ProteinInfoPanel geneInfo={geneInfo} proteinInfo={proteinInfo} /> :
                        <GeneInfoPanel geneSymbol={geneInfo.symbol} />
                }
            ])

            if (!geneInfo.homologene) {
                return;
            } else {
                const remaingGenes = geneInfo.homologene.genes.filter(([taxid, entrezgene]) => {
                    return isExpectedSpecies(`${taxid}`)
                })

                const geneInfos = remaingGenes.map(([taxid, entrezgene]) => {
                    if (taxid === geneInfo.taxid) {
                        return geneInfo;
                    }
                    return fetchMyGeneInfo(entrezgene.toString());
                });

                Promise.all(geneInfos).then((geneInfos) => {
                    const oGeneInfos = geneInfos.map((geneInfo, index) => {
                        return {
                            taxid: geneInfo.taxid,
                            geneInfo,
                            species: guessSpecies(`${geneInfo.taxid}`),
                            abbr: guessSpeciesAbbr(`${geneInfo.taxid}`)
                        };
                    })
                    const orderedGeneInfos = oGeneInfos.sort((a, b) => {
                        return expectedOrder.indexOf(a.taxid.toString()) - expectedOrder.indexOf(b.taxid.toString());
                    });
                    setAllGeneInfos(orderedGeneInfos);
                }).catch((error) => {
                    console.error(error);
                    setAllGeneInfos([]);
                });

                const proteinInfos = remaingGenes.map(([taxid, entrezgene]) => {
                    if (taxid === geneInfo.taxid) {
                        return proteinInfo;
                    }
                    return fetchProteinInfoByGeneInfo(geneInfo);
                });

                Promise.all(proteinInfos).then((proteinInfos) => {
                    const oProteinInfos = proteinInfos.map((proteinInfo, index) => {
                        return proteinInfo;
                    });

                    const proteinInfoMap: Record<string, {
                        proteinInfo: UniProtEntry;
                        geneInfo: GeneInfo;
                    }> = {};
                    oProteinInfos.forEach((proteinInfo, index) => {
                        const genePair = remaingGenes[index];
                        const taxid = genePair[0].toString();
                        const geneInfo = allGeneInfos.find((geneInfo) => geneInfo.taxid.toString() === taxid);
                        proteinInfoMap[taxid] = {
                            proteinInfo,
                            geneInfo: geneInfo?.geneInfo || {} as GeneInfo
                        }
                    });

                    setAllProteinInfos(proteinInfoMap);
                }).catch((error) => {
                    console.error(error);
                });
            }
        }

        init();
    }, []);

    useEffect(() => {
        let oItems = allGeneInfos.map((geneInfoMap) => {
            const proteinInfo = allProteinInfos[geneInfoMap.taxid.toString()]?.proteinInfo;
            return {
                label: geneInfoMap.abbr,
                key: geneInfoMap.taxid,
                children: isProteinCoding(geneInfoMap.geneInfo) ?
                    < ProteinInfoPanel geneInfo={geneInfoMap.geneInfo} proteinInfo={proteinInfo} /> :
                    <GeneInfoPanel geneSymbol={geneInfoMap.geneInfo.symbol} />
            }
        });

        if (oItems.length === 0) {
            return;
        } else {
            const alignmentData = Object.keys(allProteinInfos).map((taxid) => {
                const proteinInfo = allProteinInfos[taxid].proteinInfo;
                const geneInfo = allProteinInfos[taxid].geneInfo;
                return {
                    sequence: proteinInfo.sequence.value,
                    species: guessSpecies(taxid),
                    geneSymbol: geneInfo.symbol,
                    entrezgene: geneInfo.entrezgene           
                }
            });
            oItems.push({
                label: 'Alignment',
                key: oItems.length + 1,
                children: <AlignmentViewer data={alignmentData} />
            })
        }

        setItems(oItems);
    }, [allGeneInfos]);

    return (items.length === 0 ?
        <Empty description="No information available." /> :
        <Tabs
            className="composed-protein-panel"
            tabPosition="left"
            items={items}
        />
    )
}

export default ComposedProteinPanel;
