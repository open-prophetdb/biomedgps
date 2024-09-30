import { DownloadOutlined, EditOutlined, UploadOutlined } from '@ant-design/icons';
import type { ProFormColumnsType, ProFormLayoutType } from '@ant-design/pro-form';
import { BetaSchemaForm, ProProvider, ProFormSelect } from '@ant-design/pro-components';
import { Button, Col, Empty, Row, Space, Tooltip, Form, Select } from 'antd';
import GeneSearcher from '../GeneSearcher';
import { GenesQueryParams, GeneDataResponse } from '../GeneSearcher';
import FormItem from 'antd/lib/form/FormItem';
import React, { memo, useContext, useEffect, useState } from 'react';
import type { TaskHistory } from '../WorkflowList/data';

import './index.less';

type DataItem = {
  name: string;
  state: string;
};

const datasets = [
  {
    datasetName: "GSE251790_Female_CSF",
    fieldValue: {
      exp_file: "/data/biomedgps/cromwell/data/gene_expression.tsv",
      sample_info_file: "/data/biomedgps/cromwell/data/sample_info.tsv",
    },
    fieldValueEnum: {
      which_groups: ["Female_MECFS", "Female_Control"]
    }
  }
]

export type ArgumentProps = {
  // queryGenes: (params: GenesQueryParams) => Promise<GeneDataResponse>;
  columns: ProFormColumnsType<DataItem>[];
  fieldsValue?: any;
  contextData?: any; // TODO: Can we add all metadata about a dataset here?
  height?: string;
  labelSpan?: number;
  onSubmit?: (values: any) => Promise<TaskHistory>;
  readonly?: boolean;
};

const ArgumentForm: React.FC<ArgumentProps> = (props) => {
  const { height, labelSpan, onSubmit, fieldsValue } = props;

  const [columns, setColumns] = useState<ProFormColumnsType<DataItem>[]>(props.columns);

  const activateBtn = (
    <FormItem
      label="Editor"
      style={{ width: '50%' }}
      labelCol={{ span: 6 }}
      wrapperCol={{ span: 18 }}
    >
      <Button style={{ width: '100%' }}>
        <EditOutlined />
        Edit
      </Button>
    </FormItem>
  );

  const [layoutType, setLayoutType] = useState<ProFormLayoutType>('QueryFilter');
  const [form] = Form.useForm();

  // Ensure columns are initialized on first load
  useEffect(() => {
    if (props.columns && props.columns.length > 0) {
      setColumns(props.columns);
    }
  }, [props.columns]);

  useEffect(() => {
    if (fieldsValue) {
      form.setFieldsValue(fieldsValue)
    }
  }, [fieldsValue, columns])

  console.log('ArgumentForm updated');

  const values = useContext(ProProvider);
  return columns && columns.length > 0 ? (
    <Row className="argument-form">
      <ProProvider.Provider
        value={{
          ...values,
          valueTypeMap: {
            gene_searcher: {
              render: (text: any) => <a>{text}</a>,
              renderFormItem: (text: any, props: any) => {
                console.log("Gene Searcher Component: ", props, form.getFieldValue(props?.id))
                const initialValue = form.getFieldValue(props?.id)
                return (<GeneSearcher
                  placeholder="Enter gene symbol, entrez id"
                  initialValue={initialValue ? initialValue : props?.formItemProps?.initialValue}
                  {...props?.fieldProps}
                  mode={props?.fieldProps?.mode ? props?.fieldProps?.mode : 'single'}
                  style={{ width: '100%' }} />)
              },
            },
            dataset_searcher: {
              render: (text: any) => <a>{text}</a>,
              renderFormItem: (text: any, props: any) => {
                console.log("Dataset Searcher Component: ", props, text)
                return <Select placeholder="Select a dataset" onChange={(value) => {
                  const dataset = datasets.find((dataset) => dataset.datasetName === value)
                  console.log("Dataset Searcher Component: ", dataset)
                  const fieldValue = dataset?.fieldValue ?? {};

                  form.setFieldsValue({
                    ...fieldValue
                  })

                  const fieldValueEnum = dataset?.fieldValueEnum ?? {};
                  // Update the valueEnum of which_groups
                  const updatedColumns = columns.map((column) => {
                    const keys = Object.keys(fieldValueEnum);
                    if (keys.includes(column.dataIndex as string)) {
                      const valueEnum = fieldValueEnum[column.dataIndex as keyof typeof fieldValueEnum] as string[];
                      return {
                        ...column,
                        valueEnum: valueEnum.reduce((acc: any, cur: any) => {
                          acc[cur] = cur;
                          return acc;
                        }, {})
                      };
                    }

                    return column;
                  });
                  setColumns(updatedColumns);

                  form.setFieldsValue({
                    [props.id]: value
                  })
                }}>
                  {datasets.map((dataset) => (
                    <Select.Option key={dataset.datasetName} value={dataset.datasetName}>
                      {dataset.datasetName}
                    </Select.Option>
                  ))}
                </Select>
              },
            }
          }
        }}
      >
        <Col className="argument-form__header" style={{ display: 'none' }}>
          <ProFormSelect
            label="Layout"
            labelCol={{ span: 8 }}
            wrapperCol={{ span: 16 }}
            options={['ModalForm', 'QueryFilter']}
            fieldProps={{
              value: layoutType,
              onChange: (e) => setLayoutType(e),
            }}
          />
          <Space className="btn-group" style={{ display: 'none' }}>
            <Tooltip title={"Import argument file"}>
              <Button disabled icon={<UploadOutlined />}>
                Import
              </Button>
            </Tooltip>
            <Tooltip title={"Export all arguments as a file"}>
              <Button disabled icon={<DownloadOutlined />}>
                Export
              </Button>
            </Tooltip>
          </Space>
        </Col>
        {/* More details on https://procomponents.ant.design/components/schema/#%E8%87%AA%E5%AE%9A%E4%B9%89-valuetype */}
        {/* @ts-ignore */}
        <BetaSchemaForm<DataItem>
          className="schema-form vertical"
          trigger={activateBtn}
          style={{ height }}
          disabled={props.readonly}
          span={labelSpan}
          form={form}
          defaultCollapsed={false}
          layoutType={layoutType}
          layout="vertical"
          onFinish={async (values) => {
            if (onSubmit) {
              onSubmit(values)
                .then((response) => {
                  console.log('onSubmit ArgumentForm: ', response);
                })
                .catch((error) => {
                  console.log('onSubmit ArgumentForm Error: ', error);
                });
            }
          }}
          columns={columns}
        />
      </ProProvider.Provider>
    </Row>
  ) : (
    <Empty description="No arguments" />
  );
};

export default memo(ArgumentForm);
