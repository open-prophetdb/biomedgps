import React, { useState } from 'react';
import { Button, Tag } from 'antd';
import parse from 'html-react-parser';
import type { PublicationDetail } from 'biominer-components/dist/typings';

const Desc: React.FC<{
    publication: PublicationDetail,
    abstract: string,
    showAbstract: (doc_id: string) => Promise<PublicationDetail>,
    showPublication: (publication: PublicationDetail) => void,
    startNode?: string,
    endNode?: string,
}> = (props) => {
    const { publication } = props;
    const words = [props.startNode || '', props.endNode || ''];

    const fetchAbstract = (doc_id: string) => {
        props.showAbstract(doc_id).then((publication) => {
            console.log('Publication: ', publication);
        }).catch((error) => {
            console.error('Error: ', error);
        });
    };

    const escapeRegExp = (str: string) => {
        return str.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    }

    const highlightWords = (text: string, words: string[]): string => {
        let newText = text;
        words.forEach(word => {
            let escapedWord = escapeRegExp(word);
            let regex = new RegExp(`(${escapedWord})(?![^<]*>|[^<>]*<\/)`, 'gi');
            newText = newText.replace(regex, '<span class="highlight">$1</span>');
        });

        return newText;
    }

    return (
        <div>
            <p>
                {parse(highlightWords(publication.summary, words))}
                <Button type="link" onClick={() => {
                    if (!props.abstract) {
                        fetchAbstract(publication.doc_id);
                    }
                }} style={{ paddingLeft: '2px' }}>
                    {props.abstract ? 'Hide Abstract' : 'Show Abstract'}
                </Button>
            </p>
            {
                props.abstract ?
                    <p>{parse(highlightWords(props.abstract, words))}</p> : null
            }
            <p>
                <Tag>{publication.year}</Tag><Tag>Journal&nbsp;|&nbsp;{publication.journal}</Tag>&nbsp;{publication.authors ? publication.authors.join(', ') : 'Unknown'}
            </p>
            {
                <p>
                    Cited by {publication.citation_count ? publication.citation_count : 0} publications &nbsp; | &nbsp;
                    <a onClick={(e) => { props.showPublication(publication) }}>View Publication</a>
                </p>
            }
        </div>
    );
};

export default Desc;