import { QuestionCircleOutlined, InfoCircleOutlined, UserOutlined, FieldTimeOutlined } from '@ant-design/icons';
import { Space, Menu, Button, message, Dropdown } from 'antd';
import React, { useEffect, useState } from 'react';
import { getJwtAccessToken, logoutWithRedirect } from '@/components/util';
import { useAuth0 } from "@auth0/auth0-react";
import type { MenuProps } from 'antd';
import { history } from 'umi';
import { jwtDecode } from "jwt-decode";

import styles from './index.less';
import './extra.less'

export type SiderTheme = 'light' | 'dark';

export interface GlobalHeaderRightProps {
  username?: string;
}

const GlobalHeaderRight: React.FC<GlobalHeaderRightProps> = (props) => {
  const { loginWithRedirect, isAuthenticated, logout, user, getIdTokenClaims, getAccessTokenSilently } = useAuth0();
  const [current, setCurrent] = useState('user');
  const [username, setUsername] = useState(props.username || user?.name || user?.email || user?.nickname || 'Anonymous');

  useEffect(() => {
    const checkTokenValidity = async () => {
      if (!isAuthenticated) return;

      try {
        // const tokenClaims = await getIdTokenClaims();
        // if (tokenClaims) {
        //   const decodedToken = jwtDecode(tokenClaims.__raw);
        // }
        const token = getJwtAccessToken();
        if (token) {
          const decodedToken: any = jwtDecode(token);
          const currentTime = Date.now() / 1000;

          if (decodedToken.exp < currentTime + 5 * 60) {
            const newAccessToken = await getAccessTokenSilently();
            document.cookie = `jwt_access_token=${newAccessToken};max-age=86400;path=/`;
          }
        }
      } catch (error) {
        console.error('Error refreshing access token:', error);
      }
    };

    const intervalId = setInterval(checkTokenValidity, 5 * 60 * 1000);

    return () => clearInterval(intervalId);
  }, [isAuthenticated, getIdTokenClaims, getAccessTokenSilently]);

  useEffect(() => {
    // If the user is not authenticated, redirect to the login page.
    if (!isAuthenticated) {
      logoutWithRedirect();
      return;
    }

    // Save the id token to the cookie.
    getIdTokenClaims().then((claims) => {
      if (!claims) {
        return;
      }

      document.cookie = `jwt_access_token=${claims.__raw};max-age=86400;path=/`;

      // Get the redirectUrl from the query string.
      const redirectUrl = localStorage.getItem('redirectUrl');
      if (redirectUrl) {
        history.push('/' + redirectUrl);
      } else {
        history.push('/');
      }
    });
  }, [isAuthenticated]);

  useEffect(() => {
    if (props.username) {
      setUsername(props.username);
    } else if (user) {
      setUsername(user.name || user.email || user.nickname || 'Anonymous');
    }
  }, [props.username, user]);

  const directItems: MenuProps['items'] = [
    {
      label: username,
      key: 'user',
      icon: <UserOutlined />,
    },
    {
      label: 'v0.3.1',
      key: 'version',
      icon: <FieldTimeOutlined />
    },
  ]

  const items: MenuProps['items'] = [
    {
      label: 'Help',
      key: 'help',
      icon: <QuestionCircleOutlined />,
    },
    {
      label: 'About Us',
      key: 'about',
      icon: <InfoCircleOutlined />,
    },
    {
      label: 'ChangeLog',
      key: 'changelog',
      icon: <FieldTimeOutlined />
    },
  ]

  const onClick = (item: any) => {
    if (item.key === 'about') {
      history.push('/about')
    } else if (item.key === 'help') {
      history.push('/help')
    } else if (item.key === 'changelog') {
      history.push('/changelog')
    } else if (item.key === 'version') {
      window.open('https://github.com/open-prophetdb/biomedgps/releases', '_blank');
    }
  };

  const logoutWithRedirectRaw = () => {
    logout({ logoutParams: { returnTo: window.location.origin } }).then(() => {
      // Remove the jwt_access_token from the cookie.
      document.cookie = 'jwt_access_token=;max-age=0;path=/';
      // Redirect to a warning page that its route name is 'not-authorized'.
      history.push('/not-authorized');
    }).catch((error) => {
      message.error("Failed to logout, please try again later.");
      console.log("logout error: ", error);
    });
  }

  return (
    <Space className={`${styles.right} ${styles.light} right-content`}>
      <Menu onClick={onClick} selectedKeys={[current]} theme="light" mode="inline" items={directItems} />
      <Dropdown menu={{ items, onClick: onClick }} placement="bottomLeft">
        <Button type="text" icon={<InfoCircleOutlined />} style={{ height: '40px' }}>About</Button>
      </Dropdown>
      <Button type={isAuthenticated ? 'default' : 'primary'} danger={isAuthenticated}
        onClick={() => isAuthenticated ? logoutWithRedirectRaw() : loginWithRedirect()}>
        Sign {isAuthenticated ? 'Out' : 'In'}
      </Button>
    </Space>
  );
};
export default GlobalHeaderRight;
