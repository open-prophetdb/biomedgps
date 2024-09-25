import type { ProFormColumnsType } from '@ant-design/pro-form';
import { GridContent } from '@ant-design/pro-layout';
import { Col, message, Row, Spin, Tabs, Form, Empty, Select } from 'antd';
import type { StaticContext } from 'react-router';
import type { RouteComponentProps } from 'react-router-dom';
import { useIntl, useModel } from 'umi';

import {
  CheckCircleOutlined,
  InfoCircleOutlined,
} from '@ant-design/icons';
import React, { useEffect, useState } from 'react';
import './index.less';

// Custom Component
import ArgumentForm from '@/components/ArgumentForm';
import MarkdownViewer from '@/components/MarkdownViewer';
import Resizer from '@/components/Resizer';
import ResultPanel from './components/ResultPanel';

// Custom DataType
import type { ChartResult, DataItem } from './components/ChartList/data';
type UIContext = Record<string, any>;

// Custom API
import {
  getTasksId as getChartTask,
  getChartsUiSchemaChartName as getChartUiSchema,
  postChartChartName as postChart
} from '@/services/swagger/StatEngine';
import { GenesQueryParams, GeneDataResponse } from '@/components/GeneSearcher';
import { getDownload as getFile } from '@/services/swagger/Instance';

// Custom Data
import { langData } from './lang';

const { TabPane } = Tabs;

export type StatEngineProps = {
  queryGenes: (params: GenesQueryParams) => Promise<GeneDataResponse>;
}

const StatEngine: React.FC<StatEngineProps & RouteComponentProps<{}, StaticContext>> = (props) => {
  const { queryGenes } = props;
  const intl = useIntl();
  console.log('StatEngine Props: ', props);

  const uiContext: UIContext = {};
  Object.keys(langData).forEach((key) => {
    uiContext[key] = intl.formatMessage(langData[key]);
  });

  const [leftSpan, setLeftSpan] = useState<number>(8);
  const [resizeBtnActive, setResizeBtnActive] = useState<boolean>(false);

  // Left Panel
  const [currentActiveKey, setCurrentActiveKey] = useState<string>('arguments');

  // Chart
  const { defaultDataset } = useModel('dataset', (ret) => ({
    defaultDataset: ret.defaultDataset,
    setDataset: ret.setDataset,
  }));
  const [currentChart, setCurrentChart] = useState<string | null>('');
  const [markdownLink, setMarkdownLink] = useState<string>('');
  const [argumentColumns, setArgumentColumns] = useState<ProFormColumnsType<DataItem>[] & any>([]);
  const [fieldsValue, setFieldsValue] = useState<any>({});

  const [form] = Form.useForm();
  useEffect(() => {
    form.setFieldsValue({
      dataset: defaultDataset
    })
  }, [defaultDataset])

  const [resultData, setResultData] = useState<ChartResult | undefined>({
    results: [],
    charts: [],
    task_id: '',
    log: '',
  });

  // Result
  const [resultLoading, setResultLoading] = useState<boolean>(false);

  useEffect(() => {
    // More details on https://v3.umijs.org/docs/routing#routing-component-parameters
    const chart = props.chart;
    if (chart) {
      setCurrentChart(chart);
    } else {
      setCurrentChart('boxplot');
    }
  }, [props.chart]);

  const setChart = (dataset: string, chart: string, fieldsValue?: Record<string, any>) => {
    getChartUiSchema({ chart_name: chart, dataset: dataset }).then((response) => {
      const schema = {
        ...response.schema,
      };

      // Reset README
      setMarkdownLink(`${response.readme}?=${(Math.random() + 1).toString(36).substring(7)}`);

      // Reset Argument
      setArgumentColumns(schema.fields);

      if (fieldsValue) {
        // Reset Fields Value
        setFieldsValue(fieldsValue);
      }
    });
  };

  const restoreChart = (chart: string, result?: ChartResult, fieldsValue?: Record<string, any>) => {
    console.log("Restore Chart: ", chart, result, fieldsValue);
    if (fieldsValue) {
      setFieldsValue(fieldsValue);
      setCurrentChart(chart);
    }

    if (result) {
      setResultData(result);
    } else {
      setResultData(undefined);
    }
  }

  const changeDataTab = (key: string) => {
    setCurrentActiveKey(key);
  };

  // For debug
  // useEffect(() => {
  //   autoFetchTask("f318ff50-4ad3-11ed-a5b3-c6aea7bb5ffb")
  // }, true)

  const autoFetchTask = (taskId: string) => {
    const interval = setInterval(() => {
      if (taskId.length > 0) {
        getChartTask({ id: taskId })
          .then((resp) => {
            if (resp.status === 'Finished') {
              setResultData({
                results: resp.response.results,
                charts: resp.response.charts,
                log: resp.response.log,
                task_id: resp.response.task_id,
              });
              setResultLoading(false);
              message.success('Load chart...');
              clearInterval(interval);
            } else if (resp.status === 'Failed') {
              setResultData({
                results: resp.response.results,
                charts: resp.response.charts,
                log: resp.response.log,
                task_id: resp.response.task_id,
              });
              setResultLoading(false);
              message.error('Something wrong, please check the log for more details.');
              clearInterval(interval);
            }
          })
          .catch((error) => {
            console.log('Get Task Error: ', error);
            clearInterval(interval);
          });
      }
    }, 1000);
  };

  const onSubmit = (values: any) => {
    const chartName: string = currentChart || '';
    console.log('onSubmit Chart: ', currentChart, values);
    values = {
      ...values,
      dataset: defaultDataset
    }
    return new Promise<{ task_id: string }>((resolve, reject) => {
      postChart({ chart_name: chartName }, values)
        .then((response) => {
          console.log('Post Chart: ', response);
          message.success(`Create the chart ${chartName} successfully.`);
          setResultLoading(true);
          autoFetchTask(response.task_id);
          resolve(response);
        })
        .catch((error) => {
          message.warn('Unknown error, please retry later.');
          console.log('Post Chart Error: ', error);
          reject(error);
        });
    });
  };

  const getRightSpan = (customLeftSpan: number): number => {
    return 24 - customLeftSpan ? 24 - customLeftSpan : 24;
  };

  useEffect(() => {
    if (currentChart && defaultDataset) {
      setChart(defaultDataset, currentChart, fieldsValue);
    }
  }, [currentChart, defaultDataset]);

  return (
    <GridContent>
      <Spin spinning={resultLoading} style={{ marginTop: '50px' }}>
        <Row className="stat-engine" gutter={8}>
          <Col className="left" xxl={leftSpan} xl={leftSpan} lg={leftSpan} md={24} sm={24} xs={24}>
            <Row className="left__content">
              <Col className="left__tabs">
                <Tabs
                  onChange={(key) => {
                    changeDataTab(key);
                  }}
                  activeKey={currentActiveKey}
                  defaultActiveKey="arguments"
                  className="left__tabs__arguments"
                >
                  <TabPane
                    tab={
                      <span>
                        <CheckCircleOutlined />
                        {uiContext.arguments}
                      </span>
                    }
                    key="arguments"
                  >
                    <Form layout={"vertical"} form={form}>
                      <Form.Item
                        shouldUpdate
                        label="Data Set"
                        name="dataset"
                        initialValue={defaultDataset}
                        rules={[{ required: true, message: 'Please select a dataset!' }]}
                      >
                        <Select
                          allowClear
                          showSearch
                          placeholder={"Select a dataset"}
                          defaultActiveFirstOption={false}
                          showArrow={true}
                          filterOption={false}
                          options={
                            [
                              {
                                value: `${defaultDataset}`,
                                label: `${defaultDataset}`,
                              },
                            ]
                          }
                          disabled
                          notFoundContent={<Empty description="No Dataset" />}
                        >
                        </Select>
                      </Form.Item>
                    </Form>
                    <ArgumentForm
                      defaultDataset={defaultDataset}
                      queryGenes={queryGenes}
                      fieldsValue={fieldsValue}
                      labelSpan={24}
                      height="calc(100% - 10px)"
                      onSubmit={onSubmit}
                      columns={argumentColumns}
                    ></ArgumentForm>
                  </TabPane>
                  <TabPane
                    tab={
                      <span>
                        <InfoCircleOutlined />
                        {uiContext.summary}
                      </span>
                    }
                    key="summary"
                  >
                    <MarkdownViewer getFile={getFile} url={markdownLink} />
                  </TabPane>
                </Tabs>
              </Col>
              <Resizer
                className="left__divider"
                HoverHandler={setResizeBtnActive}
                ClickHandler={setLeftSpan}
                btnActive={resizeBtnActive}
              ></Resizer>
            </Row>
          </Col>
          <Col
            className="right"
            xxl={getRightSpan(leftSpan)}
            xl={getRightSpan(leftSpan)}
            lg={getRightSpan(leftSpan)}
            md={24}
            sm={24}
            xs={24}
          >
            <Row className="right__content">
              <ResultPanel
                currentChart={currentChart}
                results={resultData?.results || []}
                charts={resultData?.charts || []}
                taskId={resultData?.task_id || ''}
                responsiveKey={leftSpan}
                logLink={resultData?.log || ''}
                onClickItem={restoreChart}
              ></ResultPanel>
            </Row>
          </Col>
        </Row>
      </Spin>
    </GridContent>
  );
};

export default StatEngine;
