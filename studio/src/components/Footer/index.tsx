import React, { useEffect, useState } from 'react';
import CookieConsent, { Cookies } from 'react-cookie-consent';
import { GithubOutlined, FieldTimeOutlined } from '@ant-design/icons';
import { DefaultFooter } from '@ant-design/pro-components';
import { Row } from 'antd';
import type { MenuProps } from 'antd';
import './index.less';

const Footer: React.FC = () => {
  const currentYear = new Date().getFullYear();
  const [cookieName, setCookieName] = useState<string>('biomedgps-cookie-consent-form');
  const [cookieEnabled, setCookieEnabled] = useState<boolean | undefined>(undefined);
  const version = process.env.UMI_APP_VERSION || '0.1.0';

  useEffect(() => {
    const v = Cookies.get(cookieName);
    setCookieEnabled(v === 'true' ? true : false);
    console.log('Cookie Status: ', v, typeof v, cookieEnabled);
    if (v) {
      allowTrack();
    }
  }, []);

  const allowTrack = function () {
    const link = "//cdn.clustrmaps.com/map_v2.js?cl=006c8f&w=106&t=n&d=MPWClGSAiOC-0JlMxUP_pPwFfxi1-6reWYzVXQTWDYc&co=f5f5f5"
    // Check whether the script is already loaded.
    const scripts = document.getElementsByTagName('script');
    for (let i = 0; i < scripts.length; i++) {
      if (scripts[i].src === link && scripts[i].type === 'text/javascript') {
        return;
      }
    }
    // <script type="text/javascript" src="//cdn.clustrmaps.com/map_v2.js?cl=006c8f&w=106&t=n&d=MPWClGSAiOC-0JlMxUP_pPwFfxi1-6reWYzVXQTWDYc&co=f5f5f5" async="async"></script>
    var custom_script = document.createElement('script');
    custom_script.id = 'clustrmaps';
    custom_script.setAttribute('src', link);
    // custom_script.setAttribute('async', 'async');
    custom_script.setAttribute('type', 'text/javascript');

    const footer = document.getElementsByClassName('ant-pro-global-footer');
    if (footer) {
      // We ensure that the clustrmaps script is only added once
      Array.from(footer).forEach((item) => {
        item.appendChild(custom_script);
      });
    } else {
      console.log('footer not found');
    }
  };

  return (
    <Row className='footer-container'>
      <DefaultFooter
        copyright={`${currentYear} OpenProphetDB Team | Version ${version}`}
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
      <CookieConsent
        location="bottom"
        cookieName={cookieName}
        style={{ background: '#2B373B' }}
        enableDeclineButton
        buttonStyle={{ color: '#4e503b', fontSize: '0.9rem' }}
        expires={150}
        onAccept={() => {
          allowTrack();
        }}
      >
        This website uses an toolbox from clustrmaps.com to count the number of visitors, but we
        don't gather and track your personal information.
      </CookieConsent>
    </Row>
  );
};

export default Footer;
