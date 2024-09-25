import {
  BarChartOutlined,
  EditOutlined,
  FullscreenExitOutlined,
  HistoryOutlined,
  IssuesCloseOutlined,
  // SnippetsOutlined,
  DatabaseOutlined
} from '@ant-design/icons';
import { FormattedMessage } from 'umi';
import { Button, Col, Drawer, Row, Space, Tabs, Tooltip, message } from 'antd';
import React, { memo, useEffect, useState } from 'react';
import { useIntl } from 'umi';

import ChartList from '../ChartList';
import LogViewer from '@/components/LogViewer/indexLog';
// import MarkdownViewer from '../MarkdownViewer';
import PlotlyViewer from '@/components/PlotlyViewer/indexClass';
import HistoryTable from '../HistoryTable';
import { TaskListItem } from '../HistoryTable';
import { JsonViewer } from '@textea/json-viewer';
import { CSVLink } from "react-csv";

import { getDownload as getFile } from '@/services/swagger/Instance';
import type { ChartResult } from '../ChartList/data';
import type { PlotlyChart } from '@/components/PlotlyViewer/data';

import './index.less';
import { langData } from './lang';

const { TabPane } = Tabs;

export type ResultPanelProps = {
  onClickItem: (chart: string, result?: ChartResult, fieldsValue?: Record<string, any>) => void;
  taskId: string;
  logLink: string;
  results: string[];
  charts: string[];
  currentChart: string | null;
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

type UIContext = Record<string, any>;

const ResultPanel: React.FC<ResultPanelProps> = (props) => {
  const intl = useIntl();

  const uiContext: UIContext = {};
  Object.keys(langData).forEach((key) => {
    uiContext[key] = intl.formatMessage(langData[key]);
  });

  const { onClickItem, logLink, responsiveKey, currentChart, taskId, results, charts } = props;

  const [chartTask, setChartTask] = useState<TaskListItem | undefined>(undefined);
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
      getFile({ filelink: charts[0] }).then((response: any) => {
        setPlotlyData({
          data: response.data,
          layout: response.layout,
          frames: response.frames || undefined
        });
      }).catch(error => {
        message.warn("Cannot fetch the result, please retry later.")
      });
    }
  }, [charts]);

  useEffect(() => {
    if (results.length > 0) {
      console.log('Data: ', taskId);
      getFile({ filelink: results[0] }).then((response: any) => {
        setPlotData(response)
      }).catch(error => {
        message.warn("Cannot fetch the result, please retry later.")
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

  const resultOperations = (
    <Space>
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
          {uiContext.edit}
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
          {uiContext.charts}
        </Button>
      </Tooltip>
      <Tooltip title="List all history">
        <Button
          onClick={() => {
            setHistoryVisible(true);
          }}
          icon={<HistoryOutlined />}
        >
          {uiContext.history}
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
              {uiContext.figure}
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
              {uiContext.log}
            </span>
          }
          key="log"
        >
          <LogViewer getFile={getFile} height="calc(100vh - 200px)" url={logLink} />
        </TabPane>
        {
          plotData ? (<TabPane
            tab={
              <span>
                <DatabaseOutlined />
                {uiContext.data}
              </span>
            }
            key="data"
          >
            <CSVLink data={plotData}
              filename="data.csv"
              className="button">
              {uiContext.download} {uiContext.data}
            </CSVLink>
            <JsonViewer value={plotData} />
          </TabPane>) : null}
        {chartTask ? (<TabPane
          tab={
            <span>
              <IssuesCloseOutlined />
              {uiContext.metadata}
            </span>
          }
          key="metadata"
        >
          <a className="button" onClick={() => { downloadAsJSON(chartTask, "download-anchor") }}>
            {uiContext.download} {uiContext.metadata}
          </a>
          <a id="download-anchor" style={{ display: 'none' }}></a>
          <JsonViewer value={chartTask} />
        </TabPane>) : null}
      </Tabs>
      <Drawer
        title="Chart Store"
        placement="right"
        closable
        width="70%"
        onClose={() => {
          setChartsVisible(false);
        }}
        visible={chartsVisible}
      >
        <ChartList
          onClickItem={(chart, result, fieldsValue) => {
            onClickItem(chart.short_name, result, fieldsValue);
            setChartsVisible(false);
          }}
        ></ChartList>
      </Drawer>

      <Drawer
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
      </Drawer>
    </Row>
  );
};

export default memo(ResultPanel);
