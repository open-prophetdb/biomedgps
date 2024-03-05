import { createElement } from "react";
import * as icons from "@ant-design/icons";
import type { MenuDataItem } from '@ant-design/pro-components';

export const routes = [
  {
    path: '/predict-model',
    name: 'Predict Drug/Target',
    icon: 'history',
    component: './ModelConfig',
    category: 'predict-model'
  },
  {
    path: '/knowledge-graph',
    name: 'Explain Your Results',
    icon: 'comment',
    component: './KnowledgeGraph',
    category: 'knowledge-graph'
  },
  {
    path: '/knowledge-graph-editor',
    name: 'knowledge-graph-editor',
    icon: 'link',
    hideInMenu: true,
    component: './KnowledgeGraphEditor',
    category: 'knowledge-graph'
  },
  {
    name: 'chatbot',
    icon: 'comment',
    path: '/chatbot',
    hideInMenu: true,
    component: './ChatBot',
  },
  {
    name: 'about',
    icon: 'info-circle',
    path: '/about',
    hideInMenu: true,
    component: './About',
  },
  {
    name: 'help',
    icon: 'question-circle',
    path: '/help',
    hideInMenu: true,
    component: './Help',
  },
  {
    name: 'changelog',
    icon: 'field-time',
    path: '/changelog',
    hideInMenu: true,
    component: './ChangeLog',
  },
  {
    name: 'not-authorized',
    hideInMenu: true,
    path: '/not-authorized',
    component: './NotAuthorized',
  },
  {
    path: '/',
    redirect: '/predict-model',
  },
  {
    component: './404',
  },
];

export const dynamicRoutesToUsableRoutes = (routes: MenuDataItem[]): MenuDataItem[] => {
  return routes.map(route => {
    // route 是后端返回的数据
    // item 是最终antd-pro需要数据
    const item: MenuDataItem = {
      ...route,
      exact: false,
    };

    // icon 匹配
    if (route?.icon) {
      // @ts-ignore
      item.icon = createElement(icons[route.icon as string]);
    }

    // 组件匹配, 因为后端菜单配置的时候只会返回当前菜单对应的组件标识，所以通过组件标识来匹配组件
    // if (route?.component) {
    //   item.component = Component[route.component || ""];
    //   // item.exact = true;
    // }

    // @ts-ignore
    if (route.routes && route.routes.length > 0) {
      // @ts-ignore
      item.routes = [
        // 如果有子路由那么肯定是要进行重定向的，重定向为第一个组件
        {
          path: item.path,
          // @ts-ignore
          redirect: route.routes[0].path,
          // exact: true
        },
        ...dynamicRoutesToUsableRoutes(route.routes),
      ];
      item.children = [
        {
          path: item.path,
          // @ts-ignore
          redirect: route.routes[0].path,
          // exact: true
        },
        ...dynamicRoutesToUsableRoutes(route.routes),
      ];
    }

    return item;
  });
}
