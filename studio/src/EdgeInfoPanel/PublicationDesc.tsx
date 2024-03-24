import React, { useState } from 'react';
import { Button } from 'antd';
import parse from 'html-react-parser';
import type { PublicationDetail } from 'biominer-components/dist/typings';

export const SEPARATOR = '#';

const Desc: React.FC<{
    publication: PublicationDetail,
    showAbstract: (doc_id: string) => Promise<PublicationDetail>,
    showPublication: (publication: PublicationDetail) => void,
    queryStr: string
}> = (props) => {
    const { publication } = props;
    const [abstract, setAbstract] = useState<string>('');
    const [abstractVisible, setAbstractVisible] = useState<boolean>(false);

    const fetchAbstract = (doc_id: string) => {
        props.showAbstract(doc_id).then((publication) => {
            console.log('fetchAbstract for a publication: ', publication);
            setAbstract(publication.article_abstract || '');
            setAbstractVisible(true);
        }).catch((error) => {
            console.error('Error: ', error);
            setAbstract('');
            setAbstractVisible(false);
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
                {parse(highlightWords(publication.summary, props.queryStr.split(SEPARATOR)))}
                <Button type="link" onClick={() => {
                    if (abstractVisible) {
                        setAbstractVisible(false);
                    } else {
                        fetchAbstract(publication.doc_id);
                    }
                }} style={{ paddingLeft: '2px' }}>
                    {abstractVisible ? 'Hide Abstract' : 'Show Abstract'}
                </Button>
            </p>
            {
                abstractVisible ?
                    <p>{parse(highlightWords(abstract, props.queryStr.split(SEPARATOR)))}</p> : null
            }
            <p>
                {publication.year} | {publication.journal} &nbsp; | &nbsp; {publication.authors ? publication.authors.join(', ') : 'Unknown'}
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