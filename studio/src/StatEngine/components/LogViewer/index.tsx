// @ts-ignore
import { DownloadOutlined, SyncOutlined } from '@ant-design/icons';
import { Button, Col, Empty, Space } from 'antd';
import { memo } from 'react';
import { LazyLog } from 'react-lazylog';

export type LogProps = {
  url: string;
  height: string;
};

const LogViewer: React.FC<LogProps> = (props) => {
  const { url, height } = props;

  const buttons = (
    <Space style={{ marginTop: '5px', display: 'flex', justifyContent: 'flex-end' }}>
      <Button icon={<SyncOutlined />}>Force Update</Button>
      <Button icon={<DownloadOutlined />}>Download</Button>
    </Space>
  );

  console.log('LogViewer updated');

  return url ? (
    <Col>
      <LazyLog height={height} url={url} enableSearch extraLines={1} follow />
      {buttons}
    </Col>
  ) : (
    <Empty />
  );
};

export default memo(LogViewer);
