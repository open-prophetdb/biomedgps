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
import axios from 'axios';
import './index.less';


export type MarkdownParams = {
  filelink: string,
}

export type MarkdownProps = {
  url: string | null;
  enableToc?: boolean;
  getFile?: (params: MarkdownParams) => Promise<any>;
};

const MarkdownViewer: React.FC<MarkdownProps> = (props) => {
  const { url, getFile, enableToc } = props;

  const fetchMarkdown = function (url: string): Promise<string> {
    if (url.match(/^(minio|file):\/\//)) {
      if (getFile) {
        return getFile({
          filelink: url
        }).then((response: any) => {
          return response
        }).catch((error: any) => {
          return error.data.msg ? error.data.msg : error.data
        })
      } else {
        return new Promise((resolve, reject) => {
          resolve("Please specify getFile function.")
        })
      }
    } else {
      try {
        return axios(url).then((response) => {
          if (response.status !== 200) {
            return 'No Content.';
          }
          return response.data;
        });
      } catch (error) {
        console.log(`Cannot fetch ${url}, the reason is ${error}`)
        return new Promise((resolve, reject) => {
          reject('No Content.')
        });
      }
    }
  }

  const [markdown, setMarkdown] = useState<string | null>(null);

  useEffect(() => {
    if (url) {
      fetchMarkdown(url).then((response) => setMarkdown(response || null));
    }
  }, [url]);

  console.log('MarkdownViewer: updated');

  let rehypePlugins = []
  if (enableToc) {
    rehypePlugins = [rehypeRaw, rehypeSlug, rehypeToc, rehypeAutolinkHeadings, rehypeVideo]
  } else {
    rehypePlugins = [rehypeRaw, rehypeSlug, rehypeAutolinkHeadings, rehypeVideo]
  }

  return markdown ? (
    <ReactMarkdown
      key={url}
      rehypePlugins={rehypePlugins}
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
