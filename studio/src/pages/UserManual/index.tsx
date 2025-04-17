import { Card } from 'antd';
import { useModel } from 'umi';
import React, { useEffect } from 'react';
import { MarkdownViewer } from 'biominer-components';
import RehypeRaw from 'rehype-raw';

import './index.less';

const UserManual: React.FC = () => {
  const [markdown, setMarkdown] = React.useState('');
  const { initialState } =
    useModel('@@initialState');
  // @ts-ignore
  const markdownLink = `${initialState?.customSettings?.userManualUrl}`;

  useEffect(() => {
    fetch(markdownLink)
      .then((response) => response.text())
      .then((text) => setMarkdown(text));
  }, []);

  return (
    <Card className="user-manual">
      <MarkdownViewer markdown={markdown} rehypePlugins={[RehypeRaw]} />
    </Card>
  );
};

export default UserManual;
