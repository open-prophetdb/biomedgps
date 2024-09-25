import { Drawer } from 'antd';
import React, { useState, useRef, useEffect } from 'react';
import { useIntl, FormattedMessage } from 'umi';
import { PageContainer } from '@ant-design/pro-layout';
import type { ProColumns, ActionType } from '@ant-design/pro-table';
import ProTable from '@ant-design/pro-table';
import type { ProDescriptionsItemProps } from '@ant-design/pro-descriptions';
import ProDescriptions from '@ant-design/pro-descriptions';
import { getTasks } from '@/services/swagger/StatEngine';
import type { SortOrder } from 'antd/es/table/interface';
import { ChartResult } from '../ChartList/data';
import './index.less';

type PageParams = {
  current?: number | undefined;
  pageSize?: number | undefined;
  status?: string;
};

export type TaskListItem = {
  response: {
    log?: string;
    results?: string[];
    charts?: string[];
    response_type?: string;
    task_id?: string;
  };
  description: string;
  finished_time: any;
  plugin_name: string;
  payload: Record<string, any>;
  name: string;
  plugin_type: string;
  percentage: number;
  status: string;
  id: string;
  started_time: number;
  plugin_version: string;
  owner: any;
};

export type HistoryTableProps = {
  onClickItem?: (chart: string, result?: ChartResult, task?: TaskListItem) => void;
  pluginName?: string;
  forceUpdateKey?: string;
};

type TaskListResponse = {
  total: number;
  page: number;
  page_size: number;
  data: TaskListItem[];
};

function formatResponse(response: TaskListResponse): Promise<Partial<TaskListResponse>> {
  return Promise.resolve({
    ...response,
    success: true,
  });
}

const TableList: React.FC<HistoryTableProps> = (props) => {
  const { onClickItem, pluginName, forceUpdateKey } = props;

  const [showDetail, setShowDetail] = useState<boolean>(false);
  const [, setForceUpdate] = useState<string>();

  useEffect(() => {
    setForceUpdate('');
  }, [forceUpdateKey])

  const actionRef = useRef<ActionType>();
  const [currentRow, setCurrentRow] = useState<TaskListItem>();
  // const [selectedRowsState, setSelectedRows] = useState<TaskListItem[]>([]);

  const listTasks = async (
    params: PageParams,
    sort: Record<string, SortOrder>,
    filter: Record<string, React.ReactText[] | null>,
  ) => {
    const queryParams = {
      page: params.current,
      pape_size: params.pageSize,
    }

    if (pluginName) {
      queryParams['plugin_name'] = pluginName
    }

    if (filter.status) {
      queryParams['status'] = filter.status
    }

    if (params.status) {
      queryParams['status'] = params.status
    }

    console.log("List Tasks: ", queryParams, params);

    return await getTasks(queryParams)
      .then((response) => {
        return formatResponse(response);
      })
      .catch((error) => {
        console.log('requestDEGs Error: ', error);
        return formatResponse({ total: 0, page: 1, page_size: 10, data: [] });
      });
  }

  /**
   * @en-US International configuration
   * @zh-CN 国际化配置
   * */
  const intl = useIntl();

  const columns: ProColumns<TaskListItem>[] = [
    {
      title: <FormattedMessage id="stat-engine.history-table.id" defaultMessage="Task ID" />,
      dataIndex: 'id',
      tip: 'The task id is the unique key',
      hideInTable: false,
      hideInSearch: true,
      hideInForm: true,
      render: (dom, entity) => {
        return (
          <a
            onClick={() => {
              if (onClickItem) {
                onClickItem(entity.plugin_name, entity.response, entity)
              }
            }}
          >
            {dom}
          </a>
        );
      }
    },
    {
      title: <FormattedMessage id="stat-engine.history-table.taskName" defaultMessage="Task Name" />,
      dataIndex: 'name',
      hideInSearch: true,
      hideInForm: true,
      hideInTable: true,
      tip: 'The task name is the unique key',
      render: (dom, entity) => {
        return (
          <a
            onClick={() => {
              setCurrentRow(entity);
              setShowDetail(true);
            }}
          >
            {dom}
          </a>
        );
      },
    },
    {
      title: <FormattedMessage id="stat-engine.history-table.pluginName" defaultMessage="Chart Name" />,
      dataIndex: 'plugin_name',
      valueType: 'text',
      render: (dom, entity) => {
        return (
          <a
            onClick={() => {
              setCurrentRow(entity);
              setShowDetail(true);
            }}
          >
            {dom}
          </a>
        );
      },
    },
    {
      title: <FormattedMessage id="stat-engine.history-table.pluginVersion" defaultMessage="Version" />,
      dataIndex: 'plugin_version',
      hideInSearch: true,
      hideInForm: true,
      valueType: 'text',
    },
    {
      title: <FormattedMessage id="stat-engine.history-table.percentage" defaultMessage="Percentage" />,
      dataIndex: 'percentage',
      hideInSearch: true,
      hideInForm: true,
      hideInTable: true,
      hideInSetting: true,
      valueType: 'progress',
    },
    {
      title: <FormattedMessage id="stat-engine.history-table.status" defaultMessage="Status" />,
      dataIndex: 'status',
      hideInForm: true,
      valueEnum: {
        Started: {
          text: <FormattedMessage id="stat-engine.history-table.started" defaultMessage="Started" />,
          status: 'Processing',
        },
        Finished: {
          text: <FormattedMessage id="stat-engine.history-table.finished" defaultMessage="Finished" />,
          status: 'Success',
        },
        Failed: {
          text: <FormattedMessage id="stat-engine.history-table.failed" defaultMessage="Failed" />,
          status: 'Error',
        },
      },
    },
    {
      title: <FormattedMessage id="stat-engine.history-table.startedAt" defaultMessage="Started" />,
      // sorter: true,
      dataIndex: 'started_time',
      hideInSearch: true,
      valueType: 'dateTime',
      renderFormItem: (item, { defaultRender, ...rest }, form) => {
        return defaultRender(item);
      },
    },
    {
      title: <FormattedMessage id="stat-engine.history-table.finishedAt" defaultMessage="Finished" />,
      // sorter: true,
      hideInSearch: true,
      dataIndex: 'finished_time',
      valueType: 'dateTime',
      renderFormItem: (item, { defaultRender, ...rest }, form) => {
        return defaultRender(item);
      },
    },
    {
      title: <FormattedMessage id="stat-engine.history-table.payload" defaultMessage="Payload" />,
      dataIndex: 'payload',
      hideInSearch: true,
      hideInForm: true,
      hideInTable: true,
      hideInSetting: true,
      valueType: 'jsonCode',
      renderText: (text, record, index, action) => {
        return JSON.stringify(text);
      },
      colSpan: 2,
    },
  ];

  return (
    <PageContainer className="history-table-page-container">
      <ProTable<TaskListItem, PageParams>
        className="history-table"
        headerTitle={intl.formatMessage({
          id: 'stat-engine.history-table.title',
          defaultMessage: 'Task History',
        })}
        actionRef={actionRef}
        rowKey="id"
        search={{
          labelWidth: 120,
        }}
        toolBarRender={() => []}
        request={listTasks}
        columns={columns}
      // rowSelection={{
      //   onChange: (_, selectedRows) => {
      //     setSelectedRows(selectedRows);
      //   },
      // }}
      />

      <Drawer
        width={'50%'}
        visible={showDetail}
        className="task-details"
        onClose={() => {
          setCurrentRow(undefined);
          setShowDetail(false);
        }}
        closable={false}
      >
        {currentRow?.name && (
          <ProDescriptions<TaskListItem>
            column={1}
            title={currentRow?.name}
            request={async () => ({
              data: currentRow || {},
            })}
            params={{
              id: currentRow?.name,
            }}
            columns={columns as ProDescriptionsItemProps<TaskListItem>[]}
          />
        )}
      </Drawer>
    </PageContainer>
  );
};

export default TableList;
