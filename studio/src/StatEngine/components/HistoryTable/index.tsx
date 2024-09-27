import { Drawer } from 'antd';
import React, { useState, useRef, useEffect } from 'react';
import { useIntl, FormattedMessage } from 'umi';
import { PageContainer } from '@ant-design/pro-layout';
import type { ProColumns, ActionType } from '@ant-design/pro-table';
import ProTable from '@ant-design/pro-table';
import type { ProDescriptionsItemProps } from '@ant-design/pro-descriptions';
import ProDescriptions from '@ant-design/pro-descriptions';
import { fetchTasks } from '../../../services/swagger/KnowledgeGraph';
import type { SortOrder } from 'antd/es/table/interface';
import { ChartResult } from '../WorkflowList/data';
import type { Workflow, TaskHistoryTableData, TaskHistory } from '../WorkflowList/data';

import './index.less';

type PageParams = {
  current?: number | undefined;
  pageSize?: number | undefined;
  status?: string;
};

export type HistoryTableProps = {
  onClickItem?: (chart: string, result?: ChartResult, task?: TaskHistory) => void;
  workflow?: Workflow;
  forceUpdateKey?: string;
};

function formatResponse(response: TaskHistoryTableData): Promise<Partial<TaskHistoryTableData>> {
  return Promise.resolve({
    ...response,
    success: true,
  });
}

const TableList: React.FC<HistoryTableProps> = (props) => {
  const { onClickItem, workflow, forceUpdateKey } = props;

  const [showDetail, setShowDetail] = useState<boolean>(false);
  const [, setForceUpdate] = useState<string>();

  useEffect(() => {
    setForceUpdate('');
  }, [forceUpdateKey])

  const actionRef = useRef<ActionType>();
  const [currentRow, setCurrentRow] = useState<TaskHistory>();
  // const [selectedRowsState, setSelectedRows] = useState<TaskHistory[]>([]);

  const listTasks = async (
    params: PageParams,
    sort: Record<string, SortOrder>,
    filter: Record<string, React.ReactText[] | null>,
  ) => {
    const queryParams = {
      page: params.current,
      pape_size: params.pageSize,
    } as Record<string, any>;

    let queryStrPayload: any[] = [];

    if (workflow) {
      queryStrPayload.push({
        field: 'workflow_id',
        value: workflow.id,
        operator: '='
      })
    }

    if (filter.status) {
      queryStrPayload.push({
        field: 'status',
        value: filter.status,
        operator: '='
      })
    }

    if (params.status) {
      queryStrPayload.push({
        field: 'status',
        value: params.status,
        operator: '='
      })
    }

    if (queryStrPayload.length > 1) {
      queryParams['query_str'] = JSON.stringify({
        operator: 'and',
        items: queryStrPayload
      })
    } else if (queryStrPayload.length === 1) {
      queryParams['query_str'] = JSON.stringify(queryStrPayload[0]);
    }

    console.log("List Tasks: ", queryParams, queryStrPayload);

    return await fetchTasks(queryParams)
      .then((response) => {
        return formatResponse({
          total: response.total,
          page: response.page,
          pageSize: response.page_size,
          data: response.records,
        });
      })
      .catch((error) => {
        console.log('requestDEGs Error: ', error);
        return formatResponse({ total: 0, page: 1, pageSize: 10, data: [] });
      });
  }

  /**
   * @en-US International configuration
   * @zh-CN 国际化配置
   * */
  const intl = useIntl();

  const columns: ProColumns<TaskHistory>[] = [
    {
      title: <FormattedMessage id="stat-engine.history-table.id" defaultMessage="Task ID" />,
      dataIndex: 'id',
      tooltip: 'The task id is the unique key',
      hideInTable: false,
      hideInSearch: true,
      hideInForm: true,
      render: (dom, entity) => {
        return (
          <a
            onClick={() => {
              if (onClickItem) {
                // TODO: Implement onClickItem
              }
            }}
          >
            {dom}
          </a>
        );
      }
    },
    {
      title: 'Task Name',
      dataIndex: 'name',
      hideInSearch: true,
      hideInForm: true,
      hideInTable: true,
      tooltip: 'The task name is the unique key',
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
      title: 'Chart Name',
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
      title: 'Version',
      dataIndex: 'plugin_version',
      hideInSearch: true,
      hideInForm: true,
      valueType: 'text',
    },
    {
      title: 'Percentage',
      dataIndex: 'percentage',
      hideInSearch: true,
      hideInForm: true,
      hideInTable: true,
      hideInSetting: true,
      valueType: 'progress',
    },
    {
      title: 'Status',
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
      title: 'Started',
      // sorter: true,
      dataIndex: 'started_time',
      hideInSearch: true,
      valueType: 'dateTime',
      renderFormItem: (item, { defaultRender, ...rest }, form) => {
        return defaultRender(item);
      },
    },
    {
      title: 'Finished',
      // sorter: true,
      hideInSearch: true,
      dataIndex: 'finished_time',
      valueType: 'dateTime',
      renderFormItem: (item, { defaultRender, ...rest }, form) => {
        return defaultRender(item);
      },
    },
    {
      title: 'Payload',
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
      <ProTable<TaskHistory, PageParams>
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
        {currentRow?.task_name && (
          <ProDescriptions<TaskHistory>
            column={1}
            title={currentRow?.task_name}
            request={async () => ({
              data: currentRow || {},
            })}
            params={{
              id: currentRow?.id,
            }}
            columns={columns as ProDescriptionsItemProps<TaskHistory>[]}
          />
        )}
      </Drawer>
    </PageContainer>
  );
};

export default TableList;
