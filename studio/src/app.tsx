import Footer from '@/components/Footer';
import Header from '@/components/Header';
import { RequestConfig, history, RuntimeConfig } from 'umi';
import { PageLoading, SettingDrawer } from '@ant-design/pro-components';
import { Auth0Provider } from '@auth0/auth0-react';
import { CustomSettings, AppVersion } from '../config/defaultSettings';

// 运行时配置
// @ts-ignore
const publicPath = window.publicPath || process.env.PUBLIC_PATH || '/';
const defaultCustomSettings = {
  changeLogUrl: `${publicPath}README/changelog.md`,
  aboutUrl: `${publicPath}README/about.md`,
  helpUrl: `${publicPath}README/help.md`,
  websiteTitle: '',
  websiteLogo: `${publicPath}logo-white.png`,
  websiteDescription: 'Network Medicine for Disease Mechanism and Treatment Based on AI and knowledge graph.',
  websiteKeywords: 'Network Medicine, MultiOmics Data, Treatment, AI, Knowledge Graph',
  defaultDataset: '000000',
  mode: 'Developer'
}

const isDev = process.env.NODE_ENV === 'development';
const apiPrefix = process.env.UMI_APP_API_PREFIX ? process.env.UMI_APP_API_PREFIX : window.location.origin;
const CLIENT_ID = process.env.UMI_APP_AUTH0_CLIENT_ID ? process.env.UMI_APP_AUTH0_CLIENT_ID : '<your-client-id>';
const AUTH0_DOMAIN = process.env.UMI_APP_AUTH0_DOMAIN ? process.env.UMI_APP_AUTH0_DOMAIN : '<your-domain>';

console.log('apiPrefix', process.env, apiPrefix);

const getJwtAccessToken = (): string | null => {
  let jwtToken = null;
  // Check if the cookie exists
  if (document.cookie && document.cookie.includes("jwt_access_token=")) {
    // Retrieve the cookie value
    // @ts-ignore
    jwtToken = document.cookie
      .split("; ")
      .find((row) => row.startsWith("jwt_access_token="))
      .split("=")[1];
  }

  if (jwtToken) {
    console.log("JWT access token found in the cookie.");
    return jwtToken;
  } else {
    console.log("JWT access token not found in the cookie.");
    return null;
  }
}

const getUsername = (): string | undefined => {
  const accessToken = getJwtAccessToken();
  if (accessToken) {
    const payload = accessToken.split('.')[1];
    const base64 = payload.replace(/-/g, '+').replace(/_/g, '/');
    const padLength = 4 - (base64.length % 4);
    const paddedBase64 = padLength < 4 ? base64 + "=".repeat(padLength) : base64;
    const payloadJson = JSON.parse(atob(paddedBase64));
    return payloadJson['username'];
  } else {
    return undefined;
  }
}

export const request: RequestConfig = {
  timeout: 120000,
  // More details on ./config/proxy.ts or ./config/config.cloud.ts
  baseURL: apiPrefix,
  errorConfig: {
    errorHandler: (resData) => {
      return {
        ...resData,
        success: false,
        showType: 0,
        errorMessage: resData.message,
      };
    },
  },
  requestInterceptors: [(url: string, options) => {
    const visitorId = localStorage.getItem('rapex-visitor-id')
    // How to get a jwt_access_token from the cookie?
    const jwt_access_token = getJwtAccessToken()

    let headers = {}
    if (visitorId) {
      headers = {
        "x-auth-users": visitorId,
        // TODO: Support JWT
        "Authorization": "Bearer " + (jwt_access_token ? jwt_access_token : visitorId)
      }
    } else {
      headers = {}
    }

    return ({
      url: url,
      options: { ...options, headers: headers }
    })
  }],
  responseInterceptors: [
    (response) => {
      console.log("responseInterceptors: ", response);
      if (response.status === 401) {
        // Save the current hash as the redirect url
        let redirectUrl = window.location.hash.split("#").pop();
        if (redirectUrl) {
          redirectUrl = redirectUrl.replaceAll('/', '')
          localStorage.setItem('redirectUrl', redirectUrl);
          // Redirect to a warning page that its route name is 'not-authorized'.
          history.push('/not-authorized?redirectUrl=' + redirectUrl);
        } else {
          localStorage.setItem('redirectUrl', '');
          history.push('/not-authorized');
        }

        return new Promise(() => { });
      }

      return response;
    }
  ],
};

/**
 * @see  https://umijs.org/docs/api/runtime-config#getinitialstate
 * */
// TODO: After releasing the first version, try to improve the customized settings.
export async function getInitialState(): Promise<{
  loading?: boolean;
  collapsed?: boolean;
  customSettings?: CustomSettings;
  appVersion?: AppVersion;
}> {
  const customSettings: CustomSettings = defaultCustomSettings;
  const appVersion: AppVersion = {
    version: 'unknown',
    dbVersion: {
      id: 0,
      applied: 'unknown',
      description: 'Cannot get version.'
    }
  };

  const settings = {
    customSettings: {
      ...customSettings,
      mode: 'Developer',
    },
    collapsed: false,
    appVersion: appVersion,
  }

  // if (history.location.pathname.startsWith('/welcome')) {
  //   return {
  //     ...settings,
  //     collapsed: true,
  //   };
  // }

  return settings;
}

export function rootContainer(container: React.ReactNode): React.ReactNode {
  return (
    <Auth0Provider
      domain={AUTH0_DOMAIN}
      clientId={CLIENT_ID}
      authorizationParams={{
        redirect_uri: window.location.origin
      }}>
      {container}
    </Auth0Provider>
  );
};

// https://umijs.org/docs/max/layout-menu#%E8%BF%90%E8%A1%8C%E6%97%B6%E9%85%8D%E7%BD%AE
// https://pro-components.antdigital.dev/components/layout
export const layout: RuntimeConfig = (initialState: any) => {
  console.log("initialState: ", initialState);

  return {
    layout: 'top',
    logo: require('@/assets/logo-white.png'),
    title: '',
    fixedHeader: false,
    locale: 'en-US',
    rightContentRender: () => {
      return <Header username={getUsername()} />;
    },
    disableContentMargin: false,
    waterMarkProps: {
      // content: initialState?.currentUser?.name,
    },
    footerRender: () => <Footer />,
    onPageChange: () => {
      const { location } = history;
    },
    links: [],
    logout: () => {
      localStorage.removeItem('rapex-visitor-id');
      localStorage.removeItem('jwt_access_token');
      localStorage.removeItem('redirectUrl');
      history.push('/login');
    },
    // menuHeaderRender: false,
    childrenRender: (children: any, props: any) => {
      if (initialState?.loading) {
        return <PageLoading />;
      } else {
        return (
          <>
            {children}
          </>
        );
      }
    },
    ...initialState?.settings,
  };
};
