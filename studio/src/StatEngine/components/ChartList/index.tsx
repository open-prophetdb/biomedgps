import { DislikeOutlined, FunctionOutlined, LikeOutlined } from '@ant-design/icons';
import { List, Space, Tag } from 'antd';
import { filter } from 'lodash';
import React, { memo, useEffect, useState } from 'react';
import { useHistory } from 'react-router-dom';
import { useIntl } from 'umi';

// API Endpoint
import { getCharts } from '@/services/swagger/StatEngine';

import type { ChartMetaData, ChartResult, Icon } from './data';
import './index.less';

// Custom Data
import { langData } from './lang';
type UIContext = Record<string, any>;

export type ChartListProps = {
  onClickItem?: (chart: ChartMetaData, result?: ChartResult, fieldsValue?: Record<string, any>) => void;
};

const ChartList: React.FC<ChartListProps> = (props) => {
  const { onClickItem } = props;

  const [charts, setCharts] = useState<ChartMetaData[]>([]);
  const [total, setTotal] = useState<number>(0);

  const history = useHistory();

  const intl = useIntl();

  const uiContext: UIContext = {};
  Object.keys(langData).forEach((key) => {
    uiContext[key] = intl.formatMessage(langData[key]);
  });

  useEffect(() => {
    getCharts({}).then((response) => {
      const chartList = filter(response.data, (item) => {
        return item.category === 'Chart';
      });

      setCharts(chartList);
      setTotal(chartList.length);
    });
  }, []);

  const IconText = ({ icon, text }) => (
    <Space>
      {React.createElement(icon)}
      {text}
    </Space>
  );

  const showTotal = (num: number) => {
    return `${uiContext.totalItems}: ${num}`;
  };

  const getLogo = (icons: Icon[]): string => {
    return icons[0].src ? icons[0].src : "";
  };

  const titleLink = (name: string, version: string) => {
    return <a className="title">{`${name}- ${version}`}</a>;
  };

  console.log('ChartList updated');

  return (
    <List
      className="chart-list"
      itemLayout="vertical"
      size="large"
      grid={{
        xs: 1,
        sm: 1,
        md: 1,
        lg: 1,
        xl: 2,
        xxl: 2,
      }}
      pagination={{
        onChange: (page) => {
          console.log(page);
        },
        pageSize: 10,
        total,
        showTotal,
        showSizeChanger: true,
        showQuickJumper: true,
      }}
      dataSource={charts}
      renderItem={(item) => (
        <List.Item
          className="chart-item"
          onClick={() => {
            if (onClickItem) {
              onClickItem(item, undefined, undefined);
            } else {
              history.push('/stat-engine/index', {
                chart: item,
                result: null,
              });
            }
          }}
          key={item.short_name}
          actions={[
            <IconText icon={LikeOutlined} text="156" key="list-vertical-star-o" />,
            <IconText icon={DislikeOutlined} text="1" key="list-vertical-like-o" />,
            <IconText icon={FunctionOutlined} text="2" key="list-vertical-message" />,
          ]}
          extra={<img alt="logo" src={getLogo(item.icons)} />}
        >
          <List.Item.Meta
            title={titleLink(item.name, item.version)}
            description={item.maintainers}
          />
          <span className="description">{item.description}</span>
          {item.tags.map((tag) => {
            return <Tag key={tag}>{tag}</Tag>;
          })}
        </List.Item>
      )}
    />
  );
};

export default memo(ChartList);
