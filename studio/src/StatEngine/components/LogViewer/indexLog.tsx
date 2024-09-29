import { DownloadOutlined, SyncOutlined } from '@ant-design/icons';
import { Button, Col, Empty, Space, Spin } from 'antd';
import { memo, useEffect, useState } from 'react';
import ReactAnsi from 'react-ansi';

import './index.less';

export type LogProps = {
  logMessage: string;
  height: string;
};

const LogViewer: React.FC<LogProps> = (props) => {
  const { logMessage, height } = props;

  const buttons = (
    <Space style={{ marginTop: '5px', display: 'flex', justifyContent: 'flex-end' }}>
      <Button icon={<SyncOutlined />} disabled>
        Force Update
      </Button>
      <Button icon={<DownloadOutlined />}>Download</Button>
    </Space>
  );

  console.log('LogViewer updated');

  if (!logMessage) {
    return <Empty description="No log message" />;
  }

  return (
    <Col className="log-container">
      <ReactAnsi bodyStyle={{ height: height, overflowY: 'auto' }} log={logMessage} />
      {buttons}
    </Col>
  );
};

export default memo(LogViewer);
