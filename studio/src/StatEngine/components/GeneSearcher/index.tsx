import { Select, Empty, Tag, message } from 'antd';
import { filter, orderBy } from 'lodash';
import React, { useEffect, useState } from 'react';
import { fetchEntities } from '@/services/swagger/KnowledgeGraph';
import type { OptionType, Entity, ComposeQueryItem, QueryItem } from 'biominer-components/dist/typings';

const { Option } = Select;

export type GeneDataResponse = {
    total: number;
    page: number;
    page_size: number;
    data: Entity[];
};

export function makeQueryEntityStr(params: Partial<Entity>, order?: string[]): string {
    let query: ComposeQueryItem = {} as ComposeQueryItem;

    let label_query_item = {} as QueryItem;

    label_query_item = {
        operator: '=',
        field: 'label',
        value: 'Gene',
    };

    let filteredKeys = filter(Object.keys(params), (key) => key !== 'label');
    if (filteredKeys.length > 1) {
        query = {
            operator: 'or',
            items: [],
        };

        if (order) {
            // Order and filter the keys.
            filteredKeys = order.filter((key) => filteredKeys.includes(key));
        }
    } else {
        query = {
            operator: 'and',
            items: [],
        };
    }

    query['items'] = filteredKeys.map((key) => {
        return {
            operator: 'ilike',
            field: key,
            value: `%${params[key as keyof Entity]}%`,
        };
    });

    if (label_query_item.field) {
        if (query['operator'] === 'and') {
            query['items'].push(label_query_item);
        } else {
            query = {
                operator: 'and',
                items: [query, label_query_item],
            };
        }
    }

    return JSON.stringify(query);
}

export type GenesQueryParams = {
    /** Query string with biomedgps specification. */
    query_str: string;
    /** Page, From 1. */
    page?: number;
    /** Num of items per page. */
    page_size?: number;
};

export type GeneSearcherProps = {
    placeholder?: string;
    initialValue?: any;
    mode?: any;
    // When multiple values was returned, the gene variable will be undefined.
    onChange?: (value: string | string[], gene: Entity | undefined) => void;
    style: React.CSSProperties;
};

const GeneSearcher: React.FC<GeneSearcherProps> = props => {
    const { initialValue } = props;
    const [geneData, setGeneData] = useState<Entity[]>([]);
    const [data, setData] = useState<OptionType[]>([]);
    const [value, setValue] = useState<string>();

    let timeout: ReturnType<typeof setTimeout> | null;
    const fetchGenes = async (
        value: string,
        callback: (options: OptionType[]) => void,
    ) => {
        // We might not get good results when the value is short than 3 characters.
        if (value.length < 3) {
            callback([]);
            return;
        }

        if (timeout) {
            clearTimeout(timeout);
            timeout = null;
        }

        // TODO: Check if the value is a valid id.

        let queryMap = {};
        let order: string[] = [];
        // If the value is a number, then maybe it is an id or xref but not for name or synonyms.
        if (value && !isNaN(Number(value))) {
            queryMap = { id: value, xrefs: value };
            order = ['id', 'xrefs', 'label'];
        } else {
            queryMap = { name: value, synonyms: value, xrefs: value, id: value };
            order = ['name', 'synonyms', 'xrefs', 'id', 'label'];
        }

        const fetchData = () => {
            fetchEntities({
                query_str: makeQueryEntityStr(queryMap, order),
                page: 1,
                page_size: 50,
                // We only want to get all valid entities.
                model_table_prefix: 'biomedgps',
            })
                .then((response) => {
                    const { records } = response;
                    // @ts-ignore
                    const options: OptionType[] = records.map((item: Entity, index: number) => ({
                        order: index,
                        value: `${item['name']}`,
                        label: <span>{`${item['name']} | ${item['id']}`}</span>,
                        description: item['description'],
                        metadata: item,
                    }));
                    console.log('getLabels results: ', options);
                    callback(orderBy(options, ['value']));
                    setGeneData(records as Entity[]);
                })
                .catch((error) => {
                    if (error.response.status === 401) {
                        message.warning("Please login to see the search results.")
                    } else {
                        message.warning("Cannot get search results for your query. Please try again later.")
                    }
                    console.log('requestNodes Error: ', error);
                    callback([]);
                });
        };

        timeout = setTimeout(fetchData, 300);
    };

    useEffect(() => {
        // To avoid the loop updating.
        if (initialValue && initialValue !== value) {
            setValue(initialValue)
            fetchGenes(initialValue, (options) => {
                setData(options);
                handleChange(initialValue, {});
            })
        }
    }, [initialValue])

    const handleSearch = (newValue: string) => {
        if (newValue) {
            fetchGenes(newValue, setData);
        } else {
            setData([]);
        }
    };

    const handleChange = (newValue: string, option: any) => {
        setValue(newValue);
        console.log("GeneSearcher handleChange: ", newValue);
        if (newValue && typeof newValue == 'string') {
            const gene = filter(geneData, (item) => {
                if (newValue.match(/[a-zA-Z][a-zA-Z0-9]+/i)) {
                    return item.name == newValue
                } else if (newValue.match(/[0-9]+/i)) {
                    return item.id.toString() == newValue
                } else {
                    return false
                }
            })

            console.log("handleChange(GeneSearcher): ", gene, geneData);
            props.onChange?.(newValue, gene[0]);
        } else {
            props.onChange?.(newValue, undefined);
        }
    };

    const options = data.map(d => <Option key={d.value}>{d.label}</Option>);

    return (
        <Select
            allowClear
            showSearch
            value={value}
            placeholder={props?.placeholder}
            style={props.style}
            defaultActiveFirstOption={false}
            filterOption={false}
            onSearch={handleSearch}
            onChange={handleChange}
            mode={props?.mode ? props?.mode : 'single'}
            notFoundContent={<Empty description="Searching ..." />}
        >
            {options}
        </Select>
    );
};

export default GeneSearcher;