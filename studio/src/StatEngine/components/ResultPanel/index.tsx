import {
  BarChartOutlined,
  EditOutlined,
  FullscreenExitOutlined,
  HistoryOutlined,
  IssuesCloseOutlined,
  // SnippetsOutlined,
  DatabaseOutlined
} from '@ant-design/icons';
import { Button, Col, Drawer, Empty, Row, Space, Tabs, Tooltip, message, Badge } from 'antd';
import React, { memo, useEffect, useState, useRef } from 'react';

import WorkflowList from '../WorkflowList';
import LogViewer from '../LogViewer/indexLog';
// import MarkdownViewer from '../MarkdownViewer';
import PlotlyViewer from 'biominer-components/dist/PlotlyViewer/indexClass';
import HistoryTable from '../HistoryTable';
import { AgGridReact } from 'ag-grid-react';
import { JsonViewer } from '@textea/json-viewer';
import { CSVLink } from "react-csv";
// @ts-ignore
import Papa from 'papaparse';
import type { ChartResult } from '../WorkflowList/data';
import type { PlotlyChart } from 'biominer-components/dist/PlotlyViewer/data';
import { fetchFileByFileName } from '../../../services/swagger/KnowledgeGraph';
import type { Workflow, TaskHistory, FileMeta } from '../WorkflowList/data';

// AG Grid theme
import 'ag-grid-enterprise';
import 'ag-grid-community/styles/ag-grid.css';
import 'ag-grid-community/styles/ag-theme-quartz.css';

import './index.less';

const { TabPane } = Tabs;

export type ResultPanelProps = {
  onClickItem: (workflowName: string, result?: ChartResult, fieldsValue?: Record<string, any>) => void;
  task?: TaskHistory;
  logMessage: string;
  files: FileMeta[];
  charts: FileMeta[];
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
  const { onClickItem, logMessage, responsiveKey, task, files, charts } = props;

  const [chartTask, setChartTask] = useState<TaskHistory | undefined>(undefined);
  const [plotlyEditorMode, setPlotlyEditorMode] = useState<string>('Plotly');
  const [chartsVisible, setChartsVisible] = useState<boolean>(false);
  const [editBtnActive, setEditBtnActive] = useState<boolean>(false);
  const [historyVisible, setHistoryVisible] = useState<boolean>(false);
  const [activeKey, setActiveKey] = useState<string>("chart");

  const [plotData, setPlotData] = useState<any | null>(null);
  const [columnDefs, setColumnDefs] = useState<any | null>(null);
  const [plotlyData, setPlotlyData] = useState<PlotlyChart | null>(null);

  const [taskDuration, setTaskDuration] = useState<string>('0s');
  const intervalId = useRef<NodeJS.Timeout | null>(null);

  useEffect(() => {
    if (charts.length > 0 && task) {
      console.log('Chart Task: ', task.task_id);
      fetchFileByFileName({
        task_id: task.task_id,
        file_name: charts[0].filename
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
    if (files.length > 0 && task) {
      console.log('Data: ', task.task_id);
      const filetype = files[0].filename && files[0].filename.split('.')[1];

      fetchFileByFileName({
        task_id: task.task_id,
        file_name: files[0].filename
      }).then((response: any) => {
        console.log('File Data: ', response, response.length);
        let trimmedResponse = response.trim();
        if (!trimmedResponse || trimmedResponse.length === 0) {
          setPlotData(null);
          return;
        } else {
          Papa.parse(trimmedResponse, {
            header: true,
            delimiter: filetype === 'tsv' ? '\t' : ',',
            skipEmptyLines: true,
            dynamicTyping: true,
            complete: function (results: any) {
              const parsedData = results.data;
              setPlotData(parsedData)

              if (parsedData.length > 0) {
                const firstRow = parsedData[0];
                const columns = Object.keys(firstRow).map((key) => {
                  return {
                    headerName: key,
                    field: key,
                    sortable: true,
                    filter: true
                  }
                });

                setColumnDefs(columns);
              }
            },
            error: function (error: any) {
              message.warning("Cannot parse the result, the data may not be a valid CSV/TSV file.")
              setPlotData(null)
            }
          });
        }
      }).catch(error => {
        message.warning("Cannot fetch the result, please retry later.")
        setPlotData(null)
      });
    }
  }, [files])

  useEffect(() => {
    if (logMessage.length > 0) {
      setEditBtnActive(true);
    } else {
      setEditBtnActive(false);
    }
  }, [logMessage]);

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

  const computeTaskDuration = (task?: TaskHistory) => {
    let duration = 0;

    if (task) {
      const finishedTime = task.finished_time ? new Date(task.finished_time).getTime() : new Date().getTime();
      const startedTime = new Date(task.submitted_time).getTime();

      duration = finishedTime - startedTime;
    }

    const durationInSeconds = duration / 1000;

    if (durationInSeconds <= 0) {
      return '0s';
    }

    if (durationInSeconds < 60) {
      return `${Math.round(durationInSeconds)}s`;
    } else if (durationInSeconds < 3600) {
      const minutes = Math.floor(durationInSeconds / 60);
      const seconds = Math.round(durationInSeconds % 60);
      return seconds > 0 ? `${minutes}min ${seconds}s` : `${minutes}min`;
    } else {
      const hours = Math.floor(durationInSeconds / 3600);
      const minutes = Math.floor((durationInSeconds % 3600) / 60);
      return minutes > 0 ? `${hours}h ${minutes}min` : `${hours}h`;
    }
  };

  useEffect(() => {
    if (task?.status === 'Succeeded' || task?.status === 'Failed' || task?.finished_time) {
      if (intervalId.current) {
        clearInterval(intervalId.current);
        intervalId.current = null;
      }

      setTaskDuration(computeTaskDuration(task));
      return;
    }

    if (intervalId.current) {
      clearInterval(intervalId.current);
      intervalId.current = null;
    }

    if (!intervalId.current && (props.taskStatus !== 'Succeeded' && props.taskStatus !== 'Failed')) {
      intervalId.current = setInterval(() => {
        setTaskDuration(computeTaskDuration(task));
      }, 1000);
    }

    return () => {
      if (intervalId.current) {
        clearInterval(intervalId.current);
        intervalId.current = null;
      }
    };
  }, [props.taskStatus]);

  const resultOperations = (
    <Space>
      {`Duration: ${taskDuration}`}
      {
        props.taskStatus !== null ?
          <Tooltip title='Update status automatically'>
            <Button>
              <Badge status={formatTaskStatus(props.taskStatus)} text={props.taskStatus} />
            </Button>
          </Tooltip> : null
      }
      {
        /* <Tooltip title="Edit the Chart">
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
        </Tooltip> */
      }
      {
        /* <Tooltip title="List all charts">
          <Button
            style={{ display: 'none' }}
            onClick={() => {
              setChartsVisible(true);
            }}
            icon={<BarChartOutlined />}
          >
            Charts
          </Button>
        </Tooltip> */
      }
      {
        /* <Tooltip title="List all history">
          <Button
            onClick={() => {
              setHistoryVisible(true);
            }}
            icon={<HistoryOutlined />}
          >
            History
          </Button>
        </Tooltip> */
      }
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
            className={`result-container ${plotlyEditorMode === 'PlotlyEditor' ? 'full-screen' : 'no-full-screen'}`}
          >
            {
              plotlyEditorMode === 'PlotlyEditor' ? (
                <Button
                  className="exit-editor"
                  onClick={() => {
                    setPlotlyEditorMode('Plotly');
                  }}
                >
                  <FullscreenExitOutlined />
                  Exit Editor
                </Button>
              ) : null
            }
            <PlotlyViewer
              responsiveKey={responsiveKey}
              plotlyData={plotlyData}
              key={charts.length > 0 ? charts[0].filename : 'random-string'}
              mode={plotlyEditorMode}
            />
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
          <LogViewer logMessage={logMessage} height="calc(100vh - 280px)" />
        </TabPane>
        {
          <TabPane
            tab={
              <span>
                <DatabaseOutlined />
                Data
              </span>
            }
            key="data"
          >
            {
              plotData ?
                <div className={'ag-theme-quartz'}>
                  <AgGridReact
                    rowData={plotData}
                    columnDefs={columnDefs}
                    rowSelection={'multiple'}
                    defaultColDef={{
                      flex: 1,
                      minWidth: 100,
                      resizable: true,
                      editable: false
                    }}
                    enableAdvancedFilter={true}
                    groupSelectsChildren={true}
                    rowGroupPanelShow={'always'}
                    suppressRowClickSelection={true}
                    sideBar={false}
                    groupAllowUnbalanced
                    enableCellTextSelection={true}
                    enableBrowserTooltips={true}
                    rowMultiSelectWithClick={true}
                    statusBar={{
                      statusPanels: [
                        { statusPanel: 'agTotalAndFilteredRowCountComponent', align: 'left' },
                        { statusPanel: 'agTotalRowCountComponent', align: 'center' },
                        { statusPanel: 'agFilteredRowCountComponent' },
                        { statusPanel: 'agSelectedRowCountComponent' },
                        { statusPanel: 'agAggregationComponent' },
                      ],
                    }}
                    // onGridReady={onGridReady}
                    // It seems that the row selection checkbox also works well at the row group mode.
                    // onColumnRowGroupChanged={onColumnRowGroupChanged}
                    onSelectionChanged={() => { }}
                    autoSizeStrategy={{
                      type: 'fitCellContents'
                    }}
                    // pagination={true}
                    // paginationPageSize={30}
                    getContextMenuItems={(params: any) => {
                      var result = [
                        'copy',
                        'copyWithHeaders',
                        'copyWithGroupHeaders',
                        'separator',
                        'autoSizeAll',
                        'resetColumns',
                        'expandAll',
                        'contractAll',
                        'separator',
                        'export',
                      ];
                      return result;
                    }}
                    domLayout="autoHeight"
                  />
                </div> :
                <Empty description="File does not contain data." />
            }
          </TabPane>
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
