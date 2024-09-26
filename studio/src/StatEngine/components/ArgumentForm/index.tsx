import { DownloadOutlined, EditOutlined, UploadOutlined } from '@ant-design/icons';
import type { ProFormColumnsType, ProFormLayoutType } from '@ant-design/pro-form';
import { BetaSchemaForm, ProProvider, ProFormSelect } from '@ant-design/pro-components';
import { Button, Col, Empty, Row, Space, Tooltip, Form } from 'antd';
import GeneSearcher from '@/components/GeneSearcher';
import { GenesQueryParams, GeneDataResponse } from '@/components/GeneSearcher';
import FormItem from 'antd/lib/form/FormItem';
import React, { memo, useContext, useEffect, useState } from 'react';
import { useIntl } from 'umi';
import type { ChartResult } from './data';

import './index.less';

import type { DataItem } from './data';
import { langData } from './lang';

export type ArgumentProps = {
  queryGenes: (params: GenesQueryParams) => Promise<GeneDataResponse>;
  columns: ProFormColumnsType<DataItem>[];
  fieldsValue?: any;
  height?: string;
  labelSpan?: number;
  defaultDataset: string;
  onSubmit?: (values: any) => Promise<ChartResult>;
};

type UIContext = Record<string, any>;

const ArgumentForm: React.FC<ArgumentProps> = (props) => {
  const { columns, height, labelSpan, onSubmit, queryGenes, fieldsValue, defaultDataset } = props;

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

  const intl = useIntl();

  const uiContext: UIContext = {};
  Object.keys(langData).forEach((key) => {
    uiContext[key] = intl.formatMessage(langData[key]);
  });

  useEffect(() => {
    form.resetFields()
  }, [columns])

  useEffect(() => {
    if (fieldsValue) {
      form.setFieldsValue(fieldsValue)
    }
  }, [fieldsValue])

  console.log('ArgumentForm updated');

  const values = useContext(ProProvider);
  return columns.length > 0 ? (
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
                  placeholder="Enter gene symbol, entrez id or ensembl id"
                  dataset={defaultDataset}
                  queryGenes={queryGenes}
                  initialValue={initialValue ? initialValue : props?.formItemProps?.initialValue}
                  {...props?.fieldProps}
                  mode={props?.fieldProps?.mode}
                  style={{ width: '100%' }} />)
              },
            }
          },
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
            <Tooltip title={uiContext.importTooltip}>
              <Button disabled icon={<UploadOutlined />}>
                {`${uiContext.import}`}
              </Button>
            </Tooltip>
            <Tooltip title={uiContext.exportTooltip}>
              <Button disabled icon={<DownloadOutlined />}>
                {`${uiContext.export}`}
              </Button>
            </Tooltip>
          </Space>
        </Col>
        {/* More details on https://procomponents.ant.design/components/schema/#%E8%87%AA%E5%AE%9A%E4%B9%89-valuetype */}
        <BetaSchemaForm<DataItem>
          className="schema-form vertical"
          trigger={activateBtn}
          style={{ height }}
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
    <Empty />
  );
};

export default memo(ArgumentForm);
