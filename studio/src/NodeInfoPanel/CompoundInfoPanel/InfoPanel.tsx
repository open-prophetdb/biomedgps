import { Empty, Row, Col, Tag, Descriptions, Table, Spin } from "antd";
import React, { useEffect, useState } from "react";
import type { DescriptionsProps } from 'antd';
import { CompoundInfo } from "./index.t";

import './index.less';

export interface InfoPanelProps {
    compoundInfo: CompoundInfo
}

const PubMedLinks = (text: string) => {
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

const BoldTextComponent = ({ text }: { text: string }) => {
    // Regex to find all text surrounded by double asterisks
    const regex = /\*\*(.*?)\*\*/g;
    let parts = [];
    let lastIndex = 0;

    // @ts-ignore
    text.replace(regex, (match, p1, offset) => {
        // Add text before the match
        parts.push(text.substring(lastIndex, offset));
        // Add a line break before the bold text, if it's not the first match and there's text before it
        if (offset > 0 && text.substring(lastIndex, offset).trim().length > 0) {
            parts.push(<><br /><br /></>); // Add a line break before the bold text
        }
        parts.push(<><b key={offset}>{p1}</b><br /></>); // Add the bold text
        lastIndex = offset + match.length;
    });

    // If there's text after the last match and it's not immediately following a bold text, add a line break
    // if (lastIndex < text.length && lastIndex !== 0) {
    //     parts.push(<br key={`br-${lastIndex}`} />); // Add a line break before the last piece of text
    // }

    // Add the last piece of text
    parts.push(text.substring(lastIndex));

    return <>{parts}</>;
};



export const InfoPanel: React.FC<InfoPanelProps> = (props) => {
    const { compoundInfo } = props;
    // @ts-ignore
    const [generalInfo, setGeneralInfo] = useState<DescriptionsProps['items']>([]);
    const [background, setBackground] = useState<Record<string, string>>({});
    const titleMap: Record<string, string> = {
        description: 'Description',
        synthesisReference: 'Synthesis Reference',
        indication: 'Indication',
        pharmacodynamics: 'Pharmacodynamics',
        mechanism_of_action: 'Mechanism of Action',
        toxicity: 'Toxicity',
        metabolism: 'Metabolism',
        absorption: 'Absorption',
        half_life: 'Half Life',
    };

    useEffect(() => {
        if (compoundInfo) {
            const drugbankId = compoundInfo.drugbank_id.replace('DrugBank:', '')
            // @ts-ignore
            const generalInfo: DescriptionsProps['items'] = [
                {
                    key: 'drugbank-id',
                    label: 'DrugBank ID',
                    children: <a href={`https://go.drugbank.com/drugs/${drugbankId}`} target="_blank">
                        {drugbankId}
                    </a>,
                },
                {
                    key: 'official-full-name',
                    label: 'Official Full Name',
                    children: compoundInfo.name,
                },
                {
                    key: 'cas-number',
                    label: 'CAS Number',
                    children: compoundInfo.cas_number,
                },
                {
                    key: 'unii',
                    label: 'UNII',
                    children: compoundInfo.unii,
                },
                {
                    key: 'state',
                    label: 'State',
                    children: compoundInfo.compound_state
                },
                {
                    key: 'groups',
                    label: 'Groups',
                    children: compoundInfo.groups ? compoundInfo.groups.map((group: string) => <Tag key={group}>{group}</Tag>) : null
                },
                {
                    key: 'type',
                    label: 'Category',
                    children: compoundInfo.compound_type
                },
                {
                    key: 'created',
                    label: 'Created',
                    children: compoundInfo.created
                },
                {
                    key: 'updated',
                    label: 'Updated',
                    children: compoundInfo.updated
                }
            ];

            setGeneralInfo(generalInfo);

            const background: Record<string, string> = {
                description: compoundInfo?.description,
                synthesisReference: compoundInfo?.synthesis_reference,
                indication: compoundInfo?.indication,
                pharmacodynamics: compoundInfo?.pharmacodynamics,
                mechanism_of_action: compoundInfo?.mechanism_of_action,
                toxicity: compoundInfo?.toxicity,
                metabolism: compoundInfo?.metabolism,
                absorption: compoundInfo?.absorption,
                half_life: compoundInfo?.half_life,
            }

            setBackground(background);
        }
    }, [compoundInfo]);

    return (
        compoundInfo ?
            <Row className="compound-info-panel">
                <Col className="general-information">
                    {/* @ts-ignore */}
                    <Descriptions title="General Information" bordered items={generalInfo} column={2} />
                </Col>
                <Col className="biology-background">
                    {
                        Object.keys(background).map((key) => {
                            return <div className="section" key={key}>
                                <h2 className="title">{titleMap[key] || key}</h2>
                                <p className="desc">
                                    <BoldTextComponent text={background[key]} />
                                </p>
                            </div >
                        })
                    }
                </Col>
            </Row>
            : null
    );
}

export default InfoPanel;
