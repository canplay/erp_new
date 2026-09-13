/**
 * @file menu.ts
 * @description 统一菜单配置 - 菜单项定义的唯一数据源
 * @date 2026-08-19
 */

import type { RouteMeta, RouteRecordRaw } from 'vue-router';

export interface MenuItem {
  path: string;
  title: string;
  icon: string;
  permissions?: string[] | undefined;
  menuItem?: {
    name: string;
    label: string;
    path: string;
    icon: string;
    permission?: string[] | undefined;
  };
}

export interface MenuCategory {
  title: string;
  icon: string;
  items?: MenuItem[];
  children?: MenuCategory[];
}

// ========== 菜单配置 =========

export const MENU_CONFIG: MenuCategory[] = [
  {
    title: '系统概览',
    icon: 'dashboard',
    items: [
      {
        path: '/',
        title: '仪表盘',
        icon: 'dashboard',
        menuItem: { name: 'Dashboard', label: '仪表盘', path: '/', icon: 'dashboard', permission: [] },
      },
    ],
  },
  {
    title: '系统管理',
    icon: 'settings_applications',
    items: [
      {
        path: '/users',
        title: '用户管理',
        icon: 'people',
        permissions: ['admin'],
        menuItem: { name: 'UserList', label: '用户管理', path: '/users', icon: 'people', permission: ['admin'] },
      },
      {
        path: '/roles',
        title: '角色管理',
        icon: 'admin_panel_settings',
        permissions: ['admin'],
        menuItem: { name: 'RoleList', label: '角色管理', path: '/roles', icon: 'admin_panel_settings', permission: ['admin'] },
      },
      {
        path: '/permissions',
        title: '权限配置',
        icon: 'security',
        permissions: ['admin'],
        menuItem: { name: 'PermissionList', label: '权限配置', path: '/permissions', icon: 'security', permission: ['admin'] },
      },
      {
        path: '/departments',
        title: '部门管理',
        icon: 'corporate_fare',
        permissions: ['admin'],
        menuItem: { name: 'DepartmentList', label: '部门管理', path: '/departments', icon: 'corporate_fare', permission: ['admin'] },
      },
    ],
  },
  {
    title: '安全与审计',
    icon: 'security',
    items: [
      {
        path: '/login-logs',
        title: '登录日志',
        icon: 'visibility',
        permissions: ['admin'],
        menuItem: { name: 'LoginLogs', label: '登录日志', path: '/login-logs', icon: 'visibility', permission: ['admin'] },
      },
      {
        path: '/operation-logs',
        title: '操作日志',
        icon: 'history',
        permissions: ['admin'],
        menuItem: { name: 'OperationLogs', label: '操作日志', path: '/operation-logs', icon: 'history', permission: ['admin'] },
      },
      {
        path: '/sensitive-logs',
        title: '敏感操作日志',
        icon: 'warning',
        permissions: ['admin'],
        menuItem: { name: 'SensitiveLogs', label: '敏感操作日志', path: '/sensitive-logs', icon: 'warning', permission: ['admin'] },
      },
    ],
  },
  {
    title: '业务管理',
    icon: 'business',
    items: [
      {
        path: '/ebike/vehicles',
        title: '车辆管理',
        icon: 'bike',
        permissions: ['admin'],
        menuItem: { name: 'EbikeVehicleList', label: '车辆管理', path: '/ebike/vehicles', icon: 'bike', permission: ['admin'] },
      },
      {
        path: '/ebike/orders',
        title: '订单管理',
        icon: 'receipt_long',
        permissions: ['admin'],
        menuItem: { name: 'EbikeOrderList', label: '订单管理', path: '/ebike/orders', icon: 'receipt_long', permission: ['admin'] },
      },
      {
        path: '/reports',
        title: '报表中心',
        icon: 'assessment',
        permissions: ['admin'],
        menuItem: { name: 'ReportList', label: '报表中心', path: '/reports', icon: 'assessment', permission: ['admin'] },
      },
    ],
  },
  {
    title: '系统设置',
    icon: 'tune',
    items: [
      {
        path: '/config',
        title: '系统配置',
        icon: 'settings',
        permissions: ['admin'],
        menuItem: { name: 'ConfigList', label: '系统配置', path: '/config', icon: 'settings', permission: ['admin'] },
      },
      {
        path: '/theme',
        title: '主题设置',
        icon: 'palette',
        permissions: ['admin'],
        menuItem: { name: 'ThemeConfig', label: '主题设置', path: '/theme', icon: 'palette', permission: ['admin'] },
      },
    ],
  },
];

/**
 * 从路由中提取菜单项（用于动态菜单）
 */
export function extractMenuItemsFromRoutes(routes: RouteRecordRaw[]): MenuItem[] {
  const items: MenuItem[] = [];

  for (const route of routes) {
    const menuItem = (route.meta as RouteMeta)?.menuItem as MenuItem | undefined;
    if (menuItem) {
      items.push({
        ...menuItem,
        path: route.path,
        title: (route.meta as RouteMeta)?.title as string || menuItem.title,
        icon: (route.meta as RouteMeta)?.icon as string || menuItem.icon,
        permissions: (route.meta as RouteMeta)?.permissions,
      });
    }

    if (route.children) {
      for (const child of route.children) {
        const childMenuItem = (child.meta as RouteMeta)?.menuItem as MenuItem | undefined;
        if (childMenuItem) {
          items.push({
            ...childMenuItem,
            path: child.path,
            title: (child.meta as RouteMeta)?.title as string || childMenuItem.title,
            icon: (child.meta as RouteMeta)?.icon as string || childMenuItem.icon,
            permissions: (child.meta as RouteMeta)?.permissions,
          });
        }
      }
    }
  }

  return items;
}

export default MENU_CONFIG;