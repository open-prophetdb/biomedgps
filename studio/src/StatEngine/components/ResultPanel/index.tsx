import {
  BarChartOutlined,
  EditOutlined,
  FullscreenExitOutlined,
  HistoryOutlined,
  IssuesCloseOutlined,
  // SnippetsOutlined,
  DatabaseOutlined
} from '@ant-design/icons';
import { Button, Col, Drawer, Row, Space, Tabs, Tooltip, message, Badge } from 'antd';
import React, { memo, useEffect, useState } from 'react';

import WorkflowList from '../WorkflowList';
import LogViewer from '../LogViewer/indexLog';
// import MarkdownViewer from '../MarkdownViewer';
import PlotlyViewer from 'biominer-components/dist/PlotlyViewer/indexClass';
import HistoryTable from '../HistoryTable';
import { JsonViewer } from '@textea/json-viewer';
import { CSVLink } from "react-csv";

import type { ChartResult } from '../WorkflowList/data';
import type { PlotlyChart } from 'biominer-components/dist/PlotlyViewer/data';
import { fetchFileByFileName } from '../../../services/swagger/KnowledgeGraph';
import type { Workflow, TaskHistory } from '../WorkflowList/data';

import './index.less';

const { TabPane } = Tabs;

export type ResultPanelProps = {
  onClickItem: (workflowName: string, result?: ChartResult, fieldsValue?: Record<string, any>) => void;
  taskId: string;
  logLink: string;
  results: string[];
  charts: string[];
  workflow: Workflow;
  taskStatus: 'Succeeded' | 'Running' | 'Failed' | 'Unknown' | null;
  responsiveKey: number | string;
};

export const downloadAsJSON = function (data: any, elementId: string) {
  var dataStr = 'data:text/json;charset=utf-8,' + encodeURIComponent(JSON.stringify(data))
  var dlAnchorElem = document.getElementById(elementId)
  if (dlAnchorElem) {
    dlAnchorElem.setAttribute('href', dataStr)
    dlAnchorElem.setAttribute('download', 'metadata.json')
    dlAnchorElem.click()
  } else {
    console.log(`No such html tag ${elementId}`)
  }

  console.log(`Download ${elementId}`)
}

const ResultPanel: React.FC<ResultPanelProps> = (props) => {
  const { onClickItem, logLink, responsiveKey, taskId, results, charts } = props;

  const [chartTask, setChartTask] = useState<TaskHistory | undefined>(undefined);
  const [plotlyEditorMode, setPlotlyEditorMode] = useState<string>('Plotly');
  const [chartsVisible, setChartsVisible] = useState<boolean>(false);
  const [editBtnActive, setEditBtnActive] = useState<boolean>(false);
  const [historyVisible, setHistoryVisible] = useState<boolean>(false);
  const [activeKey, setActiveKey] = useState<string>("chart");

  const [plotData, setPlotData] = useState<any | null>(null);
  const [plotlyData, setPlotlyData] = useState<PlotlyChart | null>(null);

  useEffect(() => {
    if (charts.length > 0) {
      console.log('Chart Task: ', taskId);
      fetchFileByFileName({
        task_id: taskId,
        file_name: charts[0]
      }).then((response: any) => {
        setPlotlyData({
          data: response.data,
          layout: response.layout,
          frames: response.frames || undefined
        });
      }).catch(error => {
        message.warning("Cannot fetch the result, please retry later.")
      });
    }
  }, [charts]);

  useEffect(() => {
    if (results.length > 0) {
      console.log('Data: ', taskId);
      fetchFileByFileName({
        task_id: taskId,
        file_name: results[0]
      }).then((response: any) => {
        setPlotData(response)
      }).catch(error => {
        message.warning("Cannot fetch the result, please retry later.")
      });
    }
  }, [results])

  useEffect(() => {
    if (logLink.length > 0) {
      setEditBtnActive(true);
    } else {
      setEditBtnActive(false);
    }
  }, [logLink]);

  const formatTaskStatus = (status: string | null) => {
    switch (status) {
      case 'Running':
        return 'processing';
      case 'Succeeded':
        return 'success';
      case 'Failed':
        return 'error';
      case 'Unknown':
        return 'warning';
      default:
        return 'default';
    }
  }

  const resultOperations = (
    <Space>
      {
        props.taskStatus !== null ?
          <Tooltip title='Update status automatically'>
            <Button>
              <Badge status={formatTaskStatus(props.taskStatus)} text={props.taskStatus} />
            </Button>
          </Tooltip> : null
      }
      <Tooltip title="Edit the Chart">
        <Button
          disabled={!editBtnActive}
          style={activeKey === 'chart' ? {} : { display: 'none' }}
          type="primary"
          icon={<EditOutlined />}
          onClick={() => {
            setPlotlyEditorMode('PlotlyEditor');
          }}
        >
          Edit
        </Button>
      </Tooltip>
      <Tooltip title="List all charts">
        <Button
          style={{ display: 'none' }}
          onClick={() => {
            setChartsVisible(true);
          }}
          icon={<BarChartOutlined />}
        >
          Charts
        </Button>
      </Tooltip>
      <Tooltip title="List all history">
        <Button
          onClick={() => {
            setHistoryVisible(true);
          }}
          icon={<HistoryOutlined />}
        >
          History
        </Button>
      </Tooltip>
    </Space>
  );

  console.log('ResultPanel updated');

  return (
    <Row className="result-panel">
      <Tabs
        onChange={(activeKey) => { setActiveKey(activeKey) }}
        activeKey={activeKey}
        className="tabs-result"
        tabBarExtraContent={resultOperations}>
        <TabPane
          tab={
            <span>
              <BarChartOutlined />
              Figure
            </span>
          }
          key="chart"
        >
          <Col
            id="graph-container"
            className={`result-container
        ${plotlyEditorMode === 'PlotlyEditor' ? 'full-screen' : 'no-full-screen'}`}
          >
            {plotlyEditorMode === 'PlotlyEditor' ? (
              <Button
                className="exit-editor"
                onClick={() => {
                  setPlotlyEditorMode('Plotly');
                }}
              >
                <FullscreenExitOutlined />
                Exit Editor
              </Button>
            ) : null}
            <PlotlyViewer
              responsiveKey={responsiveKey}
              plotlyData={plotlyData}
              key={charts[0]}
              mode={plotlyEditorMode}
            ></PlotlyViewer>
          </Col>
        </TabPane>
        <TabPane
          tab={
            <span>
              <IssuesCloseOutlined />
              Log
            </span>
          }
          key="log"
        >
          <LogViewer getFile={fetchFileByFileName} height="calc(100vh - 200px)" taskId={taskId} url={logLink} />
        </TabPane>
        {
          plotData ? <TabPane
            tab={
              <span>
                <DatabaseOutlined />
                Data
              </span>
            }
            key="data"
          >
            <CSVLink data={plotData}
              filename="data.csv"
              className="button">
              Download Data
            </CSVLink>
            <JsonViewer value={plotData} />
          </TabPane> : null
        }
        {
          chartTask ?
            <TabPane
              tab={
                <span>
                  <IssuesCloseOutlined />
                  Metadata
                </span>
              }
              key="metadata"
            >
              <a className="button" onClick={() => { downloadAsJSON(chartTask, "download-anchor") }}>
                Download Metadata
              </a>
              <a id="download-anchor" style={{ display: 'none' }}></a>
              <JsonViewer value={chartTask} />
            </TabPane> :
            null
        }
      </Tabs>
      <Drawer
        title="Chart Store"
        placement="right"
        closable
        width="70%"
        onClose={() => {
          setChartsVisible(false);
        }}
        open={chartsVisible}
      >
        <WorkflowList
          onClickItem={(workflow: Workflow, fieldsValue?: Record<string, any>) => {
            onClickItem(workflow.short_name, undefined, fieldsValue);
            setChartsVisible(false);
          }}
        />
      </Drawer>

      {/* <Drawer
        title="Chart History"
        placement="right"
        closable
        className='history-table-drawer'
        width="70%"
        onClose={() => {
          setHistoryVisible(false);
        }}
        visible={historyVisible}
      >
        <HistoryTable
          forceUpdateKey={`${historyVisible}`}
          pluginName={currentChart || undefined}
          onClickItem={(chartName: any, result?: ChartResult, taskListItem?: TaskListItem) => {
            onClickItem(chartName, result, taskListItem ? taskListItem.payload : undefined);
            setHistoryVisible(false);
            setChartTask(taskListItem)
          }}
        ></HistoryTable>
      </Drawer> */}
    </Row>
  );
};

export default memo(ResultPanel);
