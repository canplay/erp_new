/**
 * @file menuConfig.ts
 * @brief 菜单配置
 */

export interface MenuItem {
  id: string;
  icon?: string;
  label: string;
  route?: string;
  permission?: string;
  children?: MenuItem[];
  divider?: boolean;
}

export const menuConfig: MenuItem[] = [
  {
    id: 'dashboard',
    icon: 'dashboard',
    label: 'menu.dashboard',
    route: '/dashboard',
  },
  {
    id: 'user-management',
    icon: 'people',
    label: 'menu.userManagement',
    children: [
      {
        id: 'user-list',
        icon: 'list',
        label: 'user.title',
        route: '/users',
        permission: 'user:list',
      },
    ],
  },
  {
    id: 'permission-management',
    icon: 'admin_panel_settings',
    label: 'menu.permissionManagement',
    children: [
      {
        id: 'role-list',
        icon: 'shield',
        label: 'permission.roles',
        route: '/roles',
        permission: 'role:list',
      },
      {
        id: 'department-list',
        icon: 'corporate_fare',
        label: 'department.title',
        route: '/departments',
        permission: 'department:list',
      },
    ],
  },
  {
    id: 'system-logs',
    icon: 'history',
    label: 'menu.systemLogs',
    children: [
      {
        id: 'login-log',
        icon: 'login',
        label: 'log.loginLog',
        route: '/logs/login',
        permission: 'log:login',
      },
      {
        id: 'operation-log',
        icon: 'assignment',
        label: 'log.operationLog',
        route: '/logs/operation',
        permission: 'log:operation',
      },
    ],
  },
  {
    id: 'menu-divider-1',
    divider: true,
    label: '',
  },
  {
    id: 'system-settings',
    icon: 'settings',
    label: 'menu.systemSettings',
    children: [
      {
        id: 'system-config',
        icon: 'tune',
        label: 'menu.systemConfig',
        route: '/settings/config',
        permission: 'system:config',
      },
      {
        id: 'system-about',
        icon: 'info',
        label: 'menu.about',
        route: '/settings/about',
        permission: 'system:about',
      },
    ],
  },
];

export default menuConfig;
