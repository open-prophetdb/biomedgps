import React from 'react';
import { Row } from 'antd';
// @ts-ignore
import { AlignmentViewer as ReactAlignmentViewer } from 'react-alignment-viewer';
import type { AlignmentData } from '../index.t';
import './index.less';

type AlignmentViewerProps = {
    data: AlignmentData[];
};

function transformDataForAlignmentViewer(dataArray: AlignmentData[]): string[] {
    return dataArray.map(item => `>sp|${item.uniProtId}|${item.proteinName} ${item.proteinDescription} OS=${item.species} GN=${item.geneSymbol} PE=${item.score} SV=${item.sequenceVersion}\n${item.sequence}`);
}

const AlignmentViewer: React.FC<AlignmentViewerProps> = (props) => {
    const [dataset, setDataset] = React.useState<string[]>(transformDataForAlignmentViewer(props.data));

    return <Row className="alignment-viewer-container" style={{ marginTop: '20px', width: '100%', height: '100%' }}>
        <ReactAlignmentViewer data={dataset.join('\n')} />
    </Row>
}

export default AlignmentViewer;
