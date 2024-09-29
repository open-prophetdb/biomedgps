import type { ProFormColumnsType } from '@ant-design/pro-form';

export declare type DataItem = {
  name: string;
  state: string;
};

export declare type ReadOnlyData = Record<string, any[][]>;

export declare type Example = {
  title: string;
  key: string;
  data: [];
  arguments: Record<string, any>;
};

export declare type Icon = {
  // type: 'image/png', sizes: '192x192'
  src?: string;
  type?: string;
  sizes?: string;
};

export type TaskHistory = {
  id?: string;
  workspace_id: string;
  workflow_id: string;
  task_id: string;
  task_name: string;
  description?: string;
  submitted_time: string;
  started_time?: string;
  finished_time?: string;
  task_params: any;
  labels?: any;
  status?: string;
  owner: string;
  groups?: string[];
};

export type TaskHistoryTableData = {
  data: TaskHistory[];
  total: number;
  page: number;
  pageSize: number;
};

export type Workflow = {
  id: string;
  name: string;
  version: string;
  description?: string;
  category: string;
  home: string;
  source: string;
  short_name: string;
  icons?: any;
  author?: string;
  maintainers?: string[];
  tags?: string[];
  readme?: string;
};

export type WorkflowTableData = {
  data: Workflow[];
  total: number;
  page: number;
  pageSize: number;
};

export declare type ChartData = {
  fields: ProFormColumnsType<DataItem>[];
  examples: Example[];
};

export type FileMeta = {
  filename: string;
  filetype: string;
};

export declare type ChartResult = {
  files?: FileMeta[];
  charts?: FileMeta[];
  task_id?: string;
  log_message?: string;
};
