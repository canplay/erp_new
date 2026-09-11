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
    component: () => import('@/pages/LoginPage/Main.vue'),
    meta: { title: '登录', requiresAuth: false },
  },

  // ============ 主布局路由（需要登录） ============
  {
    path: '/',
    component: () => import('@/layouts/MainLayout/MainLayout.vue'),
    meta: { requiresAuth: true },
    children: [
      // 首页 - 仪表盘
      {
        path: '',
        name: 'Dashboard',
        component: () => import('@/pages/DashboardPage.vue'),
        meta: { title: '仪表盘', icon: 'dashboard' },
      },

      // 用户管理
      {
        path: 'users',
        name: 'UserList',
        component: () => import('@/pages/user/UserListPage.vue'),
        meta: { title: '用户管理', icon: 'people', permissions: [...PERM_USER_LIST] },
      },
      {
        path: 'users/:id',
        name: 'UserDetail',
        component: () => import('@/pages/user/UserDetailPage.vue'),
        meta: { title: '用户详情', icon: 'person' },
      },

      // 角色管理
      {
        path: 'roles',
        name: 'RoleList',
        component: () => import('@/pages/role/RoleListPage.vue'),
        meta: { title: '角色管理', icon: 'admin_panel_settings', permissions: [...PERM_ROLE_LIST] },
      },

      // 权限配置
      {
        path: 'permissions',
        name: 'RolePermission',
        component: () => import('@/pages/RolePermissionPage.vue'),
        meta: { title: '权限配置', icon: 'security', permissions: [...PERM_PERMISSION_MANAGE] },
      },

      // 登录日志
      {
        path: 'logs/login',
        name: 'LoginLogs',
        component: () => import('@/pages/log/LoginLogPage.vue'),
        meta: { title: '登录日志', icon: 'history', permissions: [...PERM_LOG_LOGIN] },
      },

      // 个人中心
      {
        path: 'profile',
        name: 'Profile',
        component: () => import('@/pages/ProfilePage.vue'),
        meta: { title: '个人中心', icon: 'account_circle' },
      },

      // 操作日志
      {
        path: 'logs/operation',
        name: 'OperationLogs',
        component: () => import('@/pages/log/OperationLogPage.vue'),
        meta: { title: '操作日志', icon: 'history', permissions: [...PERM_LOG_OPERATION] },
      },

      // 系统设置
      {
        path: 'settings',
        name: 'SystemSettings',
        component: () => import('@/pages/SystemSettingsPage.vue'),
        meta: { title: '系统设置', icon: 'settings', permissions: [...PERM_CONFIG_VIEW] },
      },

      // 消息通知
      {
        path: 'notifications',
        name: 'NotificationList',
        component: () => import('@/pages/notification/NotificationListPage.vue'),
        meta: { title: '消息通知', icon: 'notifications' },
      },

      // 数据字典
      {
        path: 'dictionaries',
        name: 'DictionaryList',
        component: () => import('@/pages/dictionary/DictionaryTypeListPage.vue'),
        meta: { title: '数据字典', icon: 'folder', permissions: [...PERM_DICTIONARY_LIST] },
      },

      // 系统配置
      {
        path: 'system-config',
        name: 'SystemConfig',
        component: () => import('@/pages/system/SystemConfigPage.vue'),
        meta: { title: '系统配置', icon: 'settings', permissions: [...PERM_CONFIG_VIEW] },
      },

      // 公告管理
      {
        path: 'announcements',
        name: 'AnnouncementList',
        component: () => import('@/pages/system/AnnouncementPage/Main.vue'),
        meta: { title: '公告管理', icon: 'campaign', permissions: [...PERM_ANNOUNCEMENT_LIST] },
      },

      // 通知模板管理
      {
        path: 'notification-templates',
        name: 'NotificationTemplateList',
        component: () => import('@/pages/NotificationTemplatePage.vue'),
        meta: { title: '通知模板', icon: 'notifications_active', permissions: [...PERM_NOTIFICATION_TEMPLATE] },
      },

      // 文件管理
      {
        path: 'files',
        name: 'FileManager',
        component: () => import('@/pages/FileManagerPage.vue'),
        meta: { title: '文件管理', icon: 'folder', permissions: [...PERM_FILE_MANAGE] },
      },

      // 主题设置
      {
        path: 'theme',
        name: 'ThemeSettings',
        component: () => import('@/pages/ThemeSettingsPage.vue'),
        meta: { title: '主题设置', icon: 'palette', permissions: [...PERM_THEME_SETTINGS] },
      },

      // 系统监控
      {
        path: 'monitor',
        name: 'SystemMonitor',
        component: () => import('@/pages/SystemMonitorPage.vue'),
        meta: { title: '系统监控', icon: 'monitor_heart', permissions: [...PERM_MONITOR_VIEW] },
      },

      // 数据统计大屏
      {
        path: 'data-dashboard',
        name: 'DataDashboard',
        component: () => import('@/pages/DataDashboardPage.vue'),
        meta: { title: '数据统计', icon: 'bar_chart', permissions: [...PERM_DATA_DASHBOARD] },
      },

      // 定时任务调度
      {
        path: 'task-schedule',
        name: 'TaskSchedule',
        component: () => import('@/pages/system/TaskSchedulePage.vue'),
        meta: { title: '任务调度', icon: 'schedule', permissions: [...PERM_TASK_SCHEDULE] },
      },

      // API 密钥管理
      {
        path: 'api-keys',
        name: 'ApiKeyManagement',
        component: () => import('@/pages/system/ApiKeyPage.vue'),
        meta: { title: 'API密钥', icon: 'vpn_key', permissions: [...PERM_API_KEY_MANAGE] },
      },

      // 租户管理
      {
        path: 'tenant',
        name: 'TenantManagement',
        component: () => import('@/pages/tenant/TenantManagementPage.vue'),
        meta: { title: '租户管理', icon: 'business', permissions: [...PERM_TENANT_MANAGE] },
      },

      // 权限变更日志
      {
        path: 'permission-logs',
        name: 'PermissionChangeLog',
        component: () => import('@/pages/audit/PermissionChangeLogPage.vue'),
        meta: { title: '权限变更日志', icon: 'history', permissions: [...PERM_PERMISSION_LOG] },
      },

      // ============ 新增功能路由 ============

      // 部门管理
      {
        path: 'departments',
        name: 'DepartmentList',
        component: () => import('@/pages/department/DepartmentListPage.vue'),
        meta: { title: '部门管理', icon: 'corporate_fare', permissions: [...PERM_DEPARTMENT_LIST] },
      },

      // 站内信/消息中心
      {
        path: 'messages',
        name: 'MessageCenter',
        component: () => import('@/pages/message/MessageCenterPage.vue'),
        meta: { title: '消息中心', icon: 'mail', permissions: [...PERM_MESSAGE_LIST] },
      },

      // 意见反馈
      {
        path: 'feedback',
        name: 'FeedbackList',
        component: () => import('@/pages/feedback/FeedbackListPage.vue'),
        meta: { title: '意见反馈', icon: 'feedback', permissions: [...PERM_FEEDBACK_LIST] },
      },

      // 登录设备管理
      {
        path: 'devices',
        name: 'DeviceManagement',
        component: () => import('@/pages/device/DeviceManagementPage.vue'),
        meta: { title: '登录设备', icon: 'devices', permissions: [...PERM_DEVICE_LOGIN] },
      },

      // IP白名单
      {
        path: 'ip-whitelist',
        name: 'IpWhitelist',
        component: () => import('@/pages/security/IpWhitelistPage.vue'),
        meta: { title: 'IP白名单', icon: 'vpn_lock', permissions: [...PERM_WHITELIST_IP] },
      },

      // 敏感操作审计
      {
        path: 'sensitive-audit',
        name: 'SensitiveAudit',
        component: () => import('@/pages/security/SensitiveAuditPage.vue'),
        meta: { title: '敏感操作审计', icon: 'security', permissions: [...PERM_AUDIT_SENSITIVE] },
      },

      // CMS文章管理
      {
        path: 'cms/articles',
        name: 'ArticleList',
        component: () => import('@/pages/cms/ArticleListPage.vue'),
        meta: { title: '文章管理', icon: 'article', permissions: [...PERM_CMS_ARTICLE] },
      },

      // CMS文章编辑
      {
        path: 'cms/article/edit/:id',
        name: 'ArticleEdit',
        component: () => import('@/pages/cms/ArticleEditPage.vue'),
        meta: { title: '编辑文章', icon: 'edit', permissions: [...PERM_CMS_ARTICLE] },
      },

      // CMS文章预览
      {
        path: 'cms/article/preview/:id',
        name: 'ArticlePreview',
        component: () => import('@/pages/cms/ArticleEditPage.vue'),
        meta: { title: '预览文章', icon: 'visibility', permissions: [...PERM_CMS_ARTICLE] },
      },

      // CMS分类管理
      {
        path: 'cms/categories',
        name: 'CategoryManagement',
        component: () => import('@/pages/cms/CategoryPage.vue'),
        meta: { title: '分类管理', icon: 'category', permissions: [...PERM_CMS_CATEGORY] },
      },

      // ============ 工作流管理 ============
      {
        path: 'workflows',
        name: 'WorkflowList',
        component: () => import('@/pages/workflow/WorkflowListPage.vue'),
        meta: { title: '工作流管理', icon: 'account_tree', permissions: [...PERM_WORKFLOW] },
      },
      {
        path: 'workflow/:id/design',
        name: 'WorkflowDesign',
        component: () => import('@/pages/workflow/WorkflowDesignPage.vue'),
        meta: { title: '工作流设计', icon: 'design_services', permissions: [...PERM_WORKFLOW] },
      },
      {
        path: 'workflow/instances',
        name: 'WorkflowInstances',
        component: () => import('@/pages/workflow/WorkflowInstancesPage.vue'),
        meta: { title: '流程实例', icon: 'playlist_play', permissions: [...PERM_WORKFLOW] },
      },
      {
        path: 'workflow/instances/:id',
        name: 'WorkflowInstanceDetail',
        component: () => import('@/pages/workflow/WorkflowInstanceDetailPage.vue'),
        meta: { title: '实例详情', icon: 'info', permissions: [...PERM_WORKFLOW] },
      },

      // ============ 报表管理 ============
      {
        path: 'reports',
        name: 'ReportList',
        component: () => import('@/pages/report/ReportListPage.vue'),
        meta: { title: '报表管理', icon: 'assessment', permissions: [...PERM_REPORT_MANAGE] },
      },
      {
        path: 'reports/:id/edit',
        name: 'ReportEdit',
        component: () => import('@/pages/report/ReportEditPage.vue'),
        meta: { title: '编辑报表', icon: 'edit', permissions: [...PERM_REPORT_MANAGE] },
      },
      {
        path: 'reports/:id/view',
        name: 'ReportView',
        component: () => import('@/pages/report/ReportViewPage.vue'),
        meta: { title: '查看报表', icon: 'visibility', permissions: [...PERM_REPORT_MANAGE] },
      },
      {
        path: 'reports/templates',
        name: 'ReportTemplates',
        component: () => import('@/pages/report/ReportTemplatesPage.vue'),
        meta: { title: '报表模板', icon: 'dashboard_customize', permissions: [...PERM_REPORT_MANAGE] },
      },
      {
        path: 'reports/datasources',
        name: 'DataSourceManagement',
        component: () => import('@/pages/report/DataSourceManagementPage.vue'),
        meta: { title: '数据源管理', icon: 'storage', permissions: [...PERM_DATA_SOURCE_MANAGE] },
      },

      // ============ CTP 地锁管理 ============
      {
        path: 'ctp',
        name: 'CtpDeviceList',
        component: () => import('@/pages/ctp/DeviceListPage.vue'),
        meta: { title: '地锁设备管理', icon: 'lock', permissions: [...PERM_CTP_DEVICE] },
      },
      {
        path: 'ctp/device/:deviceNo',
        name: 'CtpDeviceDetail',
        component: () => import('@/pages/ctp/DeviceDetailPage.vue'),
        meta: { title: '设备详情', icon: 'info' },
      },

      // ============ LPR 车牌识别 ============
      {
        path: 'lpr/records',
        name: 'LprRecords',
        component: () => import('@/pages/lpr/PassRecordsPage.vue'),
        meta: { title: '通行记录', icon: 'directions_car', permissions: [...PERM_LPR_MANAGE] },
      },
      {
        path: 'lpr/monitor',
        name: 'LprMonitor',
        component: () => import('@/pages/lpr/LiveMonitorPage.vue'),
        meta: { title: '实时监控', icon: 'videocam', permissions: [...PERM_LPR_MANAGE] },
      },
      {
        path: 'lpr/devices',
        name: 'LprDevices',
        component: () => import('@/pages/lpr/LprDevicesPage.vue'),
        meta: { title: '识别设备', icon: 'security', permissions: [...PERM_LPR_MANAGE] },
      },

      // ============ XLT 停车管理 ============
      {
        path: 'xlt',
        name: 'XltDashboard',
        component: () => import('@/pages/xlt/DashboardPage.vue'),
        meta: { title: '停车看板', icon: 'local_parking', permissions: [...PERM_XLT_MANAGE] },
      },
      {
        path: 'xlt/vehicles',
        name: 'XltVehicles',
        component: () => import('@/pages/xlt/ParkingVehiclesPage.vue'),
        meta: { title: '在场车辆', icon: 'time_to_leave', permissions: [...PERM_XLT_MANAGE] },
      },
      {
        path: 'xlt/records',
        name: 'XltRecords',
        component: () => import('@/pages/xlt/ParkingRecordsPage.vue'),
        meta: { title: '进出记录', icon: 'history', permissions: [...PERM_XLT_MANAGE] },
      },
      {
        path: 'xlt/billing',
        name: 'XltBilling',
        component: () => import('@/pages/xlt/BillingRulesPage.vue'),
        meta: { title: '计费规则', icon: 'receipt', permissions: [...PERM_XLT_MANAGE] },
      },

      // ============ Tow 拖车管理 ============
      {
        path: 'tow',
        name: 'TowTaskList',
        component: () => import('@/pages/tow/TowTaskListPage.vue'),
        meta: { title: '拖车任务', icon: 'local_shipping', permissions: [...PERM_TOW_MANAGE] },
      },
      {
        path: 'tow/car/:id',
        name: 'TowCarDetail',
        component: () => import('@/pages/tow/CarArchivePage.vue'),
        meta: { title: '车辆档案', icon: 'description' },
      },
      {
        path: 'tow/config',
        name: 'TowConfig',
        component: () => import('@/pages/tow/ConfigPage.vue'),
        meta: { title: '字典配置', icon: 'settings', permissions: [...PERM_TOW_MANAGE] },
      },

      // ============ Ebike 共享单车监管 ============
      {
        path: 'ebike',
        name: 'EbikeIndex',
        component: () => import('@/pages/ebike/IndexPage.vue'),
        meta: { title: '单车监管地图', icon: 'pedal_bike', permissions: [...PERM_EBIKE_MANAGE] },
      },
      {
        path: 'ebike/cars',
        name: 'EbikeCarList',
        component: () => import('@/pages/ebike/CarPage.vue'),
        meta: { title: '车辆管理', icon: 'directions_bike', permissions: [...PERM_EBIKE_MANAGE] },
      },
      {
        path: 'ebike/storages',
        name: 'EbikeStorageList',
        component: () => import('@/pages/ebike/StoragePage.vue'),
        meta: { title: '停放区管理', icon: 'place', permissions: [...PERM_EBIKE_MANAGE] },
      },
      {
        path: 'ebike/orders',
        name: 'EbikeOrderList',
        component: () => import('@/pages/ebike/OrderPage.vue'),
        meta: { title: '订单管理', icon: 'receipt_long', permissions: [...PERM_EBIKE_MANAGE] },
      },
      {
        path: 'ebike/options',
        name: 'EbikeOptions',
        component: () => import('@/pages/ebike/OptionsPage.vue'),
        meta: { title: '系统配置', icon: 'settings_suggest', permissions: [...PERM_EBIKE_MANAGE] },
      },
      {
        path: 'ebike/reports',
        name: 'EbikeReports',
        component: () => import('@/pages/ebike/ReportsPage.vue'),
        meta: { title: '统计报表', icon: 'bar_chart', permissions: [...PERM_EBIKE_MANAGE] },
      },
    ],
  },

  // ============ 错误页面 ============
  {
    path: '/:catchAll(.*)*',
    component: () => import('@/pages/ErrorNotFound.vue'),
  },
  {
    path: '/403',
    name: 'Forbidden',
    component: () => import('@/pages/ErrorForbidden.vue'),
    meta: { title: '权限不足' },
  },
];

// 创建 router 实例
export const router = createRouter({
  history: createWebHistory(),
  routes,
});

export default router;
