import React from 'react';
import { Row } from 'antd';
// @ts-ignore
import { AlignmentViewer as ReactAlignmentViewer } from 'react-alignment-viewer';
import type { AlignmentData } from '../index.t';

type AlignmentViewerProps = {
    data: AlignmentData[];
};

function transformDataForAlignmentViewer(dataArray: AlignmentData[]): string[] {
    return dataArray.map(item => `>${item.species}|${item.geneSymbol}|${item.entrezgene}\n${item.sequence}`);
}

const AlignmentViewer: React.FC<AlignmentViewerProps> = (props) => {
    const [dataset, setDataset] = React.useState<string[]>(transformDataForAlignmentViewer(props.data));

    return <Row className="alignment-viewer-container" style={{ width: '100%', height: '100%' }}>
        <ReactAlignmentViewer data={dataset} />
    </Row>
}

export default AlignmentViewer;
