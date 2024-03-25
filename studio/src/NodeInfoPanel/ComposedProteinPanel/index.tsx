import { Empty, Tabs, Spin } from "antd";
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
    const uniprotIds = geneInfo.uniprot ? (geneInfo.uniprot.TrEMBL || []) : [];
    if (!uniprotId && uniprotIds.length === 0) {
        return {} as UniProtEntry;
    }

    if (uniprotId) {
        return fetchProteinInfo(uniprotId);
    } else {
        return fetchProteinInfo(uniprotIds[0]);
    }
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
    const [loading, setLoading] = useState<boolean>(false);

    useEffect(() => {
        // Get the geneInfo and proteinInfo from the mygene.info and uniprot API.
        // If possible, we would like to get all the geneInfo and proteinInfo from the homologene.
        const init = async () => {
            setLoading(true);
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

                console.log("All expected species and genes: ", remaingGenes);

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

                    const proteinInfos = remaingGenes.map(([taxid, entrezgene]) => {
                        if (taxid === geneInfo.taxid) {
                            return proteinInfo;
                        }
                        const gInfo = orderedGeneInfos.find((geneInfo) => geneInfo.taxid === taxid)?.geneInfo || {} as GeneInfo;
                        return fetchProteinInfoByGeneInfo(gInfo);
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
                            const geneInfo = orderedGeneInfos.find((geneInfo) => geneInfo.taxid.toString() === taxid);
                            proteinInfoMap[taxid] = {
                                proteinInfo,
                                // TODO: It might cause that we get the wrong fasta data at the AlignmentViewer.
                                geneInfo: geneInfo?.geneInfo || {} as GeneInfo
                            }
                        });

                        setAllProteinInfos(proteinInfoMap);
                    }).catch((error) => {
                        console.error(error);
                        setAllProteinInfos({});
                        setLoading(false);
                    });
                }).catch((error) => {
                    console.error(error);
                    setAllGeneInfos([]);
                    setLoading(false);
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
            let alignmentData: any[] = [];
            Object.keys(allProteinInfos).forEach((taxid) => {
                const proteinInfo = allProteinInfos[taxid].proteinInfo;
                const geneInfo = allProteinInfos[taxid].geneInfo;
                const uniProtType = geneInfo.uniprot && geneInfo.uniprot['Swiss-Prot'] ? 'Swiss-Prot' : 'TrEMBL';
                if (proteinInfo?.sequence?.value) {
                    alignmentData.push({
                        sequenceVersion: proteinInfo.entryAudit?.sequenceVersion || 0,
                        score: proteinInfo.annotationScore || 0,
                        proteinName: proteinInfo.uniProtkbId,
                        proteinDescription: proteinInfo.proteinDescription?.recommendedName?.fullName?.value || '',
                        uniProtId: proteinInfo.primaryAccession,
                        uniProtType,
                        sequence: proteinInfo.sequence.value,
                        species: guessSpecies(taxid),
                        geneSymbol: geneInfo.symbol,
                        entrezgene: geneInfo.entrezgene
                    })
                }
            });
            oItems.push({
                label: 'Alignment',
                key: oItems.length + 1,
                children: <AlignmentViewer data={alignmentData} />
            })
        }

        setItems(oItems);
    }, [allGeneInfos, allProteinInfos]);

    return (items.length === 0 ?
        <Spin spinning={loading}><Empty description={loading ? 'Loading' : 'No information available.'} /></Spin> :
        <Tabs
            className="composed-protein-panel"
            tabPosition="left"
            items={items}
        />
    )
}

export default ComposedProteinPanel;
