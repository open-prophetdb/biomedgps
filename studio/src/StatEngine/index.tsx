import type { ProFormColumnsType } from '@ant-design/pro-form';
import { GridContent } from '@ant-design/pro-layout';
import { Col, message, Row, Spin, Tabs, Form, Empty, Select, Input, Button, Popover, Descriptions } from 'antd';

import {
  CheckCircleOutlined,
  InfoCircleOutlined,
  QuestionCircleFilled,
} from '@ant-design/icons';
import React, { useEffect, useState } from 'react';
import './index.less';

// Custom Component
import ArgumentForm from './components/ArgumentForm';
import MarkdownViewer from './components/MarkdownViewer';
import Resizer from './components/Resizer';
import ResultPanel from './components/ResultPanel';

// Custom DataType
import type { ChartResult, DataItem } from './components/WorkflowList/data';
import type { Workflow, TaskHistory } from './components/WorkflowList/data';

// Custom API
import {
  fetchTaskByTaskId,
  fetchWorkflowSchema,
  postTask,
} from '../services/swagger/KnowledgeGraph';

const { TabPane } = Tabs;

export type StatEngineProps = {
  workflowId: string;
  workflow: Workflow;
  task?: TaskHistory;
}

const StatEngine: React.FC<StatEngineProps> = (props) => {
  console.log('StatEngine Props: ', props);

  const [taskName, setTaskName] = useState<string>(props.task?.task_name || '');
  const [taskNameWarning, setTaskNameWarning] = useState<string | null>(null);
  const [taskDescription, setTaskDescription] = useState<string>(props.task?.description || '');
  const [workflowInfoVisible, setWorkflowInfoVisible] = useState<boolean>(false);
  const [taskStatus, setTaskStatus] = useState<'Running' | 'Succeeded' | 'Failed' | 'Unknown' | null>(null);

  const [leftSpan, setLeftSpan] = useState<number>(8);
  const [resizeBtnActive, setResizeBtnActive] = useState<boolean>(false);

  // Left Panel
  const [currentActiveKey, setCurrentActiveKey] = useState<string>('arguments');

  // Chart
  const [markdown, setMarkdown] = useState<string | null>(null);
  const [argumentColumns, setArgumentColumns] = useState<ProFormColumnsType<DataItem>[] & any>([]);
  const [fieldsValue, setFieldsValue] = useState<Record<string, any>>(props.task?.task_params || {});

  const [resultData, setResultData] = useState<ChartResult | undefined>({
    results: [],
    charts: [],
    task_id: '',
    log: '',
  });

  // Result
  const [resultLoading, setResultLoading] = useState<boolean>(false);

  // useEffect(() => {
  //   // More details on https://v3.umijs.org/docs/routing#routing-component-parameters
  //   const chart = props.chart;

  //   if (chart) {
  //     setCurrentChart(chart);
  //   } else {
  //     setCurrentChart('boxplot');
  //   }
  // }, [props.chart]);

  const setChart = (workflowId: string, fieldsValue?: Record<string, any>) => {
    fetchWorkflowSchema({ id: workflowId }).then((response) => {
      const schema = {
        ...response.schema,
      };

      // Reset README
      setMarkdown(schema.readme);

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

  useEffect(() => {
    if (props.task) {
      autoFetchTask(props.task.task_id || "");
    }
  }, [props.task]);

  const autoFetchTask = (taskId: string) => {
    const interval = setInterval(() => {
      if (taskId.length > 0) {
        fetchTaskByTaskId({ task_id: taskId })
          .then((resp) => {
            const { task, workflow } = resp;
            const results: { files: Record<string, any>[], charts: Record<string, any>[] } | null = task.results;

            if (task.status === 'Succeeded') {
              setResultData({
                results: results?.files.map((file) => file.filelink) || [],
                charts: results?.charts.map((chart) => chart.filelink) || [],
                log: task.log_message,
                task_id: task.task_id,
              });
              setTaskStatus('Succeeded');
              message.success('Load chart...');
              clearInterval(interval);
            } else if (task.status === 'Failed') {
              setResultData({
                results: results?.files.map((file) => file.filelink) || [],
                charts: results?.charts.map((chart) => chart.filelink) || [],
                log: task.log_message,
                task_id: task.task_id,
              });
              setTaskStatus('Failed');
              message.error('Something wrong, please check the log for more details.');
              clearInterval(interval);
            } else {
              setTaskStatus('Running');
            }
          })
          .catch((error) => {
            console.log('Get Task Error: ', error);
            clearInterval(interval);
            setTaskStatus('Unknown');
          });
      }
    }, 5000);
  };

  const onSubmit = (values: Pick<TaskHistory, 'task_params'>): Promise<TaskHistory> => {
    if (taskName.length === 0) {
      setTaskNameWarning("Please enter your task name.");
      return Promise.reject(new Error('Please enter your task name.'));
    }

    console.log('onSubmit Chart: ', values);
    values = {
      ...values,
    }

    // @ts-ignore, we don't need more fields for now
    const task: TaskHistory = {
      // TODO: Change to the real workspace id
      workspace_id: '00000000-0000-0000-0000-000000000000',
      workflow_id: props.workflowId,
      task_name: taskName,
      description: taskDescription,
      task_params: values,
      // Just a placeholder for avoiding boring TypeScript compiler
      owner: ''
    }

    return new Promise<TaskHistory>((resolve, reject) => {
      postTask(task, values)
        .then((response) => {
          console.log('Post Chart: ', response);
          message.success(`Create the ${taskName} successfully.`);
          setResultLoading(true);
          autoFetchTask(response.task_id);
          resolve(response);
        })
        .catch((error) => {
          message.warning('Unknown error, please retry later.');
          console.log('Post Chart Error: ', error);
          reject(error);
        });
    });
  };

  const getRightSpan = (customLeftSpan: number): number => {
    return 24 - customLeftSpan ? 24 - customLeftSpan : 24;
  };

  useEffect(() => {
    if (props.workflowId) {
      setChart(props.workflowId, fieldsValue);
    }
  }, [props.workflowId]);

  return (
    <GridContent>
      <Row className="stat-engine-header">
        <Form.Item validateStatus={taskNameWarning ? 'error' : ''} help={taskNameWarning} style={{ width: '40%', marginRight: '10px' }}>
          <Input placeholder='Enter Your Task Name' value={taskName} onChange={(e) => {
            setTaskNameWarning(null);
            setTaskName(e.target.value)
          }} allowClear
            disabled={props.task !== undefined} size='large' />
        </Form.Item>
        <Form.Item style={{ width: 'calc(60% - 100px)' }}>
          <Input placeholder='Enter Your Task Description' value={taskDescription} onChange={(e) => setTaskDescription(e.target.value)} allowClear
            disabled={props.task !== undefined} size='large' />
        </Form.Item>
        <Popover content={
          <Descriptions title="Workflow Summary" column={2} bordered>
            <Descriptions.Item label="Name">{props.workflow.name}</Descriptions.Item>
            <Descriptions.Item label="Short Name">{props.workflow.short_name}</Descriptions.Item>
            <Descriptions.Item label="Category">{props.workflow.category}</Descriptions.Item>
            <Descriptions.Item label="Author">{props.workflow.author}</Descriptions.Item>
            <Descriptions.Item label="Maintainers">{props.workflow.maintainers}</Descriptions.Item>
            <Descriptions.Item label="Tags">{props.workflow.tags}</Descriptions.Item>
            <Descriptions.Item label="Version">{props.workflow.version}</Descriptions.Item>
            <Descriptions.Item label="Created Time">{props.workflow.source}</Descriptions.Item>
            <Descriptions.Item label="Description" span={2}>{props.workflow.description}</Descriptions.Item>
          </Descriptions>
        } open={workflowInfoVisible} onOpenChange={setWorkflowInfoVisible} mouseEnterDelay={0.5} trigger='click'>
          <Button icon={<QuestionCircleFilled />} size='large' shape='default' style={{ marginTop: '10px' }} />
        </Popover>
      </Row>
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
                        Arguments
                      </span>
                    }
                    key="arguments"
                  >
                    <ArgumentForm
                      readonly={props.task !== undefined}
                      contextData={{}}
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
                        Help Document
                      </span>
                    }
                    key="help"
                  >
                    <MarkdownViewer markdownContent={markdown || 'No help document available.'} />
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
                taskStatus={taskStatus}
                workflow={props.workflow}
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
