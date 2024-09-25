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

export declare type ChartMetaData = {
  id: string;
  name: string;
  version: string;
  description: string;
  category: string;
  home: string;
  source: string;
  short_name: string;
  icons: Icon[];
  author: string;
  maintainers: string[];
  tags: string[];
  readme: string;
};

export declare type ChartData = {
  fields: ProFormColumnsType<DataItem>[];
  examples: Example[];
};

export declare type ChartResult = {
  results?: string[];
  charts?: string[];
  task_id?: string;
  log?: string;
};
