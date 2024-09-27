import { Empty } from 'antd';
import React, { memo, useEffect, useState } from 'react';
import ReactMarkdown from 'react-markdown';
import rehypeRaw from 'rehype-raw';
import rehypeToc from 'rehype-toc';
import rehypeVideo from 'rehype-video';
import rehypeSlug from 'rehype-slug';
import rehypeAutolinkHeadings from 'rehype-autolink-headings';
import remarkGfm from 'remark-gfm';
import remarkToc from 'remark-toc';

import './index.less';


export type MarkdownProps = {
  markdownContent?: string;
};

const MarkdownViewer: React.FC<MarkdownProps> = (props) => {
  const { markdownContent } = props;

  const [markdown, setMarkdown] = useState<string | null>(null);

  useEffect(() => {
    if (markdownContent) {
      setMarkdown(markdownContent);
    }
  }, [markdownContent]);

  console.log('MarkdownViewer: updated');

  return markdown ? (
    <ReactMarkdown
      key={markdown}
      rehypePlugins={[rehypeRaw, rehypeSlug, rehypeToc, rehypeAutolinkHeadings, rehypeVideo]}
      className="markdown-viewer"
      remarkPlugins={[remarkGfm, remarkToc]}
    >
      {markdown}
    </ReactMarkdown>
  ) : (
    <Empty />
  );
};

export default memo(MarkdownViewer);
