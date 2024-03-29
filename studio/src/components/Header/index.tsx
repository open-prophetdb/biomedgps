import { QuestionCircleOutlined, InfoCircleOutlined, UserOutlined, FieldTimeOutlined } from '@ant-design/icons';
import { Space, Menu, Button, message } from 'antd';
import React, { useEffect, useState } from 'react';
import { history } from 'umi';
import { useAuth0 } from "@auth0/auth0-react";
import type { MenuProps } from 'antd';
import styles from './index.less';
import './extra.less'

export type SiderTheme = 'light' | 'dark';

export interface GlobalHeaderRightProps {
  username?: string;
}

const GlobalHeaderRight: React.FC<GlobalHeaderRightProps> = (props) => {
  const { loginWithRedirect, isAuthenticated, logout, user, getIdTokenClaims } = useAuth0();
  const [current, setCurrent] = useState('user');
  const [username, setUsername] = useState(props.username || user?.name || user?.email || user?.nickname || 'Anonymous');

  useEffect(() => {
    // If the user is not authenticated, redirect to the login page.
    if (!isAuthenticated) {
      // Save the current hash as the redirect url
      let redirectUrl = window.location.hash.split("#").pop();
      if (redirectUrl) {
        redirectUrl = redirectUrl.replaceAll('/', '')
        localStorage.setItem('redirectUrl', redirectUrl);
      } else {
        localStorage.setItem('redirectUrl', '');
      }
      // Redirect to a warning page that its route name is 'not-authorized'.
      history.push('/not-authorized');
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

  const items: MenuProps['items'] = [
    {
      label: username,
      key: 'user',
      icon: <UserOutlined />,
    },
    {
      label: 'About',
      key: 'about',
      icon: <InfoCircleOutlined />,
    },
    {
      label: 'Help',
      key: 'help',
      icon: <QuestionCircleOutlined />,
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
    }
  };

  const logoutWithRedirect = () => {
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
      <Menu onClick={onClick} selectedKeys={[current]} theme="light" mode="inline" items={items} />
      <Button type={isAuthenticated ? 'default' : 'primary'} danger={isAuthenticated}
        onClick={() => isAuthenticated ? logoutWithRedirect() : loginWithRedirect()}>
        Sign {isAuthenticated ? 'Out' : 'In'}
      </Button>
    </Space>
  );
};
export default GlobalHeaderRight;
