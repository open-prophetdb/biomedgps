import React, { useEffect, useState } from 'react';
import { Layout, Menu, Form, Input, InputNumber, Button, Select, Empty, Col, Row, Tooltip, message, Spin, Popover } from 'antd';
import { DotChartOutlined, DribbbleOutlined, AimOutlined, BranchesOutlined, BugOutlined, ZoomInOutlined } from '@ant-design/icons';
import { history } from 'umi';
// import { createFromIconfontCN } from '@ant-design/icons';
import { GraphTable } from 'biominer-components';
import { makeDataSources, pushGraphDataToLocalStorage } from 'biominer-components/dist/KnowledgeGraph/utils';
import { APIs, GraphData, COMPOSED_ENTITY_DELIMITER, Entity } from 'biominer-components/dist/typings';
import { fetchEntities, fetchPredictedNodes, fetchOneStepLinkedNodes } from '@/services/swagger/KnowledgeGraph';
import { EdgeAttribute } from 'biominer-components/dist/EdgeTable/index.t';
import { NodeAttribute } from 'biominer-components/dist/NodeTable/index.t';
import { makeQueryEntityStr } from 'biominer-components/dist/utils';
import { sortBy } from 'lodash';
import { fetchStatistics } from '@/services/swagger/KnowledgeGraph';
import { makeRelationTypes } from 'biominer-components/dist/utils';
import type { OptionType, RelationStat, ComposeQueryItem, QueryItem, GraphEdge, GraphNode } from 'biominer-components/dist/typings';
import EntityCard from '@/components/EntityCard';
import { truncateString } from '@/components/util';

import './index.less';

const { Header, Sider } = Layout;

const makeQueryStr = (entity_type: string, entity_id: string): string => {
  const source_query: ComposeQueryItem = {
    operator: 'and',
    items: [
      {
        field: 'source_type',
        operator: '=',
        value: entity_type,
      },
      {
        field: 'source_id',
        operator: '=',
        value: entity_id,
      },
    ],
  };

  const target_query: ComposeQueryItem = {
    operator: 'and',
    items: [
      {
        field: 'target_type',
        operator: '=',
        value: entity_type,
      },
      {
        field: 'target_id',
        operator: '=',
        value: entity_id,
      },
    ],
  };

  let query: ComposeQueryItem = {
    operator: 'or',
    items: [source_query, target_query],
  };

  return JSON.stringify(query);
}

// const IconFont = createFromIconfontCN({
//   scriptUrl: '//at.alicdn.com/t/c/font_3865804_no8ogbfj0q.js',
// });

type NodeIdSearcherProps = {
  allowMultiple?: boolean;
  placeholder?: string;
  entityType: string;
  handleSearchNode?: (entityType: string, value: string) => void;
  getEntities: APIs['GetEntitiesFn'];
  onSelect?: (value: string) => void;
}

let timeout: ReturnType<typeof setTimeout> | null;
// This function is used to fetch the entities of the selected entity type.
// All the nodes will be added to the options as a dropdown list.
export const fetchNodes = async (
  getEntities: APIs['GetEntitiesFn'],
  entityType: string,
  value: string,
  callback: (any: any) => void,
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
    queryMap = { id: value, xrefs: value, label: entityType };
    order = ['id', 'xrefs'];
  } else {
    queryMap = { name: value, synonyms: value, xrefs: value, id: value, label: entityType };
    order = ['name', 'synonyms', 'xrefs', 'id'];
  }

  // TODO: Need to add a prefix for the model table. In the future, we may allow users to select the model to confirm the model_table_prefix.

  const fetchData = () => {
    getEntities({
      query_str: makeQueryEntityStr(queryMap, order),
      model_table_prefix: 'biomedgps',
      page: 1,
      page_size: 50,
    })
      .then((response) => {
        const { records } = response;
        const options: OptionType[] = records.map((item: Entity, index: number) => ({
          order: index,
          value: item['id'],
          label: `${item['id']} | ${item['name']}`,
          description: item['description'],
          metadata: item,
        }));
        console.log('getLabels results: ', options);
        callback(options);
      })
      .catch((error) => {
        console.log('requestNodes Error: ', error);
        callback([]);
      });
  };

  timeout = setTimeout(fetchData, 300);
};

const NodeIdSearcher = (props: NodeIdSearcherProps) => {
  const [entityOptions, setEntityOptions] = useState<OptionType[] | undefined>(undefined);
  const [loading, setLoading] = useState(false);

  const handleSearchNode = function (entityType: string, value: string) {
    if (value) {
      setLoading(true);
      fetchNodes(props.getEntities, entityType, value, (options) => {
        setEntityOptions(options);
        setLoading(false);
      });
    } else {
      setEntityOptions(undefined);
    }
  };

  return <Select
    mode={props.allowMultiple ? 'multiple' : undefined}
    showSearch
    allowClear
    defaultActiveFirstOption={false}
    loading={loading}
    placeholder={props.placeholder}
    onChange={(value) => {
      props.onSelect && props.onSelect(value);
    }}
    onSearch={(value) => handleSearchNode(props.entityType, value)}
    getPopupContainer={(triggerNode: HTMLElement) => {
      return triggerNode.parentNode as HTMLElement;
    }}
    // options={entityOptions}
    filterOption={false}
    notFoundContent={
      <Empty
        description={
          loading
            ? 'Searching...'
            : entityOptions !== undefined
              ? 'Not Found or Too Short Input'
              : props.entityType === undefined
                ? 'Please select a node type first.'
                : `Enter your interested ${props.entityType} ...`
        }
      />
    }
  >
    {entityOptions &&
      entityOptions.map((option: any) => (
        <Select.Option key={option.value} value={option.value} disabled={option.disabled}>
          {option.metadata ? (
            <Popover
              mouseEnterDelay={0.5}
              placement="rightTop"
              title={option.label}
              content={EntityCard(option.metadata)}
              trigger="hover"
              getPopupContainer={(triggeredNode: any) => document.body}
              overlayClassName="entity-id-popover"
              autoAdjustOverflow={false}
              destroyTooltipOnHide={true}
              zIndex={1500}
            >
              {truncateString(option.label, 50)}
            </Popover>
          ) : (
            option.label
          )}
        </Select.Option>
      ))}
  </Select>;
}

type ModelParameter = {
  key: string;
  name: string;
  type: string;
  description: string;
  required: boolean;
  defaultValue?: any;
  entityType?: string;
  options?: any[];
  allowMultiple?: boolean;
}

type ModelItem = {
  shortName: string;
  name: string;
  icon: React.ReactNode;
  description: string;
  parameters: ModelParameter[];
  handler?: (params: any) => Promise<{
    params: any;
    data: GraphData;
  }>;
  disabled?: boolean;
}

const ModelConfig: React.FC = (props) => {
  const leftSpan = 6;
  const [form] = Form.useForm();
  const predictionType = Form.useWatch('prediction_type', form);

  const [loading, setLoading] = useState(false);
  const [currentModel, setCurrentModel] = useState(0);
  const [params, setParams] = useState({});
  const [graphData, setGraphData] = useState<GraphData>({ nodes: [], edges: [] });
  const [edgeDataSources, setEdgeDataSources] = useState<EdgeAttribute[]>([]);
  const [nodeDataSources, setNodeDataSources] = useState<NodeAttribute[]>([]);
  const [relationTypeOptions, setRelationTypeOptions] = useState<OptionType[]>([]);
  const [relationStat, setRelationStat] = useState<RelationStat[] | undefined>([]);

  useEffect(() => {
    fetchStatistics().then((data) => {
      const relationStats = data.relation_stat;
      setRelationStat(relationStats);

      const relationTypes = makeRelationTypes(relationStats);
      setRelationTypeOptions(relationTypes);
    });
  }, []);

  useEffect(() => {
    const entityType = form.getFieldValue('entity_type');
    const defaultRelationType = getDefaultRelationType(entityType, predictionType);
    form.setFieldsValue({ relation_type: defaultRelationType });

    // Reset the entity_id field when the prediction type is changed, because the change of prediction type may lead to the component of entity_id missing.
    form.setFieldsValue({ entity_id: undefined });
  }, [predictionType]);

  const formatScore = (score: number) => {
    // Keep 3 decimal places
    return parseFloat(score.toFixed(3));
  }

  useEffect(() => {
    if (graphData && graphData.edges) {
      const data = makeDataSources(graphData.edges).map((edge) => {
        return {
          ...edge,
          score: formatScore(edge.score)
        }
      });
      setEdgeDataSources(sortBy(data, ['score']).reverse());
    }

    if (graphData && graphData.nodes) {
      setNodeDataSources(makeDataSources(graphData.nodes));
    }
  }, [graphData]);

  useEffect(() => {
    cleanup();
  }, [currentModel]);

  const cleanup = () => {
    form.resetFields();
    setParams({});
    setGraphData({ nodes: [], edges: [] });
    cleanTable()
  }

  const cleanTable = () => {
    setEdgeDataSources([]);
    setNodeDataSources([]);
  }

  const [models, setModels] = useState<ModelItem[]>([{
    shortName: 'Disease',
    name: 'Prediction for Disease',
    icon: <BugOutlined />,
    description: 'To find TopK similar diseases, drugs or targets with a given disease',
    parameters: [{
      key: 'prediction_type',
      name: 'Prediction Type',
      type: 'select',
      description: 'Select a type for predicting the result, e.g. SimilarDisease is for predicting similar diseases for a given disease.',
      required: true,
      options: [
        { label: 'Similar Diseases', value: 'Disease' },
        { label: 'Predicted Drugs', value: 'Compound' },
        { label: 'Predicted Targets', value: 'Gene' }
      ],
      // defaultValue: 'Disease'
    },
    {
      key: 'relation_type',
      name: 'Relation Type for Prediction',
      type: 'RelationTypeSearcher',
      description: 'Select a relation type for predicting the result, e.g. Hetionet::DrD::Disease:Disease is for predicting similar diseases for a given disease. The number in the prefix of the relation type is the number of knowledges using to train the model. The larger the number may means the more reliable the prediction.',
      required: true,
      entityType: 'Disease'
    },
    {
      key: 'entity_id',
      name: 'Disease Name',
      type: 'NodeIdSearcher',
      description: 'Enter a name of disease for which you want to find similar diseases, drugs or targets. If you find multiple items, you might need to select the most relevant one.',
      required: true,
      entityType: 'Disease'
    },
    // {
    //   key: 'similarity_score_threshold',
    //   name: 'Similarity',
    //   type: 'number',
    //   description: 'Similarity threshold',
    //   defaultValue: 0.5,
    //   required: false
    // },
    {
      key: 'topk',
      name: 'TopK',
      type: 'number',
      description: 'Number of results to return',
      defaultValue: 10,
      required: false
    }],
    handler: (param: any) => {
      // const query = {
      //   operator: 'in',
      //   value: ["Disease"],
      //   field: 'entity_type',
      // };

      // const relation_type_map: Record<string, any> = {
      //   SimilarDisease: 'Hetionet::DrD::Disease:Disease',
      //   PredictedDrugs: 'DRUGBANK::treats::Compound:Disease',
      //   PredictedTargets: 'GNBR::J::Gene:Disease'
      // }

      // TODO: Need to update the relation_type automatically
      const relation_type = param.relation_type;

      let params: any = {
        node_id: `${param.entity_type}${COMPOSED_ENTITY_DELIMITER}${param.entity_id}`,
        relation_type: relation_type,
        topk: param.topk || 10,
      };

      // TODO: Do we need to add a query string?
      // if (query) {
      //   params['query_str'] = JSON.stringify(query);
      // }

      // TODO: How to use similarity_score_threshold?

      return new Promise((resolve, reject) => {
        fetchPredictedNodes(params).then((data) => {
          console.log('Diseases: ', params, data);
          resolve({
            params,
            data
          });
        }).catch((error) => {
          console.log('Diseases Error: ', error);
          reject({ nodes: [], edges: [], error: error })
        });
      });
    }
  },
  {
    shortName: 'Drug',
    name: 'Prediction for Drug',
    icon: <AimOutlined />,
    description: 'To predict similar drugs, indications or targets for a given drug',
    parameters: [{
      key: 'prediction_type',
      name: 'Prediction Type',
      type: 'select',
      description: 'Select a type for predicting the result, e.g. SimilarDrug is for predicting similar drugs for a given drug.',
      required: true,
      options: [
        { label: 'Similar Drugs', value: 'Compound' },
        { label: 'Predicted Indications', value: 'Disease' },
        { label: 'Predicted Targets', value: 'Gene' }
      ],
      // defaultValue: 'Compound'
    },
    {
      key: 'relation_type',
      name: 'Relation Type for Prediction',
      type: 'RelationTypeSearcher',
      description: 'Select a relation type for predicting the result, e.g. DRUGBANK::treats::Compound:Disease is for predicting diseases for a given drug. The number in the prefix of the relation type is the number of knowledges using to train the model. The larger the number may means the more reliable the prediction.',
      required: true,
      entityType: 'Compound'
    },
    {
      key: 'entity_id',
      name: 'Drug Name',
      type: 'NodeIdSearcher',
      description: 'Enter a name of drug for which you want to find similar drugs, indications or targets. If you find multiple items, you might need to select the most relevant one.',
      required: true,
      entityType: 'Compound'
    },
    // {
    //   key: 'score_threshold',
    //   name: 'Score',
    //   type: 'number',
    //   description: 'Score threshold',
    //   required: false,
    //   defaultValue: 0.5
    // },
    {
      key: 'topk',
      name: 'TopK',
      type: 'number',
      description: 'Number of results to return',
      required: false,
      defaultValue: 10
    }],
    handler: (param: any) => {
      // const relation_type_map: Record<string, any> = {
      //   SimilarDrug: 'Hetionet::CrC::Compound:Compound',
      //   PredictedIndications: 'DRUGBANK::treats::Compound:Disease',
      //   PredictedTargets: 'DRUGBANK::target::Compound:Gene'
      // }

      // TODO: Need to update the relation_type automatically
      const relation_type = param.relation_type;

      let params: any = {
        node_id: `${param.entity_type}${COMPOSED_ENTITY_DELIMITER}${param.entity_id}`,
        relation_type: relation_type,
        topk: param.topk || 10,
      };

      // TODO: Do we need to add a query string?
      // if (query) {
      //   params['query_str'] = JSON.stringify(query);
      // }

      // TODO: How to use similarity_score_threshold?

      return new Promise((resolve, reject) => {
        fetchPredictedNodes(params).then((data) => {
          console.log('Drugs: ', params, data);
          resolve({
            params,
            data
          });
        }).catch((error) => {
          console.log('Drugs Error: ', error);
          reject({ nodes: [], edges: [], error: error })
        });
      });
    }
  },
  {
    shortName: 'Gene',
    name: 'Prediction for Gene/Protein',
    icon: <ZoomInOutlined />,
    description: 'To predict drugs/diseases for a given gene/protein',
    parameters: [{
      key: 'prediction_type',
      name: 'Prediction Type',
      type: 'select',
      description: 'Select a type for predicting the result, e.g. PredictedDrugs is for predicting drugs for a given gene.',
      required: true,
      options: [
        { label: 'Predicted Drugs', value: 'Compound' },
        { label: 'Predicted Diseases', value: 'Disease' }
      ],
      // defaultValue: 'Compound'
    },
    {
      key: 'entity_id',
      name: 'Gene/Protein Name',
      type: 'NodeIdSearcher',
      description: 'Enter a name of gene for which you want to find drugs/diseases. If you find multiple items, you might need to select the most relevant one.',
      required: true,
      entityType: 'Gene'
    },
    {
      key: 'relation_type',
      name: 'Relation Type for Prediction',
      type: 'RelationTypeSearcher',
      description: 'Select a relation type for predicting the result, e.g. DRUGBANK::target::Compound:Gene is for predicting drugs for a given gene. The number in the prefix of the relation type is the number of knowledges using to train the model. The larger the number may means the more reliable the prediction.',
      required: true,
      entityType: 'Gene'
    },
    {
      key: 'topk',
      name: 'TopK',
      type: 'number',
      description: 'Number of results to return',
      required: false,
      defaultValue: 10
    }],
    handler: (param: any) => {
      // const relation_type_map: Record<string, any> = {
      //   PredictedDrugs: 'DRUGBANK::target::Compound:Gene',
      //   PredictedDiseases: 'GNBR::J::Gene:Disease'
      // }

      const relation_type = param.relation_type;

      let params: any = {
        node_id: `${param.entity_type}${COMPOSED_ENTITY_DELIMITER}${param.entity_id}`,
        relation_type: relation_type,
        topk: param.topk || 10,
      };

      return new Promise((resolve, reject) => {
        fetchPredictedNodes(params).then((data) => {
          console.log('Genes: ', params, data);
          resolve({
            params,
            data
          });
        }).catch((error) => {
          console.log('Genes Error: ', error);
          reject({ nodes: [], edges: [], error: error })
        });
      });
    }
  },
  {
    shortName: 'Symptom',
    name: 'Prediction for Symptom',
    icon: <DribbbleOutlined />,
    description: 'To predict drugs for a given group of symptoms',
    parameters: [{
      key: 'prediction_type',
      name: 'Prediction Type',
      type: 'select',
      description: 'Select a type for predicting the result, e.g. Disease is for predicting diseases for a given symptom.',
      required: true,
      options: [
        { label: 'Predicted Drugs', value: 'Compound' },
        { label: 'Predicted Diseases', value: 'Disease' },
      ],
      // defaultValue: 'Disease'
    },
    {
      key: 'relation_type',
      name: 'Relation Type for Prediction',
      type: 'RelationTypeSearcher',
      description: 'Select a relation type for predicting the result, e.g. HSDN::has_symptom::Disease:Symptom is for predicting diseases for a given symptom. The number in the prefix of the relation type is the number of knowledges using to train the model. The larger the number may means the more reliable the prediction.',
      required: true,
      entityType: 'Symptom'
    },
    {
      key: 'entity_id',
      name: 'Symptom Name',
      type: 'NodeIdSearcher',
      description: 'Enter a name of symptom for which you want to find similar drugs. If you find multiple items, you might need to select the most relevant one or select multiple items.',
      required: true,
      entityType: 'Symptom',
      allowMultiple: true
    },
    {
      key: 'topk',
      name: 'TopK',
      type: 'number',
      description: 'Number of results to return',
      required: false,
      defaultValue: 10
    }],
    handler: (param: any) => {
      console.log('Symptoms Parameters: ', param)
      const relation_type = param.relation_type;

      let node_id = '';
      if (param.entity_id.length > 1) {
        let node_ids = [];
        for (let i = 0; i < param.entity_id.length; i++) {
          node_ids.push(`${param.entity_type}${COMPOSED_ENTITY_DELIMITER}${param.entity_id[i]}`);
        }

        console.log("node_ids: ", node_ids)
        node_id = node_ids.join(',');
      } else {
        node_id = `${param.entity_type}${COMPOSED_ENTITY_DELIMITER}${param.entity_id}`;
      }

      let params: any = {
        node_id: node_id,
        relation_type: relation_type,
        topk: param.topk || 10,
      };

      return new Promise((resolve, reject) => {
        fetchPredictedNodes(params).then((data) => {
          console.log('Symptoms: ', params, data);
          resolve({
            params,
            data
          });
        }).catch((error) => {
          console.log('Symptoms Error: ', error);
          reject({ nodes: [], edges: [], error: error })
        });
      });
    },
  },
  {
    shortName: 'MOA',
    name: 'Predicted MOAs',
    icon: <BranchesOutlined />,
    description: 'To predict MOAs for a given drug and disease',
    parameters: [{
      key: 'entity_id',
      name: 'Disease',
      type: 'NodeIdSearcher',
      description: 'Enter a name of disease for which you want to find mode of actions',
      required: true,
      entityType: 'Disease'
    }, {
      key: 'entity_id',
      name: 'Drug',
      type: 'NodeIdSearcher',
      description: 'Enter a name of drug for which you want to find mode of actions',
      required: true,
      entityType: 'Drug'
    }, {
      key: 'topk',
      name: 'TopK',
      type: 'number',
      description: 'Number of results to return',
      required: false,
      defaultValue: 10
    }],
    disabled: true
  }])

  const handleMenuClick = (e: any) => {
    console.log('handleMenuClick: ', e);
    if (models[e.key]) {
      setCurrentModel(e.key);
    }
  };

  const getDefaultRelationType = (entityType: string, predictionType: string) => {
    const DefaultRelationTypeMap: Record<string, string> = {
      'Disease:Disease': 'Hetionet::DrD::Disease:Disease',
      'Compound:Compound': 'Hetionet::CrC::Compound:Compound',
      'Disease:Compound': 'DRUGBANK::treats::Compound:Disease',
      'Disease:Gene': 'GNBR::J::Gene:Disease',
      'Compound:Disease': 'DRUGBANK::treats::Compound:Disease',
      'Compound:Gene': 'DRUGBANK::target::Compound:Gene',
      'Gene:Disease': 'GNBR::J::Gene:Disease',
      // TODO: the relation type is non-standard
      'Symptom:Disease': 'HSDN::has_symptom:Disease:Symptom',
      'Symptom:Compound': 'DrugBank::treats::Compound:Symptom',
    };

    const entityPair = `${entityType}:${predictionType}`;
    return DefaultRelationTypeMap[entityPair]
  }


  const detectComponent = (item: ModelParameter, onChange: (value: any) => void): React.ReactNode => {
    if (item.type === 'NodeIdSearcher') {
      return <NodeIdSearcher
        placeholder={item.description}
        entityType={item.entityType || 'Disease'}
        onSelect={(value) => {
          onChange(value);
        }}
        allowMultiple={item.allowMultiple}
        handleSearchNode={(entityType, value) => console.log(entityType, value)}
        // @ts-ignore
        getEntities={fetchEntities}
      />
    } else if (item.type === 'RelationTypeSearcher') {
      console.log("RelationTypeSearcher: ", item, relationTypeOptions, form.getFieldValue('entity_type'), predictionType);

      // TODO: Need to improve the regex to match the standard format of relation type.
      let filteredRelationTypeOptions = relationTypeOptions.filter((option) => {
        return item.entityType ? option.value.indexOf(item.entityType) !== -1 && option.value.match(/[a-zA-Z\+_\-]+::[a-zA-Z\+_\-]+::?[a-zA-Z]+:[a-zA-Z]+/g) : true;
      });

      let defaultRelationType = undefined;
      if (predictionType) {
        if (predictionType === item.entityType) {
          filteredRelationTypeOptions = filteredRelationTypeOptions.filter((option) => {
            return option.value.indexOf(`${predictionType}:${predictionType}`) !== -1;
          })
        } else {
          filteredRelationTypeOptions = filteredRelationTypeOptions.filter((option) => {
            return option.value.indexOf(predictionType) !== -1;
          })
        }

        if (item.entityType) {
          defaultRelationType = getDefaultRelationType(item.entityType, predictionType) || filteredRelationTypeOptions[0]?.value;
        }
      };

      return <Select
        filterOption={(input, option) => {
          console.log('filterOption: ', input, option);
          // @ts-ignore
          return option?.key.toLowerCase().indexOf(input.toLowerCase()) >= 0;
        }}
        getPopupContainer={(triggerNode) => {
          return triggerNode.parentNode;
        }}
        showSearch
        allowClear
        defaultValue={defaultRelationType}
        autoClearSearchValue={false}
        placeholder="Please select relation type for predicting the result"
        onSelect={(value) => onChange(value)}
      >
        {filteredRelationTypeOptions.map((item: OptionType) => {
          return (
            <Select.Option key={item.value} value={item.value}>
              <Tooltip title={item.description} placement="right">
                <div className="option-container">
                  <div className="option-label">{item.label}</div>
                  <div className="option-description">{item.description}</div>
                </div>
              </Tooltip>
            </Select.Option>
          );
        })}
      </Select >
    } else if (item.type === 'number') {
      return <InputNumber
        style={{ width: '100%' }}
        onChange={(value) => onChange(value)}
        placeholder={item.description}
        min={1}
        max={500}
      />
    } else if (item.type === 'select') {
      return <Select
        style={{ width: '100%' }}
        onChange={(value) => onChange(value)}
        placeholder={item.description}
        options={item.options || []}
      />
    } else {
      return <Input
        style={{ width: '100%' }}
        onChange={(event) => onChange(event.target.value)}
        placeholder={item.description}
      />
    }
  }

  const renderForm = () => {
    const parameters = models[currentModel].parameters;
    const entityIdIndex = parameters.findIndex((param) => param.key === 'entity_id');

    const formItems = models[currentModel].parameters.map((param, index) => {
      return (
        <Form.Item
          key={index}
          label={param.name}
          initialValue={param.defaultValue || undefined}
          name={param.key}
          required={param.required}
          tooltip={param.description}
          rules={[{ required: param.required, message: `${param.description}` }]}
        >
          {detectComponent(param, (value) => {
            form.setFieldValue(param.key, value);
            if (param.entityType) {
              form.setFieldValue('entity_type', param.entityType);
            }
            console.log("onSelect: ", param.key, value, form.getFieldsValue(), form.getFieldValue('entity_type'));
          })}
        </Form.Item>
      );
    });

    if (entityIdIndex !== -1) {
      const param = parameters[entityIdIndex];
      // Add entity type field into the formItems array where the position is entityIdIndex + 1
      formItems.splice(entityIdIndex, 0, <Form.Item
        key='entity_type'
        label='Which Type'
        hidden={true}
        initialValue={param.entityType || 'Disease'}
        name='entity_type'
        required={param.required}
        tooltip="The type of entity you want to search for."
      >
        <Input disabled value={param.entityType} />
      </Form.Item>);
    }

    return formItems;
  };

  // Placeholder function for submitting the form
  const handleSubmit = () => {
    // We need to clean the table before we submit the form, otherwise, the table will show the previous result.
    cleanTable();
    setLoading(true);
    form
      .validateFields()
      .then((values) => {
        const updatedValues = form.getFieldsValue();
        console.log('ModelConfig - onConfirm: ', values, updatedValues);

        const model = models[currentModel];
        if (model && model.handler) {
          model.handler(updatedValues).then((resp) => {
            const { params, data } = resp;
            console.log('ModelConfig - onConfirm - handler: ', params, data);
            setParams(params);
            setGraphData(data);
          }).catch((error) => {
            console.log('ModelConfig - onConfirm - handler - Error: ', error);
            message.warning("Cannot find any result for the given parameters.", 5)
            setParams({});
            setGraphData(error);
          }).finally(() => {
            setLoading(false);
          });
        } else {
          setLoading(false);
        }
      })
      .catch((error) => {
        console.log('onConfirm Error: ', error);
        setLoading(false);
      });
  };

  const detectColor = (modelName: string) => {
    if (modelName === models[currentModel].name) {
      return '#000000d9';
    } else {
      return '#999';
    }
  }

  return (
    <Layout className='model-panel' key={currentModel}>
      <Sider width={100}>
        <Menu mode="inline" defaultSelectedKeys={['0']} style={{ height: '100%' }} onClick={handleMenuClick} selectedKeys={[currentModel.toString()]}>
          {models.map((model, index) => (
            <Menu.Item key={index} icon={null} disabled={model.disabled}>
              <Tooltip title={`${model.disabled ? 'Disabled' : ''} > ${model.name} | ${model.description}`} placement="right" key={index}>
                <Button icon={model.icon} size='large' style={{ color: detectColor(model.name) }}></Button>
              </Tooltip>
              <span>{model.shortName}</span>
            </Menu.Item>
          ))}
        </Menu>
      </Sider>
      <Row className='model-config-panel' gutter={16}>
        <Col className="model-parameter" span={leftSpan}>
          <Header className="model-parameter-header">
            <h3>{models[currentModel].name}</h3>
            <p>{models[currentModel].description}</p>
          </Header>
          <Form layout="vertical" onFinish={handleSubmit} className='model-parameter-body' form={form} key={predictionType}>
            {renderForm()}
            <Button type="primary" htmlType="submit" className='model-parameter-button'
              size='large' loading={loading}>
              Apply Parameters
            </Button>
          </Form>
        </Col>
        <Col className="model-result" span={24 - leftSpan}>
          {loading ?
            <Empty description='Predicting using the given parameters...'
              style={{ display: 'flex', justifyContent: 'center', alignItems: 'center', height: '100%', flexDirection: 'column' }}>
              <Spin size="large" />
            </Empty> :
            <GraphTable edgeDataSources={edgeDataSources} nodeDataSources={nodeDataSources} key={JSON.stringify(params)}
              emptyMessage='Please setup related parameters in the left side and generate some predicted result first.'
              onExplainRow={(row: EdgeAttribute) => {
                setLoading(true);
                const source_id = row.source_id;
                const source_type = row.source_type;
                const target_id = row.target_id;
                const target_type = row.target_type;
                const relation_type = row.reltype;

                const first_fn = fetchOneStepLinkedNodes({
                  query_str: makeQueryStr(source_type, source_id),
                  page_size: 40
                })

                const second_fn = fetchOneStepLinkedNodes({
                  query_str: makeQueryStr(target_type, target_id),
                  page_size: 40
                })

                Promise.all([first_fn, second_fn]).then((responses) => {
                  const source_nodes = responses[0];
                  const target_nodes = responses[1];

                  let d = {
                    nodes: source_nodes.nodes.concat(target_nodes.nodes) as GraphNode[],
                    edges: source_nodes.edges.concat(target_nodes.edges) as GraphEdge[]
                  }

                  const edges = edgeDataSources
                    .filter((edge) => row.relid === edge.relid)
                    .map((edge) => edge.metadata);

                  d = {
                    nodes: d.nodes,
                    edges: d.edges.concat(edges as GraphEdge[])
                  }

                  console.log('ExplainRow: ', row, d, source_nodes, target_nodes, edges);
                  setLoading(false);
                  if (d && d.nodes && d.nodes.length > 0) {
                    pushGraphDataToLocalStorage(d);
                    history.push('/knowledge-graph');
                  } else {
                    message.warning("Cannot find an attention subgraph for explaining the predicted relation.", 5)
                  }
                }).catch((error) => {
                  setLoading(false);
                  console.log('ExplainRow Error: ', error);
                  message.warning("Cannot find an attention subgraph for explaining the predicted relation.", 5)
                });
              }}
              onLoadGraph={(graph) => {
                console.log('onLoadGraph: ', graph);
                if (graph && graph.nodes && graph.nodes.length > 0) {
                  pushGraphDataToLocalStorage(graph);
                  history.push('/knowledge-graph');
                } else {
                  message.warning("You need to generate some predicted result and pick up the interested rows first.", 5)
                }
              }}
              edgeStat={relationStat}
            />
          }
        </Col>
      </Row>
    </Layout>
  );
};

export default ModelConfig;
