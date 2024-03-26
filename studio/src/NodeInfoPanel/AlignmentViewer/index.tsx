import React, { useEffect } from 'react';
import { Empty, Row, Spin } from 'antd';
// @ts-ignore
import { AlignmentViewer as ReactAlignmentViewer } from 'react-alignment-viewer';
import type { AlignmentData } from '../index.t';
import biomsa from 'biomsa';
import { expectedSpeciesOrder } from '@/components/util';

import './index.less';

type AlignmentViewerProps = {
    data: AlignmentData[];
};

const transformDataForAlignmentViewer = (dataArray: AlignmentData[]): string[] => {
    return dataArray.map(item => {
        const prefix = item.uniProtType === 'Swiss-Prot' ? 'sp' : 'tr';
        return `>${prefix}|${item.uniProtId}|${item.proteinName} ${item.proteinDescription} OS=${item.species} GN=${item.geneSymbol} PE=${item.score} SV=${item.sequenceVersion}\n${item.sequence}`
    });
}

const parseFasta = (fastaString: string) => {
    // Split the string into lines
    const lines = fastaString.trim().split(/\r?\n/);

    // Initialize an array to hold the parsed records
    const fastaRecords = [];

    // Temporary variables to hold the current record's information
    let currentId: string | null = null;
    let currentSeq: string[] = [];

    lines.forEach(line => {
        if (line.startsWith('>')) { // Header line
            if (currentId !== null) {
                // Save the previous record
                fastaRecords.push({
                    id: currentId,
                    seq: currentSeq.join('')
                });
                currentSeq = [];
            }
            // Extract ID (substring after '>' and before the first space if present)
            currentId = line;
        } else {
            // Sequence line, append to current sequence
            currentSeq.push(line);
        }
    });

    // Don't forget to save the last record
    if (currentId !== null) {
        fastaRecords.push({
            id: currentId,
            seq: currentSeq.join('')
        });
    }

    return fastaRecords;
}

const alignSeqs = async (seqs: string[]): Promise<string[]> => {
    return biomsa.align(seqs, {
        method: 'diag',
        type: 'amino',
        gapchar: '-',
    })
}

const AlignmentViewer: React.FC<AlignmentViewerProps> = (props) => {
    const [dataset, setDataset] = React.useState<string>("");
    const [errorMsg, setErrorMsg] = React.useState<string>("No data");
    const [loading, setLoading] = React.useState<boolean>(false);

    useEffect(() => {
        if (props.data && props.data.length > 0) {
            setLoading(true);
            const orderedData = props.data.sort((a, b) => {
                const indexA = expectedSpeciesOrder.indexOf(a.species);
                const indexB = expectedSpeciesOrder.indexOf(b.species);

                // Handling unknown species by sorting them to the end
                const unknownIndex = expectedSpeciesOrder.length;
                return (indexA === -1 ? unknownIndex : indexA) - (indexB === -1 ? unknownIndex : indexB);
            });
            console.log("AlignmentViewer orderedData: ", props.data, orderedData);
            const d = transformDataForAlignmentViewer(orderedData);
            const dataset = d.join('\n');
            console.log("AlignmentViewer dataset: ", dataset);

            const fastaRecords = parseFasta(dataset);
            const ids = fastaRecords.map(record => record.id);
            const seqs = fastaRecords.map(record => record.seq);
            alignSeqs(seqs).then((alignedSeqs: string[]) => {
                const alignedFastaRecords = ids.map((id, index) => `${id}\n${alignedSeqs[index]}`);
                const alignedDataset = alignedFastaRecords.join('\n');
                console.log("AlignmentViewer alignedDataset: ", alignedDataset);
                setDataset(alignedDataset);
                setErrorMsg("No data");
                setLoading(false);
            }).catch((error) => {
                console.error(error);
                setDataset("");
                setErrorMsg(error.message);
                setLoading(false);
            });
        }
    }, []);

    return <Row className="alignment-viewer-container" style={{ marginTop: '20px', width: '100%', height: '100%' }}>
        {
            dataset === "" ?
                <Spin spinning={loading}>
                    <Empty description={errorMsg} />
                </Spin> :
                <ReactAlignmentViewer data={dataset} height={"500px"} />
        }
    </Row>
}

export default AlignmentViewer;
