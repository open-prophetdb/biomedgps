import Footer from '@/components/Footer';
import Header from '@/components/Header';
import { ConfigProvider, Row } from 'antd';
import { RequestConfig, history, RuntimeConfig, request as UmiRequest, matchRoutes } from 'umi';
import { PageLoading, SettingDrawer } from '@ant-design/pro-components';
import { Auth0Provider } from '@auth0/auth0-react';
import { CustomSettings, AppVersion } from '../config/defaultSettings';
import { getJwtAccessToken, logout, logoutWithRedirect, getUsername, isAuthEnabled, isAuthenticated } from '@/components/util';

// import * as Sentry from "@sentry/react";

// Configure Sentry for error tracking
// Sentry.init({
//   dsn: "https://6b871a833586050acae9100637b200c6@o143851.ingest.us.sentry.io/4506958288846848",
//   integrations: [
//     Sentry.browserTracingIntegration(),
//     Sentry.replayIntegration({
//       maskAllText: false,
//       blockAllMedia: false,
//     }),
//   ],
//   // Performance Monitoring
//   tracesSampleRate: 1.0, //  Capture 100% of the transactions
//   // Set 'tracePropagationTargets' to control for which URLs distributed tracing should be enabled
//   tracePropagationTargets: ["localhost", /^https:\/\/drugs.3steps\.cn\/api/],
//   // Session Replay
//   replaysSessionSampleRate: 0.1, // This sets the sample rate at 10%. You may want to change it to 100% while in development and then sample at a lower rate in production.
//   replaysOnErrorSampleRate: 1.0, // If you're not already sampling the entire session, change the sample rate to 100% when sampling sessions where errors occur.
// });

// 运行时配置
// @ts-ignore
const publicPath = window.publicPath || process.env.PUBLIC_PATH || '/';
const defaultCustomSettings = {
  privacyPolicyUrl: `${publicPath}README/privacy_policy.md`,
  changeLogUrl: `${publicPath}README/changelog.md`,
  aboutUrl: `${publicPath}README/about.md`,
  helpUrl: `${publicPath}README/help.md`,
  downloadUrl: `${publicPath}README/download.md`,
  websiteTitle: '',
  websiteLogo: `${publicPath}logo-white.png`,
  websiteDescription: 'Network Medicine for Disease Mechanism and Treatment Based on AI and knowledge graph.',
  websiteKeywords: 'Network Medicine, MultiOmics Data, Treatment, AI, Knowledge Graph',
  defaultDataset: '000000',
  mode: 'Developer'
}

const isDev = process.env.NODE_ENV === 'development';
const version = process.env.UMI_APP_VERSION ? process.env.UMI_APP_VERSION : 'unknown';
const introPageEnabled = process.env.UMI_APP_INTRO_PAGE_ENABLED ? process.env.UMI_APP_INTRO_PAGE_ENABLED === 'true' : false;
const apiPrefix = process.env.UMI_APP_API_PREFIX ? process.env.UMI_APP_API_PREFIX : window.location.origin;
const CLIENT_ID = process.env.UMI_APP_AUTH0_CLIENT_ID ? process.env.UMI_APP_AUTH0_CLIENT_ID : '<your-client-id>';
const AUTH0_DOMAIN = process.env.UMI_APP_AUTH0_DOMAIN ? process.env.UMI_APP_AUTH0_DOMAIN : '<your-domain>';

console.log('apiPrefix', process.env, apiPrefix);

export const request: RequestConfig = {
  timeout: 120000,
  // More details on ./config/proxy.ts or ./config/config.cloud.ts
  baseURL: apiPrefix,
  errorConfig: {
    errorHandler: (resData) => {
      console.log("errorHandler: ", resData);

      // @ts-ignore
      if (resData.response && (resData.response.status === 401 || resData.response.status === 0)) {
        logoutWithRedirect();
      }

      return {
        ...resData,
        success: false,
        showType: 0,
        errorMessage: resData.message,
      };
    },
  },
  requestInterceptors: [(url: string, options) => {
    // How to get a jwt_access_token from the cookie?
    const jwt_access_token = getJwtAccessToken()

    let headers = {}

    headers = {
      "x-auth-users": getUsername(),
      // TODO: Support JWT
      "Authorization": "Bearer " + (jwt_access_token ? jwt_access_token : 'NOTOKEN')
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
        logoutWithRedirect();

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
  const component = <ConfigProvider theme={{
    token: {
      fontSize: 15
    }
  }}>
    {container}
  </ConfigProvider>;

  if (!isAuthEnabled()) {
    return component;
  }

  return (
    <Auth0Provider
      domain={AUTH0_DOMAIN}
      clientId={CLIENT_ID}
      authorizationParams={{
        redirect_uri: window.location.origin
      }}>
      {component}
    </Auth0Provider>
  );
};

// Gateway to validate the user's authentication status, If you want to ignore more paths, you can add them to the ignore list.
export function onRouteChange({ clientRoutes, location }: { clientRoutes: any, location: any }) {
  const route = matchRoutes(clientRoutes, location.pathname)?.pop()?.route;
  const ignoreList = ['/login', '/not-authorized', '/', '/privacy-policy', '/changelog', '/help'];
  console.log("isAuthenticated: ", isAuthenticated(), history.location.pathname, route?.path);
  if (!isAuthenticated() && !ignoreList.includes(route?.path || '')) {
    logoutWithRedirect();
  }
}

// https://umijs.org/docs/max/layout-menu#%E8%BF%90%E8%A1%8C%E6%97%B6%E9%85%8D%E7%BD%AE
// https://pro-components.antdigital.dev/components/layout
export const layout: RuntimeConfig = (initialState: any) => {
  console.log("initialState: ", initialState);
  const { location } = history;
  const isHomePage = location.pathname === '/';

  if (isHomePage && introPageEnabled) {
    return {
      headerRender: false,
      footerRender: () => <Footer />,
      menuRender: false,
      childrenRender: (children: any, props: any) => {
        return (
          <>
            {children}
          </>
        );
      }
    }
  }

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

      // You can modify the css style of the menu item at the global.less file.
      var spans = document.querySelectorAll('span.ant-pro-base-menu-horizontal-item-text');
      console.log("Add new-tag to ME/CFS: ", spans);
      spans.forEach(function (span) {
        console.log("span.innerHTML: ", span.innerHTML);
        if (span.innerHTML.startsWith("ME/CFS")) {
          span.classList.add('new-tag');
        }
      });
    },
    links: [],
    logout: () => {
      logout();
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
