import { HotTable } from '@handsontable/react';
import { message } from 'antd';
import { map } from 'lodash';
import React, { memo, useEffect, useState } from 'react';
import { getLocale } from 'umi';

import type { DataLoader, PapaTableData, TableData } from '../Common/data';
import { fetchData } from '../Common/service';
import type { ColumnDefinition, ColumnSchema, ColumnType, FieldType, Validator } from './data';

import 'handsontable/dist/handsontable.full.min.css';
import 'handsontable/languages/zh-CN';
import './index.less';

export type DataTableProps = {
  dataKey: string;
  dataLoader?: DataLoader;
  columns?: ColumnSchema[];
  height?: number | string;
  width?: number | string;
  updateData: (dataKey: string, data: any[][], headers: string[]) => void;
};

const GenericValidator = (query: any, callback: any) => {
  callback(true);
};

const genRegexValidator = (pattern: string) => {
  return (query: any, callback: any) => {
    const regex = new RegExp(pattern);
    callback(regex.test(query));
  };
};

const genMinMaxValidator = (min: number, max: number) => {
  return (query: any, callback: any) => {
    if (query < min || query > max || min > max) {
      callback(false);
    }

    callback(true);
  };
};

const convertType = (dataType: FieldType): ColumnType => {
  if (['float', 'int', 'double'].includes(dataType)) {
    return 'numeric';
  }

  if (dataType === 'boolean') {
    return 'dropdown';
  }

  return 'text';
};

const getValidator = (column: ColumnSchema): Validator => {
  if (column.validator === 'minMax' && column.min && column.max) {
    return genMinMaxValidator(column.min, column.max);
  }

  if (column.validator === 'regex' && column.pattern) {
    return genRegexValidator(column.pattern);
  }

  return GenericValidator;
};

const makeColumns = (columns: ColumnSchema[] | undefined): ColumnDefinition[] | undefined => {
  if (columns) {
    const columnDefs: ColumnDefinition[] = [];
    columns.forEach((column) => {
      columnDefs.push({
        data: column.name,
        type: convertType(column.type),
        source: column.choices,
        validator: getValidator(column),
        allowEmpty: false,
      });
    });
    return columnDefs;
  }

  return undefined;
};

const getHeader = (data: TableData): string[] => {
  if (data.length > 0) {
    return Object.keys(data[0]);
  }

  return [];
};

const convertData = (data: TableData): any[][] | undefined => {
  if (data.length > 0) {
    const headers = getHeader(data);
    const body = map(data, (item) => {
      const record: any = [];
      headers.forEach((field) => {
        record.push(item[field]);
      });

      return record;
    });

    return body;
  }

  return undefined;
};

const tableSettings = {
  bindRowsWithHeaders: true,
  colHeaders: true,
  rowHeaders: true,
  filters: true,
  className: 'htCenter',
  minRows: 30,
  minCols: 5,
  data: null,
  manualColumnFreeze: true,
  dropdownMenu: [
    'col_left',
    '---------',
    'col_right',
    '---------',
    'undo',
    '---------',
    'redo',
    '---------',
    'make_read_only',
    '---------',
    'clear_column',
    '---------',
    'alignment',
    '---------',
    'filter_by_condition',
    'filter_operators',
    'filter_by_condition2',
    'filter_by_value',
    'filter_action_bar',
  ],
  autoRowSize: true,
  autoColSize: true,
  stretchH: 'all',
  height: '100%',
  width: '100%',
  manualColumnResize: true,
  multiColumnSorting: true,
  undo: true,
  redo: true,
  contextMenu: {
    items: {
      make_read_only: {},
      redo: {},
      undo: {},
      row_below: {},
      row_above: {},
      freeze_column: {},
      unfreeze_column: {},
    },
  },
};

const DataTable: React.FC<DataTableProps> = (props) => {
  const { dataKey, dataLoader, height, width, columns, updateData } = props;

  const [tableData, setTableData] = useState<any[][] | undefined>(undefined);
  const [tableHeader, setTableHeader] = useState<string[]>([]);
  const [ref, setRef] = useState<HotTable>();

  const onUpdateData = (
    key: string,
    callback: typeof updateData,
    sourceHeader: string[] | undefined,
    sourceData: any[][] | undefined,
  ) => {
    console.log('onUpdateData: ', {
      sourceData,
      sourceHeader,
      hotTableData: ref?.hotInstance.getData(),
      hotTableHeader: ref?.hotInstance.getColHeader(),
    });
    const hotTableData =
      sourceData && sourceData.length > 0 ? sourceData : ref?.hotInstance.getData();
    const hotTableHeader =
      sourceHeader && sourceHeader.length > 0 ? sourceHeader : ref?.hotInstance.getColHeader();
    // @ts-ignore
    callback(key, hotTableData || [], hotTableHeader || []);
  };

  useEffect(() => {
    if (dataLoader && dataLoader.dataSourceType === 'csvFile') {
      fetchData(dataLoader.dataSource)
        .then((response) => {
          console.log('getFile: ', response);
          const papaTableData: PapaTableData = response;
          setTableData(convertData(papaTableData.data));
          setTableHeader(getHeader(papaTableData.data));
          message.success('Loaded Suessfully.');
        })
        .catch((error) => {
          console.log('getFile Error: ', error);
          message.error("Can't load the data, please check your url & try agian later.");
        });
    }
  }, [dataLoader]);

  console.log('DataTable: ', { dataLoader, columns: makeColumns(columns), tableData, tableHeader });

  return (
    <HotTable
      ref={(tableRef: HotTable) => {
        setRef(tableRef);
      }}
      language={getLocale()}
      className="data-table"
      data={tableData}
      settings={tableSettings}
      colHeaders={tableHeader}
      rowHeaders={true}
      height={height}
      width={width}
      columns={makeColumns(columns)}
      afterChange={(changes) => {
        console.log('Changes in DataTable: ', changes);
        onUpdateData(dataKey, updateData, tableHeader, ref?.hotInstance.getData());
      }}
      afterLoadData={(sourceData) => {
        onUpdateData(dataKey, updateData, tableHeader, sourceData);
      }}
      licenseKey="non-commercial-and-evaluation"
    />
  );
};

export default memo(DataTable);
