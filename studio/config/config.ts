// https://umijs.org/config/
import { defineConfig } from 'umi';
import path from 'path';
import proxy from './proxy';
import CompressionPlugin from 'compression-webpack-plugin';
import { routes as defaultRoutes } from './routes';

// const isDev = process.env.NODE_ENV === 'development';
// const isStatic = isDev ? true : (process.env.UMI_APP_IS_STATIC ? process.env.UMI_APP_IS_STATIC : false);

export default defineConfig({
  hash: true,
  history: {
    type: 'hash',
  },
  metas: [{ name: 'title', content: "Network Medicine Platform" }, { name: 'description', content: "Network Medicine for Disease Mechanism and Treatment Based on AI and knowledge graph." }, { name: 'keywords', content: "Network Medicine, MultiOmics Data, Treatment, AI, Knowledge Graph" }],
  favicons: [
    "/assets/gene.png",
  ],
  publicPath: '/',
  antd: {},
  access: {},
  model: {},
  initialState: {},
  request: {},
  npmClient: 'yarn',
  dva: {},
  chainWebpack: (config: any, { env }) => {
    config.merge({
      resolve: {
        fallback: {
          'perf_hooks': false,
        }
      }
    });

    // https://github.com/webpack/webpack/discussions/13585
    config.resolve.alias.set('perf_hooks', path.resolve(__dirname, 'perf_hooks.ts'));
    // console.log("config.resolve.alias", config.resolve.alias);

    if (env === 'production') {
      config.plugin('compression-webpack-plugin').use(
        new CompressionPlugin({
          test: /.js$|.html$|.css$/,
          threshold: 10240,
          deleteOriginalAssets: false,
        }),
      );
    }
  },
  layout: {
    // https://umijs.org/docs/max/layout-menu
    locale: false,
  },
  locale: {
    default: 'en-US',
    antd: true,
    title: true,
    baseNavigator: true,
    baseSeparator: '-',
  },
  targets: {
    chrome: 80
  },
  // umi routes: https://umijs.org/docs/routing
  // We load routes dynamically from the config file.
  routes: defaultRoutes,
  // Theme for antd: https://ant.design/docs/react/customize-theme-cn
  theme: {
    // 如果不想要 configProvide 动态设置主题需要把这个设置为 default
    // 只有设置为 variable， 才能使用 configProvide 动态设置主色调
    // https://ant.design/docs/react/customize-theme-variable-cn
    'root-entry-name': 'variable',
  },
  title: "Network Medicine Platform",
  ignoreMomentLocale: true,
  // proxy: proxy[REACT_APP_ENV || 'dev'],
  proxy: proxy['dev'],
  manifest: {
    basePath: '/',
  },
  // Fast Refresh 热更新
  fastRefresh: true,
  // https://pro.ant.design/docs/openapi
  // https://github.com/ant-design/ant-design-pro/blob/753945ec3d561a81851c3d6861b365f0a837c711/config/config.ts#L137
  presets: [require.resolve('umi-presets-pro')],
  openAPI: [
    {
      namespace: 'swagger',
      requestLibPath: "import { request } from 'umi'",
      // schemaPath: join(__dirname, 'api.json'),
      // You may need to open the apifox before running `yarn openapi`.
      // schemaPath: "http://127.0.0.1:4523/export/openapi?projectId=1645899&version=3.1",
      // TODO: ApiFox cannot import the spec correctly.
      schemaPath: "http://localhost:8000/spec",
      projectName: "swagger",
      mock: false,
    }
  ],
  mfsu: {},
  scripts: []
});
