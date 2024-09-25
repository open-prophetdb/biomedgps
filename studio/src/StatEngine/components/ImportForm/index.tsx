import { InfoCircleFilled } from '@ant-design/icons';
import { Button, Col, Input, message, Row, Tabs, Tag, Tooltip } from 'antd';
import React, { memo, useState } from 'react';

import type { DataLoader } from '../Common/data';

import './index.less';

const { TabPane } = Tabs;
const { Search } = Input;

export type ImportFormProps = {
  onLoad: (loader: DataLoader) => void;
};

const example = 'http://nordata-cdn.oss-cn-shanghai.aliyuncs.com/iris.csv';

const ImportForm: React.FC<ImportFormProps> = (props) => {
  const { onLoad } = props;

  const [exampleLink, setExampleLink] = useState<string>('');
  // const [loadActive, setLoadActive] = useState<boolean>(false);

  const doCopy = (link: string) => {
    setExampleLink(link);
    navigator.clipboard.writeText(link);
    message.success('Copy Successful');
  };

  const onSearch = (externalURL: string) => {
    // setLoadActive(true);
    // TODO: Robust?
    if (externalURL.length > 0) {
      console.log('onSearch: ', externalURL);
      onLoad({
        dataSource: externalURL,
        dataSourceType: 'csvFile',
        queryParams: {},
        dataType: 'objectArray',
      });
    }
  };

  console.log('ImportForm updated');

  return (
    <Row className="import-form">
      <Col className="control-panel">
        <span>
          <InfoCircleFilled />
          Use first row as column headers &nbsp;
        </span>
      </Col>
      <Tabs defaultActiveKey="1" type="card">
        <TabPane tab={<span>By URL</span>} key="1">
          <Row className="import-box">
            <Search
              placeholder="Input your data with URL"
              style={{ width: '80%' }}
              // disabled={!exampleLink || loadActive}
              onSearch={onSearch}
              value={exampleLink}
              enterButton="Load"
            />
            <Row>HTTPS only. Supported file types: CSV, TSV.</Row>
            <br />
            <span style={{ marginBottom: '10px' }}>Example Data URL</span>
            <Tooltip placement="top" title={<a onClick={() => doCopy(example)}>Copy Link</a>}>
              <Tag>{example}</Tag>
            </Tooltip>
          </Row>
        </TabPane>
        <TabPane tab={<span>Browser</span>} key="2" disabled>
          <Row className="import-box">
            <Button>Browser Remote Files</Button>
            <Row>Supported file types: CSV, TSV</Row>
          </Row>
        </TabPane>
        <TabPane tab={<span>DataSets</span>} key="3" disabled>
          <Row className="import-box">
            <Button>Browser DataSets</Button>
          </Row>
        </TabPane>
      </Tabs>
    </Row>
  );
};

export default memo(ImportForm);
