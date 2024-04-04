import React, { useEffect, useState } from 'react'
import { Empty, Spin, Tabs, Descriptions, Table } from 'antd'
import type { CompoundInfo, Patent } from './index.t';
import InfoPanel from './InfoPanel';
import Reference from './Reference';

type CompoundInfoPanelProps = {
    rootId?: string,
    // DrugBank ID
    compoundInfo: CompoundInfo
}

const formatPatentNumber = (country: string, number: string): string | null => {
    if (country === 'United States') {
        return `US${number}`;
    } else if (country === 'Canada') {
        return `CA${number}`;
    } else {
        return null;
    }
}

const CompoundInfoPanel: React.FC<CompoundInfoPanelProps> = (props) => {
    const { compoundInfo } = props;

    const [items, setItems] = useState<any[]>([]);
    const [patentColumns, setPatentColumns] = useState<any[]>([]);
    const [loading, setLoading] = useState<boolean>(false);

    useEffect(() => {
        if (!compoundInfo) {
            return;
        }

        const patentKeyMap: Record<string, string> = {
            country: 'Country',
            approved: 'Approved Date',
            expires: 'Expires Date',
            number: 'Patent Number',
            pediatric_extension: 'Pediatric Extension',
        };

        const getColumn = (key: string) => ({
            title: patentKeyMap[key],
            dataIndex: key,
            key,
            align: 'center',
            render: (text: string, record: Patent) => {
                if (key === 'number') {
                    const patentNumber = formatPatentNumber(record.country, record.number);
                    return patentNumber ? <a href={`https://patents.google.com/patent/${patentNumber}`} target="_blank">{text}</a> : text;
                }

                return text;
            }
        });

        if (compoundInfo.patents) {
            setPatentColumns(Object.keys(compoundInfo.patents[0]).map(getColumn));
        }
    }, [compoundInfo])

    useEffect(() => {
        if (!compoundInfo) {
            return;
        }

        if (patentColumns.length === 0) {
            return;
        }

        setItems([
            {
                label: "General",
                key: "general",
                children: <InfoPanel compoundInfo={compoundInfo} />
            },
            {
                label: 'Patents',
                key: 'patents',
                children: <Table dataSource={compoundInfo.patents} columns={patentColumns} rowKey='key' pagination={
                    {
                        pageSize: 100,
                        hideOnSinglePage: true
                    }
                } />
            },
            {
                label: 'Clinical Trials',
                key: 'clinicalTrials',
                children: <Empty description="Coming soon..." />
            },
            {
                label: 'References',
                key: 'references',
                children: <Reference compoundInfo={compoundInfo} />
            },
            {
                label: 'Drug Interactions',
                key: 'drugInteractions',
                children: <Empty description="Coming soon..." />
            }
        ])
    }, [compoundInfo, patentColumns])

    return (items.length === 0 ?
        <Spin spinning={loading}><Empty description={loading ? 'Loading' : 'No information available.'} /></Spin> :
        <Tabs
            className="composed-compound-info-panel"
            tabPosition="left"
            items={items}
        />
    )
}

export default CompoundInfoPanel;
