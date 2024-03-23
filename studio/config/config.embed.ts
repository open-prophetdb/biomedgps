// https://umijs.org/config/
import { defineConfig } from 'umi';

// How to improve the loading performance: https://juejin.cn/post/7207743145998811173
export default defineConfig({
  outputPath: '../assets',
  publicPath: '/assets/',
  history: {
    type: 'hash',
  },
  // https://umijs.org/blog/code-splitting#%E4%BB%A3%E7%A0%81%E6%8B%86%E5%88%86%E6%8C%87%E5%8D%97 (It's similar with dynamicImport in umi 3.x)
  codeSplitting: {
    jsStrategy: 'depPerChunk'
  },
  esbuildMinifyIIFE: true,
  favicons: ['/assets/gene.png'],
  jsMinifier: 'terser',
  jsMinifierOptions: {
    
  },
  proxy: undefined,
  locale: {
    default: 'en-US',
    antd: true,
    title: true,
    baseNavigator: true,
    baseSeparator: '-',
  },
});