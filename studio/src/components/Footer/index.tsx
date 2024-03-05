import { GithubOutlined } from '@ant-design/icons';
import { DefaultFooter } from '@ant-design/pro-components';
import { Row } from 'antd';
import './index.less';

const Footer: React.FC = () => {
  const currentYear = new Date().getFullYear();

  return (
    <Row className='footer-container'>
      <DefaultFooter
        copyright={`${currentYear} OpenProphetDB Team`}
        links={[
          {
            key: 'open-prophetdb',
            title: 'OpenProphetDB',
            href: 'http://www.prophetdb.org/',
            blankTarget: true,
          },
          {
            key: 'github',
            title: <GithubOutlined />,
            href: 'https://github.com/open-prophetdb',
            blankTarget: true,
          },
          {
            key: 'chinese-quartet',
            title: 'Chinese Quartet',
            href: 'https://chinese-quartet.org',
            blankTarget: true,
          },
        ]}
      />
    </Row>
  );
};

export default Footer;
