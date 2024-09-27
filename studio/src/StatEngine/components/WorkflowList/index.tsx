import { DislikeOutlined, FunctionOutlined, LikeOutlined } from '@ant-design/icons';
import { List, Space, Tag } from 'antd';
import { filter } from 'lodash';
import React, { memo, useEffect, useState } from 'react';
import { history } from 'umi';

// API Endpoint
import { fetchWorkflows } from '../../../services/swagger/KnowledgeGraph';

import type { Workflow, WorkflowTableData, Icon } from './data';
import './index.less';

// Custom Data

export type WorkflowListProps = {
  onClickItem?: (workflow: Workflow, fieldsValue?: Record<string, any>) => void;
};

const WorkflowList: React.FC<WorkflowListProps> = (props) => {
  const { onClickItem } = props;

  const [workflows, setWorkflows] = useState<Workflow[]>([]);
  const [total, setTotal] = useState<number>(0);

  useEffect(() => {
    fetchWorkflows({}).then((response) => {
      const workflowList = filter(response.records, (item) => {
        return item.category === 'Workflow';
      });

      setWorkflows(workflowList);
      setTotal(workflowList.length);
    });
  }, []);

  const IconText = ({ icon, text }: { icon: any, text: string }) => (
    <Space>
      {React.createElement(icon)}
      {text}
    </Space>
  );

  const showTotal = (num: number) => {
    return `Total ${num} workflows`;
  };

  const getLogo = (icons: Icon[]): string => {
    return icons[0].src ? icons[0].src : "";
  };

  const titleLink = (name: string, version: string) => {
    return <a className="title">{`${name}- ${version}`}</a>;
  };

  console.log('WorkflowList updated');

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
      dataSource={workflows}
      renderItem={(item) => (
        <List.Item
          className="chart-item"
          onClick={() => {
            if (onClickItem) {
              onClickItem(item, undefined);
            } else {
              // history.push('/stat-engine/index', {
              //   workflow: item,
              //   result: null,
              // });
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
          {item.tags && item.tags.map((tag) => {
            return <Tag key={tag}>{tag}</Tag>;
          })}
        </List.Item>
      )}
    />
  );
};

export default memo(WorkflowList);
