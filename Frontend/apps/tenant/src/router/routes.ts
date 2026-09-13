/**
 * @file routes.ts
 * @description 路由配置
 * @date 2026-04-02
 */

import type { RouteRecordRaw } from 'vue-router';
import {
  PERM_USER_LIST, PERM_ROLE_LIST,
  PERM_PERMISSION_MANAGE, PERM_PERMISSION_LOG, PERM_DEPARTMENT_LIST,
  PERM_CONFIG_VIEW, PERM_ANNOUNCEMENT_LIST, PERM_LOG_LOGIN, PERM_LOG_OPERATION,
  PERM_AUDIT_SENSITIVE, PERM_DEVICE_LOGIN, PERM_WHITELIST_IP,
  PERM_MESSAGE_LIST, PERM_NOTIFICATION_TEMPLATE, PERM_DICTIONARY_LIST,
  PERM_FILE_MANAGE, PERM_MONITOR_VIEW, PERM_DATA_DASHBOARD, PERM_TASK_SCHEDULE,
  PERM_API_KEY_MANAGE, PERM_TENANT_MANAGE, PERM_FEEDBACK_LIST, PERM_THEME_SETTINGS,
  PERM_REPORT_MANAGE, PERM_DATA_SOURCE_MANAGE, PERM_CTP_DEVICE, PERM_LPR_MANAGE,
  PERM_XLT_MANAGE, PERM_TOW_MANAGE, PERM_EBIKE_MANAGE, PERM_CMS_ARTICLE,
  PERM_CMS_CATEGORY, PERM_WORKFLOW,
} from '@/constants/permissions';
import { createRouter, createWebHistory } from 'vue-router';

const routes: RouteRecordRaw[] = [
  // ============ 公共路由 ============
  {
    path: '/login',
    name: 'Login',
    component: () => import('@erp-new-frontend-monorepo/pages/src/LoginPage/Main.vue'),
    meta: { title: '登录', requiresAuth: false ,
        cap: 'auth:login'
      },
  },

  // ============ 主布局路由（需要登录） ============
  {
    path: '/',
    component: () => import('@/layouts/MainLayout/MainLayout.vue'),
    meta: { requiresAuth: true ,
        cap: 'dashboard:view'
      },
    children: [
      // 首页 - 仪表盘
      {
        path: '',
        name: 'Dashboard',
        component: () => import('@erp-new-frontend-monorepo/pages/src/DashboardPage.vue'),
        meta: { title: '仪表盘', icon: 'dashboard' },
      },

      // 用户管理
      {
        path: 'users',
        name: 'UserList',
        component: () => import('@erp-new-frontend-monorepo/pages/src/user/UserListPage.vue'),
        meta: { title: '用户管理', icon: 'people', permissions: [...PERM_USER_LIST] ,
        cap: 'user:list'
      },
      },
      {
        path: 'users/:id',
        name: 'UserDetail',
        component: () => import('@erp-new-frontend-monorepo/pages/src/user/UserDetailPage.vue'),
        meta: { title: '用户详情', icon: 'person' ,
        cap: 'user:detail'
      },
      },

      // 角色管理
      {
        path: 'roles',
        name: 'RoleList',
        component: () => import('@erp-new-frontend-monorepo/pages/src/role/RoleListPage.vue'),
        meta: { title: '角色管理', icon: 'admin_panel_settings', permissions: [...PERM_ROLE_LIST] ,
        cap: 'role:list'
      },
      },

      // 权限配置
      {
        path: 'permissions',
        name: 'RolePermission',
        component: () => import('@erp-new-frontend-monorepo/pages/src/RolePermissionPage.vue'),
        meta: { title: '权限配置', icon: 'security', permissions: [...PERM_PERMISSION_MANAGE] ,
        cap: 'permission:view'
      },
      },

      // 登录日志
      {
        path: 'logs/login',
        name: 'LoginLogs',
        component: () => import('@erp-new-frontend-monorepo/pages/src/log/LoginLogPage.vue'),
        meta: { title: '登录日志', icon: 'history', permissions: [...PERM_LOG_LOGIN] ,
        cap: 'log:login'
      },
      },

      // 个人中心
      {
        path: 'profile',
        name: 'Profile',
        component: () => import('@erp-new-frontend-monorepo/pages/src/ProfilePage.vue'),
        meta: { title: '个人中心', icon: 'account_circle' ,
        cap: 'profile:view'
      },
      },

      // 操作日志
      {
        path: 'logs/operation',
        name: 'OperationLogs',
        component: () => import('@erp-new-frontend-monorepo/pages/src/log/OperationLogPage.vue'),
        meta: { title: '操作日志', icon: 'history', permissions: [...PERM_LOG_OPERATION] ,
        cap: 'log:operation'
      },
      },

      // 系统设置
      {
        path: 'settings',
        name: 'SystemSettings',
        component: () => import('@erp-new-frontend-monorepo/pages/src/SystemSettingsPage.vue'),
        meta: { title: '系统设置', icon: 'settings', permissions: [...PERM_CONFIG_VIEW] ,
        cap: 'settings:view'
      },
      },

      // 消息通知
      {
        path: 'notifications',
        name: 'NotificationList',
        component: () => import('@erp-new-frontend-monorepo/pages/src/notification/NotificationListPage.vue'),
        meta: { title: '消息通知', icon: 'notifications' ,
        cap: 'notifications:view'
      },
      },

      // 数据字典
      {
        path: 'dictionaries',
        name: 'DictionaryList',
        component: () => import('@erp-new-frontend-monorepo/pages/src/dictionary/DictionaryTypeListPage.vue'),
        meta: { title: '数据字典', icon: 'folder', permissions: [...PERM_DICTIONARY_LIST] ,
        cap: 'dictionary:type:list'
      },
      },

      // 系统配置
      {
        path: 'system-config',
        name: 'SystemConfig',
        component: () => import('@erp-new-frontend-monorepo/pages/src/system/SystemConfigPage.vue'),
        meta: { title: '系统配置', icon: 'settings', permissions: [...PERM_CONFIG_VIEW] ,
        cap: 'system-config:view'
      },
      },

      // 公告管理
      {
        path: 'announcements',
        name: 'AnnouncementList',
        component: () => import('@erp-new-frontend-monorepo/pages/src/system/AnnouncementPage/Main.vue'),
        meta: { title: '公告管理', icon: 'campaign', permissions: [...PERM_ANNOUNCEMENT_LIST] ,
        cap: 'announcement:list'
      },
      },

      // 通知模板管理
      {
        path: 'notification-templates',
        name: 'NotificationTemplateList',
        component: () => import('@erp-new-frontend-monorepo/pages/src/NotificationTemplatePage.vue'),
        meta: { title: '通知模板', icon: 'notifications_active', permissions: [...PERM_NOTIFICATION_TEMPLATE] ,
        cap: 'notification-template:list'
      },
      },

      // 文件管理
      {
        path: 'files',
        name: 'FileManager',
        component: () => import('@erp-new-frontend-monorepo/pages/src/FileManagerPage.vue'),
        meta: { title: '文件管理', icon: 'folder', permissions: [...PERM_FILE_MANAGE] ,
        cap: 'files:view'
      },
      },

      // 主题设置
      {
        path: 'theme',
        name: 'ThemeSettings',
        component: () => import('@erp-new-frontend-monorepo/pages/src/ThemeSettingsPage.vue'),
        meta: { title: '主题设置', icon: 'palette', permissions: [...PERM_THEME_SETTINGS] ,
        cap: 'theme:switch'
      },
      },

      // 系统监控
      {
        path: 'monitor',
        name: 'SystemMonitor',
        component: () => import('@erp-new-frontend-monorepo/pages/src/SystemMonitorPage.vue'),
        meta: { title: '系统监控', icon: 'monitor_heart', permissions: [...PERM_MONITOR_VIEW] ,
        cap: 'monitor:view'
      },
      },

      // 数据统计大屏
      {
        path: 'data-dashboard',
        name: 'DataDashboard',
        component: () => import('@erp-new-frontend-monorepo/pages/src/DataDashboardPage.vue'),
        meta: { title: '数据统计', icon: 'bar_chart', permissions: [...PERM_DATA_DASHBOARD] ,
        cap: 'data-dashboard:view'
      },
      },

      // 定时任务调度
      {
        path: 'task-schedule',
        name: 'TaskSchedule',
        component: () => import('@erp-new-frontend-monorepo/pages/src/system/TaskSchedulePage.vue'),
        meta: { title: '任务调度', icon: 'schedule', permissions: [...PERM_TASK_SCHEDULE] ,
        cap: 'task-schedule:view'
      },
      },

      // API 密钥管理
      {
        path: 'api-keys',
        name: 'ApiKeyManagement',
        component: () => import('@erp-new-frontend-monorepo/pages/src/system/ApiKeyPage.vue'),
        meta: { title: 'API密钥', icon: 'vpn_key', permissions: [...PERM_API_KEY_MANAGE] ,
        cap: 'api-key:list'
      },
      },

      // 租户管理
      {
        path: 'tenant',
        name: 'TenantManagement',
        component: () => import('@erp-new-frontend-monorepo/pages/src/tenant/TenantManagementPage.vue'),
        meta: { title: '租户管理', icon: 'business', permissions: [...PERM_TENANT_MANAGE] ,
        cap: 'tenant:list'
      },
      },

      // 权限变更日志
      {
        path: 'permission-logs',
        name: 'PermissionChangeLog',
        component: () => import('@erp-new-frontend-monorepo/pages/src/audit/PermissionChangeLogPage.vue'),
        meta: { title: '权限变更日志', icon: 'history', permissions: [...PERM_PERMISSION_LOG] ,
        cap: 'permission-log:view'
      },
      },

      // ============ 新增功能路由 ============

      // 部门管理
      {
        path: 'departments',
        name: 'DepartmentList',
        component: () => import('@erp-new-frontend-monorepo/pages/src/department/DepartmentListPage.vue'),
        meta: { title: '部门管理', icon: 'corporate_fare', permissions: [...PERM_DEPARTMENT_LIST] ,
        cap: 'department:list'
      },
      },

      // 站内信/消息中心
      {
        path: 'messages',
        name: 'MessageCenter',
        component: () => import('@erp-new-frontend-monorepo/pages/src/message/MessageCenterPage.vue'),
        meta: { title: '消息中心', icon: 'mail', permissions: [...PERM_MESSAGE_LIST] ,
        cap: 'messages:view'
      },
      },

      // 意见反馈
      {
        path: 'feedback',
        name: 'FeedbackList',
        component: () => import('@erp-new-frontend-monorepo/pages/src/feedback/FeedbackListPage.vue'),
        meta: { title: '意见反馈', icon: 'feedback', permissions: [...PERM_FEEDBACK_LIST] ,
        cap: 'feedback:list'
      },
      },

      // 登录设备管理
      {
        path: 'devices',
        name: 'DeviceManagement',
        component: () => import('@erp-new-frontend-monorepo/pages/src/device/DeviceManagementPage.vue'),
        meta: { title: '登录设备', icon: 'devices', permissions: [...PERM_DEVICE_LOGIN] ,
        cap: 'device:list'
      },
      },

      // IP白名单
      {
        path: 'ip-whitelist',
        name: 'IpWhitelist',
        component: () => import('@erp-new-frontend-monorepo/pages/src/security/IpWhitelistPage.vue'),
        meta: { title: 'IP白名单', icon: 'vpn_lock', permissions: [...PERM_WHITELIST_IP] ,
        cap: 'ip-whitelist:view'
      },
      },

      // 敏感操作审计
      {
        path: 'sensitive-audit',
        name: 'SensitiveAudit',
        component: () => import('@erp-new-frontend-monorepo/pages/src/security/SensitiveAuditPage.vue'),
        meta: { title: '敏感操作审计', icon: 'security', permissions: [...PERM_AUDIT_SENSITIVE] ,
        cap: 'sensitive-audit:view'
      },
      },

      // CMS文章管理
      {
        path: 'cms/articles',
        name: 'ArticleList',
        component: () => import('@erp-new-frontend-monorepo/pages/src/cms/ArticleListPage.vue'),
        meta: { title: '文章管理', icon: 'article', permissions: [...PERM_CMS_ARTICLE] ,
        cap: 'cms:article:list'
      },
      },

      // CMS文章编辑
      {
        path: 'cms/article/edit/:id',
        name: 'ArticleEdit',
        component: () => import('@erp-new-frontend-monorepo/pages/src/cms/ArticleEditPage.vue'),
        meta: { title: '编辑文章', icon: 'edit', permissions: [...PERM_CMS_ARTICLE] ,
        cap: 'cms:article:edit'
      },
      },

      // CMS文章预览
      {
        path: 'cms/article/preview/:id',
        name: 'ArticlePreview',
        component: () => import('@erp-new-frontend-monorepo/pages/src/cms/ArticleEditPage.vue'),
        meta: { title: '预览文章', icon: 'visibility', permissions: [...PERM_CMS_ARTICLE] ,
        cap: 'cms:article:preview'
      },
      },

      // CMS分类管理
      {
        path: 'cms/categories',
        name: 'CategoryManagement',
        component: () => import('@erp-new-frontend-monorepo/pages/src/cms/CategoryPage.vue'),
        meta: { title: '分类管理', icon: 'category', permissions: [...PERM_CMS_CATEGORY] ,
        cap: 'cms:category:list'
      },
      },

      // ============ 工作流管理 ============
      {
        path: 'workflows',
        name: 'WorkflowList',
        component: () => import('@erp-new-frontend-monorepo/pages/src/workflow/WorkflowListPage.vue'),
        meta: { title: '工作流管理', icon: 'account_tree', permissions: [...PERM_WORKFLOW] ,
        cap: 'workflow:list'
      },
      },
      {
        path: 'workflow/:id/design',
        name: 'WorkflowDesign',
        component: () => import('@erp-new-frontend-monorepo/pages/src/workflow/WorkflowDesignPage.vue'),
        meta: { title: '工作流设计', icon: 'design_services', permissions: [...PERM_WORKFLOW] ,
        cap: 'workflow:design'
      },
      },
      {
        path: 'workflow/instances',
        name: 'WorkflowInstances',
        component: () => import('@erp-new-frontend-monorepo/pages/src/workflow/WorkflowInstancesPage.vue'),
        meta: { title: '流程实例', icon: 'playlist_play', permissions: [...PERM_WORKFLOW] ,
        cap: 'workflow:instance:list'
      },
      },
      {
        path: 'workflow/instances/:id',
        name: 'WorkflowInstanceDetail',
        component: () => import('@erp-new-frontend-monorepo/pages/src/workflow/WorkflowInstanceDetailPage.vue'),
        meta: { title: '实例详情', icon: 'info', permissions: [...PERM_WORKFLOW] ,
        cap: 'workflow:instance:detail'
      },
      },

      // ============ 报表管理 ============
      {
        path: 'reports',
        name: 'ReportList',
        component: () => import('@erp-new-frontend-monorepo/pages/src/report/ReportListPage.vue'),
        meta: { title: '报表管理', icon: 'assessment', permissions: [...PERM_REPORT_MANAGE] ,
        cap: 'report:list'
      },
      },
      {
        path: 'reports/:id/edit',
        name: 'ReportEdit',
        component: () => import('@erp-new-frontend-monorepo/pages/src/report/ReportEditPage.vue'),
        meta: { title: '编辑报表', icon: 'edit', permissions: [...PERM_REPORT_MANAGE] ,
        cap: 'report:edit'
      },
      },
      {
        path: 'reports/:id/view',
        name: 'ReportView',
        component: () => import('@erp-new-frontend-monorepo/pages/src/report/ReportViewPage.vue'),
        meta: { title: '查看报表', icon: 'visibility', permissions: [...PERM_REPORT_MANAGE] ,
        cap: 'report:view'
      },
      },
      {
        path: 'reports/templates',
        name: 'ReportTemplates',
        component: () => import('@erp-new-frontend-monorepo/pages/src/report/ReportTemplatesPage.vue'),
        meta: { title: '报表模板', icon: 'dashboard_customize', permissions: [...PERM_REPORT_MANAGE] ,
        cap: 'report:template:list'
      },
      },
      {
        path: 'reports/datasources',
        name: 'DataSourceManagement',
        component: () => import('@erp-new-frontend-monorepo/pages/src/report/DataSourceManagementPage.vue'),
        meta: { title: '数据源管理', icon: 'storage', permissions: [...PERM_DATA_SOURCE_MANAGE] ,
        cap: 'report:datasource:list'
      },
      },

      // ============ CTP 地锁管理 ============
      {
        path: 'ctp',
        name: 'CtpDeviceList',
        component: () => import('@erp-new-frontend-monorepo/pages/src/ctp/DeviceListPage.vue'),
        meta: { title: '地锁设备管理', icon: 'lock', permissions: [...PERM_CTP_DEVICE] ,
        cap: 'ctp:device:list'
      },
      },
      {
        path: 'ctp/device/:deviceNo',
        name: 'CtpDeviceDetail',
        component: () => import('@erp-new-frontend-monorepo/pages/src/ctp/DeviceDetailPage.vue'),
        meta: { title: '设备详情', icon: 'info' ,
        cap: 'ctp:device:detail'
      },
      },

      // ============ LPR 车牌识别 ============
      {
        path: 'lpr/records',
        name: 'LprRecords',
        component: () => import('@erp-new-frontend-monorepo/pages/src/lpr/PassRecordsPage.vue'),
        meta: { title: '通行记录', icon: 'directions_car', permissions: [...PERM_LPR_MANAGE] ,
        cap: 'lpr:records:list'
      },
      },
      {
        path: 'lpr/monitor',
        name: 'LprMonitor',
        component: () => import('@erp-new-frontend-monorepo/pages/src/lpr/LiveMonitorPage.vue'),
        meta: { title: '实时监控', icon: 'videocam', permissions: [...PERM_LPR_MANAGE] ,
        cap: 'lpr:monitor:view'
      },
      },
      {
        path: 'lpr/devices',
        name: 'LprDevices',
        component: () => import('@erp-new-frontend-monorepo/pages/src/lpr/LprDevicesPage.vue'),
        meta: { title: '识别设备', icon: 'security', permissions: [...PERM_LPR_MANAGE] ,
        cap: 'lpr:devices:list'
      },
      },

      // ============ XLT 停车管理 ============
      {
        path: 'xlt',
        name: 'XltDashboard',
        component: () => import('@erp-new-frontend-monorepo/pages/src/xlt/DashboardPage.vue'),
        meta: { title: '停车看板', icon: 'local_parking', permissions: [...PERM_XLT_MANAGE] ,
        cap: 'xlt/vehicles:list'
      },
      },
      {
        path: 'xlt/vehicles',
        name: 'XltVehicles',
        component: () => import('@erp-new-frontend-monorepo/pages/src/xlt/ParkingVehiclesPage.vue'),
        meta: { title: '在场车辆', icon: 'time_to_leave', permissions: [...PERM_XLT_MANAGE] ,
        cap: 'xlt/vehicles:list'
      },
      },
      {
        path: 'xlt/records',
        name: 'XltRecords',
        component: () => import('@erp-new-frontend-monorepo/pages/src/xlt/ParkingRecordsPage.vue'),
        meta: { title: '进出记录', icon: 'history', permissions: [...PERM_XLT_MANAGE] ,
        cap: 'xlt/records:list'
      },
      },
      {
        path: 'xlt/billing',
        name: 'XltBilling',
        component: () => import('@erp-new-frontend-monorepo/pages/src/xlt/BillingRulesPage.vue'),
        meta: { title: '计费规则', icon: 'receipt', permissions: [...PERM_XLT_MANAGE] ,
        cap: 'xlt/billing:rules:list'
      },
      },

      // ============ Tow 拖车管理 ============
      {
        path: 'tow',
        name: 'TowTaskList',
        component: () => import('@erp-new-frontend-monorepo/pages/src/tow/TowTaskListPage.vue'),
        meta: { title: '拖车任务', icon: 'local_shipping', permissions: [...PERM_TOW_MANAGE] ,
        cap: 'tow:tasks:list'
      },
      },
      {
        path: 'tow/car/:id',
        name: 'TowCarDetail',
        component: () => import('@erp-new-frontend-monorepo/pages/src/tow/CarArchivePage.vue'),
        meta: { title: '车辆档案', icon: 'description' ,
        cap: 'tow:cars:edit'
      },
      },
      {
        path: 'tow/config',
        name: 'TowConfig',
        component: () => import('@erp-new-frontend-monorepo/pages/src/tow/ConfigPage.vue'),
        meta: { title: '字典配置', icon: 'settings', permissions: [...PERM_TOW_MANAGE] ,
        cap: 'tow:config:edit'
      },
      },

      // ============ Ebike 共享单车监管 ============
      {
        path: 'ebike',
        name: 'EbikeIndex',
        component: () => import('@erp-new-frontend-monorepo/pages/src/ebike/IndexPage.vue'),
        meta: { title: '单车监管地图', icon: 'pedal_bike', permissions: [...PERM_EBIKE_MANAGE] ,
        cap: 'ebike:cars:list'
      },
      },
      {
        path: 'ebike/cars',
        name: 'EbikeCarList',
        component: () => import('@erp-new-frontend-monorepo/pages/src/ebike/CarPage.vue'),
        meta: { title: '车辆管理', icon: 'directions_bike', permissions: [...PERM_EBIKE_MANAGE] ,
        cap: 'ebike:cars:list'
      },
      },
      {
        path: 'ebike/storages',
        name: 'EbikeStorageList',
        component: () => import('@erp-new-frontend-monorepo/pages/src/ebike/StoragePage.vue'),
        meta: { title: '停放区管理', icon: 'place', permissions: [...PERM_EBIKE_MANAGE] ,
        cap: 'ebike/storages:list'
      },
      },
      {
        path: 'ebike/orders',
        name: 'EbikeOrderList',
        component: () => import('@erp-new-frontend-monorepo/pages/src/ebike/OrderPage.vue'),
        meta: { title: '订单管理', icon: 'receipt_long', permissions: [...PERM_EBIKE_MANAGE] ,
        cap: 'ebike/orders:list'
      },
      },
      {
        path: 'ebike/options',
        name: 'EbikeOptions',
        component: () => import('@erp-new-frontend-monorepo/pages/src/ebike/OptionsPage.vue'),
        meta: { title: '系统配置', icon: 'settings_suggest', permissions: [...PERM_EBIKE_MANAGE] ,
        cap: 'ebike/options:edit'
      },
      },
      {
        path: 'ebike/reports',
        name: 'EbikeReports',
        component: () => import('@erp-new-frontend-monorepo/pages/src/ebike/ReportsPage.vue'),
        meta: { title: '统计报表', icon: 'bar_chart', permissions: [...PERM_EBIKE_MANAGE] ,
        cap: 'ebike/reports:view'
      },
      },
    ],
  },

  // ============ 错误页面 ============
  {
    path: '/:catchAll(.*)*',
    component: () => import('@erp-new-frontend-monorepo/pages/src/ErrorNotFound.vue'),
  },
  {
    path: '/403',
    name: 'Forbidden',
    component: () => import('@erp-new-frontend-monorepo/pages/src/ErrorForbidden.vue'),
    meta: { title: '权限不足' },
  },
];

// 创建 router 实例
export const router = createRouter({
  history: createWebHistory(),
  routes,
});

export default router;
