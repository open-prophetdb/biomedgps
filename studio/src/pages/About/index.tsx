import { Card } from 'antd';
import { useModel } from 'umi';
import React, { useEffect } from 'react';
import { MarkdownViewer } from 'biominer-components';
import RehypeRaw from 'rehype-raw';

import './index.less';

const About: React.FC = () => {
  const [markdown, setMarkdown] = React.useState('');
  const { initialState } =
    useModel('@@initialState');
  const markdownLink = `${initialState?.customSettings?.aboutUrl}`;

  useEffect(() => {
    fetch(markdownLink)
      .then((response) => response.text())
      .then((text) => setMarkdown(text));
  }, []);

  return (
    <Card className="about">
      <MarkdownViewer markdown={markdown} rehypePlugins={[RehypeRaw]} />
    </Card>
  );
};

export default About;
