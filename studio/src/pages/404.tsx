import { Button, Result } from 'antd';
import React from 'react';
import { history } from 'umi';

const NoFoundPage: React.FC = () => (
  <Result
    status="404"
    title="Oops, we can't seem to find the page you're looking for."
    subTitle={<span>It might be a broken link or the network might be down, Ensure your internet connection is stable and strong. <br />A quick fix might be to refresh the page. Or you can clear your browser cache and try again.</span>}
    extra={
      <Button type="primary" onClick={() => history.push('/')}>
        Back Home
      </Button>
    }
  />
);

export default NoFoundPage;
