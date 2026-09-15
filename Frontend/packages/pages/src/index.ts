/**
 * @file index.ts
 * @description Shared pages barrel export for erp_new frontend monorepo
 * @date 2026-09-15
 *
 * All shared page components from packages/pages/src/ are re-exported here
 * so that admin/tenant/ops/social apps can import via:
 *   import { SomePage } from '@erp-new-frontend-monorepo/pages';
 */

// ============ Admin-facing named components ============
// These are used by apps/admin/src/pages/* wrappers

export { default as AuditsList } from './audit/PermissionChangeLogPage.vue';
export { default as AuthLogin } from './LoginPage/Main.vue';
export { default as AuthRegister } from './LoginPage.vue';
export { default as BillingView } from './xlt/BillingRulesPage.vue';
export { default as FilesManager } from './FileManagerPage.vue';
export { default as OcrWorkbench } from './IndexPage.vue';
export { default as UsersView } from './user/UserListPage.vue';

// ============ All page components (PascalCase) ============

// Root-level pages
export { default as AnnouncementPage } from './AnnouncementPage.vue';
export { default as ApiStatistics } from './ApiStatistics.vue';
export { default as ChatPage } from './ChatPage.vue';
export { default as ContactPage } from './ContactPage.vue';
export { default as DashboardPage } from './DashboardPage.vue';
export { default as DataDashboardPage } from './DataDashboardPage.vue';
export { default as ErrorForbidden } from './ErrorForbidden.vue';
export { default as ErrorNotFound } from './ErrorNotFound.vue';
export { default as FileManagerPage } from './FileManagerPage.vue';
export { default as HealthDashboard } from './HealthDashboard.vue';
export { default as IndexPage } from './IndexPage.vue';
export { default as LoginForm } from './LoginForm.vue';
export { default as LoginHeader } from './LoginHeader.vue';
export { default as LoginPage } from './LoginPage.vue';
export { default as MomentsPage } from './MomentsPage.vue';
export { default as NotificationTemplatePage } from './NotificationTemplatePage.vue';
export { default as ProfilePage } from './ProfilePage.vue';
export { default as RolePermissionPage } from './RolePermissionPage.vue';
export { default as RoomListPage } from './RoomListPage.vue';
export { default as SecondPage } from './SecondPage.vue';
export { default as SettingsPage } from './SettingsPage.vue';
export { default as SystemMonitorPage } from './SystemMonitorPage.vue';
export { default as SystemSettingsPage } from './SystemSettingsPage.vue';
export { default as ThemeSettingsPage } from './ThemeSettingsPage.vue';

// accounts/
export { default as AccountDetailPage } from './accounts/AccountDetailPage.vue';
export { default as AccountListPage } from './accounts/AccountListPage.vue';

// ApiStatistics/
export { default as ApiStatisticsChartContainer } from './ApiStatistics/ChartContainer.vue';
export { default as ApiStatisticsMain } from './ApiStatistics/Main.vue';
export { default as ApiStatisticsStatCard } from './ApiStatistics/StatCard.vue';

// audit/
export { default as PermissionChangeLogPage } from './audit/PermissionChangeLogPage.vue';

// cms/
export { default as ArticleEditPage } from './cms/ArticleEditPage.vue';
export { default as ArticleListPage } from './cms/ArticleListPage.vue';
export { default as CategoryPage } from './cms/CategoryPage.vue';

// content/
export { default as ContentDetailPage } from './content/ContentDetailPage.vue';
export { default as ContentListPage } from './content/ContentListPage.vue';

// crawl/
export { default as CrawlHistoryPage } from './crawl/CrawlHistoryPage.vue';
export { default as SourceListPage } from './crawl/SourceListPage.vue';

// ctp/
export { default as CtpDeviceDetailPage } from './ctp/DeviceDetailPage.vue';
export { default as CtpDeviceListPage } from './ctp/DeviceListPage.vue';

// department/
export { default as DepartmentListPage } from './department/DepartmentListPage.vue';

// device/
export { default as AbnormalLoginPage } from './device/AbnormalLoginPage.vue';
export { default as DeviceManagementPage } from './device/DeviceManagementPage.vue';
export { default as LoginHistoryPage } from './device/LoginHistoryPage.vue';

// dictionary/
export { default as DictionaryTypeListPage } from './dictionary/DictionaryTypeListPage.vue';

// ebike/
export { default as EbikeCarPage } from './ebike/CarPage.vue';
export { default as EbikeIndexPage } from './ebike/IndexPage.vue';
export { default as EbikeOptionsPage } from './ebike/OptionsPage.vue';
export { default as EbikeOrderPage } from './ebike/OrderPage.vue';
export { default as EbikeReportsPage } from './ebike/ReportsPage.vue';
export { default as EbikeStoragePage } from './ebike/StoragePage.vue';

// feedback/
export { default as FeedbackListPage } from './feedback/FeedbackListPage.vue';
export { default as FeedbackStatisticsPage } from './feedback/FeedbackStatisticsPage.vue';

// insights/
export { default as InsightListPage } from './insights/InsightListPage.vue';

// log/
export { default as LoginLogPage } from './log/LoginLogPage.vue';
export { default as OperationLogPage } from './log/OperationLogPage.vue';

// LoginPage/
export { default as CaptchaWidget } from './LoginPage/CaptchaWidget.vue';
export { default as HeaderLogo } from './LoginPage/HeaderLogo.vue';
export { default as LoginPageCaptchaWidget } from './LoginPage/CaptchaWidget.vue';
export { default as LoginPageHeaderLogo } from './LoginPage/HeaderLogo.vue';
export { default as LoginPageLoginForm } from './LoginPage/LoginForm.vue';
export { default as LoginPageMain } from './LoginPage/Main.vue';
export { default as LoginPageMemoryWidget } from './LoginPage/MemoryWidget.vue';

// lpr/
export { default as LiveMonitorPage } from './lpr/LiveMonitorPage.vue';
export { default as LprDevicesPage } from './lpr/LprDevicesPage.vue';
export { default as PassRecordsPage } from './lpr/PassRecordsPage.vue';

// message/
export { default as MessageCenterPage } from './message/MessageCenterPage.vue';

// notification/
export { default as NotificationListPage } from './notification/NotificationListPage.vue';

// publish/
export { default as PublishTaskPage } from './publish/PublishTaskPage.vue';
export { default as PublishWizard } from './publish/PublishWizard.vue';
export { default as ScheduleListPage } from './publish/ScheduleListPage.vue';

// report/
export { default as DataSourceManagementPage } from './report/DataSourceManagementPage.vue';
export { default as ReportEditPage } from './report/ReportEditPage.vue';
export { default as ReportListPage } from './report/ReportListPage.vue';
export { default as ReportTemplatesPage } from './report/ReportTemplatesPage.vue';
export { default as ReportViewPage } from './report/ReportViewPage.vue';

// rewrite/
export { default as LLMProviderSettings } from './rewrite/LLMProviderSettings.vue';
export { default as RewriteEditorPage } from './rewrite/RewriteEditorPage.vue';
export { default as RewriteTaskPage } from './rewrite/RewriteTaskPage.vue';

// role/
export { default as RoleListPage } from './role/RoleListPage.vue';

// security/
export { default as IpWhitelistPage } from './security/IpWhitelistPage.vue';
export { default as SensitiveAuditPage } from './security/SensitiveAuditPage.vue';

// stats/
export { default as StatsDashboard } from './stats/StatsDashboard.vue';

// system/
export { default as SystemAnnouncementEditor } from './system/AnnouncementEditor.vue';
export { default as SystemAnnouncementList } from './system/AnnouncementList.vue';
export { default as SystemAnnouncementPage } from './system/AnnouncementPage.vue';
export { default as SystemApiKeyPage } from './system/ApiKeyPage.vue';
export { default as SystemConfigPage } from './system/SystemConfigPage.vue';
export { default as SystemTaskSchedulePage } from './system/TaskSchedulePage.vue';

// system/AnnouncementPage/
export { default as AnnouncementFormDialog } from './system/AnnouncementPage/AnnouncementFormDialog.vue';
export { default as AnnouncementTable } from './system/AnnouncementPage/AnnouncementTable.vue';
export { default as AnnouncementPageMain } from './system/AnnouncementPage/Main.vue';

// tenant/
export { default as TenantManagementPage } from './tenant/TenantManagementPage.vue';

// tow/
export { default as CarArchivePage } from './tow/CarArchivePage.vue';
export { default as ConfigPage } from './tow/ConfigPage.vue';
export { default as TowTaskListPage } from './tow/TowTaskListPage.vue';

// user/
export { default as UserDetailPage } from './user/UserDetailPage.vue';
export { default as UserListPage } from './user/UserListPage.vue';

// workflow/
export { default as WorkflowDesignPage } from './workflow/WorkflowDesignPage.vue';
export { default as WorkflowInstanceDetailPage } from './workflow/WorkflowInstanceDetailPage.vue';
export { default as WorkflowInstancesPage } from './workflow/WorkflowInstancesPage.vue';
export { default as WorkflowListPage } from './workflow/WorkflowListPage.vue';

// xlt/
export { default as XltBillingRulesPage } from './xlt/BillingRulesPage.vue';
export { default as XltDashboardPage } from './xlt/DashboardPage.vue';
export { default as XltParkingRecordsPage } from './xlt/ParkingRecordsPage.vue';
export { default as XltParkingVehiclesPage } from './xlt/ParkingVehiclesPage.vue';
