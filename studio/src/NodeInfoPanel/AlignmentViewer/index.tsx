import React, { useEffect } from 'react';
import { Empty, Row } from 'antd';
// @ts-ignore
import { AlignmentViewer as ReactAlignmentViewer } from 'react-alignment-viewer';
import type { AlignmentData } from '../index.t';
import './index.less';

type AlignmentViewerProps = {
    data: AlignmentData[];
};

function transformDataForAlignmentViewer(dataArray: AlignmentData[]): string[] {
    return dataArray.map(item => {
        const prefix = item.uniProtType === 'Swiss-Prot' ? 'sp' : 'tr';
        return `>${prefix}|${item.uniProtId}|${item.proteinName} ${item.proteinDescription} OS=${item.species} GN=${item.geneSymbol} PE=${item.score} SV=${item.sequenceVersion}\n${item.sequence}`
    });
}

const AlignmentViewer: React.FC<AlignmentViewerProps> = (props) => {
    const [dataset, setDataset] = React.useState<string>("");

    useEffect(() => {
        if (props.data && props.data.length > 0) {
            const d = transformDataForAlignmentViewer(props.data);
            const dataset = d.join('\n');
            console.log("AlignmentViewer dataset: ", dataset);
            setDataset(dataset);
        }
    }, []);

    return <Row className="alignment-viewer-container" style={{ marginTop: '20px', width: '100%', height: '100%' }}>
        {
            dataset === "" ?
                <Empty description="No data" /> :
                <ReactAlignmentViewer data={dataset} />
        }
    </Row>
}

export default AlignmentViewer;
