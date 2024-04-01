import type { Entity } from 'biominer-components/dist/typings';
import { guessColor, guessSpecies } from '../util';
import { Tag } from 'antd';
import { uniq } from 'lodash';

const EntityCard = (metadata: Entity | undefined) => {
    if (!metadata) {
        return <div>No metadata found!</div>;
    } else {
        return (
            <div style={{ overflowWrap: 'break-word', width: '420px', maxHeight: '200px', overflow: 'scroll' }}
                onClick={(e) => {
                    e.stopPropagation();
                }}>
                <p style={{ marginBottom: '5px' }}>
                    <span style={{ fontWeight: 'bold' }}>Species: </span>
                    {guessSpecies(`${metadata.taxid}` || '')}
                </p>
                <p style={{ marginBottom: '5px' }}>
                    <span style={{ fontWeight: 'bold' }}>Synonyms: </span>
                    {
                        metadata.synonyms ? uniq(metadata.synonyms.split("|")).map(item => {
                            return <Tag key={item}>{item}</Tag>;
                        }) : 'No synonyms found!'
                    }
                </p>
                <p style={{ marginBottom: '5px' }}>
                    <span style={{ fontWeight: 'bold' }}>Xrefs: </span>
                    {metadata.xrefs || 'No xrefs found!'}
                </p>
                <p style={{ marginBottom: '5px' }}>
                    <span style={{ fontWeight: 'bold' }}>Description: </span>
                    {metadata.description || 'No description found!'}
                </p>
                <p style={{ marginBottom: '5px' }}>
                    <span style={{ fontWeight: 'bold' }}>ID: </span>
                    {metadata.id}
                </p>
                <p style={{ marginBottom: '5px' }}>
                    <span style={{ fontWeight: 'bold' }}>Name: </span>
                    {metadata.name}
                </p>
                <p style={{ marginBottom: '5px' }}>
                    <span style={{ fontWeight: 'bold' }}>Label: </span>
                    <Tag color={guessColor(metadata.label)}>{metadata.label}</Tag>
                </p>
            </div>
        );
    }
};

export default EntityCard;