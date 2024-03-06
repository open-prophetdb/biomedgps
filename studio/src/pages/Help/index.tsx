import { Card } from 'antd';
import { useModel } from 'umi';
import React, { useEffect } from 'react';
import { MarkdownViewer } from 'biominer-components';
import RehypeRaw from 'rehype-raw';
import RehypeToc from 'rehype-toc';

import './index.less';

const Help: React.FC = () => {
  const [markdown, setMarkdown] = React.useState('');
  const { initialState } =
    useModel('@@initialState');
  const markdownLink = `${initialState?.customSettings?.helpUrl}`;

  useEffect(() => {
    fetch(markdownLink)
      .then((response) => response.text())
      .then((text) => setMarkdown(text));
  }, []);

  return (
    <Card className="help">
      <MarkdownViewer markdown={markdown} rehypePlugins={[RehypeRaw, RehypeToc]} />
    </Card>
  );
};

export default Help;
