/**
 * @file index.ts
 * @brief API 统一导出（规范化版本 - 避免命名冲突）
 * @date 2026-04-06
 * @note 所有导出均使用明确命名，避免 export * 导致的命名冲突
 */

// 用户管理 API（不重新导出 role/department/log，避免命名冲突）
export {
  getUserInfo,
  updateUserInfo,
  changePassword,
  updateAvatar,
  listUsers,
  getUser,
  createUser,
  updateUser,
  updateUserStatus,
  updateUserRole,
  resetUserPassword,
  deleteUser,
  batchUpdateUserRole,
  batchUpdateUserStatus,
  batchDeleteUsers,
  getUserImportTemplate,
  importUsers,
  exportUsers,
  downloadUserImportTemplate,
  getStatistics,
  listDepartments,
  getDepartmentTree,
  createDepartment,
  updateDepartment,
  deleteDepartment,
} from './user';
export type { UserStatus, RoleType } from './user';

// 角色管理 API - 使用别名避免冲突
export {
  listRoles as listAdminRoles,
  getRole as getAdminRole,
  createRole as createAdminRole,
  updateRole as updateAdminRole,
  deleteRole as deleteAdminRole,
  getRolePermissions as getAdminRolePermissions,
  setRolePermissions as updateAdminRolePermissions,
  getRoleUsers as getAdminRoleUsers,
} from './role';

// 部门管理 API - 使用别名避免冲突（user.ts 已导出基础接口）
export {
  getDepartment as getAdminDepartment,
  getDepartmentUsers as getAdminDepartmentUsers,
  moveDepartment as moveAdminDepartment,
} from './department';

// 日志管理 API - 使用别名避免冲突
export {
  listLoginLogs as listAdminLoginLogs,
  getLoginLog as getAdminLoginLog,
  exportLoginLogs,
  listOperationLogs as listAdminOperationLogs,
  getOperationLog as getAdminOperationLog,
  exportOperationLogs,
  clearLoginLogs,
  clearOperationLogs,
} from './log';

// 类型导出
export type { LoginLog, OperationLog } from './log';
// 类型导出（Department 和 User 统一从 user.ts 导出，避免与 department.ts 冲突）
export type { Department, User } from './user';

// 工作流 API - 函数
export {
  listWorkflows,
  getWorkflow,
  createWorkflow,
  updateWorkflow,
  deleteWorkflow,
  publishWorkflow,
  listInstances,
  getInstance,
  startInstance,
  executeAction,
  listTasks,
  completeTask,
  rejectTask,
  listNodes,
  createNode,
  updateNode,
  deleteNode,
  listEdges,
  createEdge,
  updateEdge,
  deleteEdge,
} from './workflow';

// 工作流 API - 类型/枚举
export type {
  WorkflowStatus,
  NodeType,
  NodeStatus,
  InstanceStatus,
  WorkflowAction,
  Position,
  Workflow,
  WorkflowNode,
  WorkflowEdge,
  WorkflowInstance,
  TaskRecord,
  CreateWorkflowParam,
  UpdateWorkflowParam,
  CreateNodeParam,
  UpdateNodeParam,
  CreateEdgeParam,
  StartInstanceParam,
  ExecuteActionParam,
  WorkflowListParam,
} from './workflow';

// 报表 API
export {
  listReports,
  getReport,
  createReport,
  updateReport,
  deleteReport,
  generateReport,
  downloadReport,
  listDataSources,
  createDataSource,
  updateDataSource,
  deleteDataSource,
  listTemplates,
  createFromTemplate,
  executeReport,
  exportReport,
  getReportData,
  listReportTasks,
  listReportHistory,
} from './report';
export type {
  ReportStatus,
  ReportTaskStatus,
  ReportType,
  DataSourceType,
  Report,
  ReportTask,
  DataSource,
  ReportTemplate,
  CreateReportParam,
  UpdateReportParam,
  ReportListParam,
} from './report';

// 定时任务 API
export {
  listScheduledTasks,
  getScheduledTask,
  createScheduledTask,
  updateScheduledTask,
  deleteScheduledTask,
  triggerScheduledTask,
  pauseScheduledTask,
  resumeScheduledTask,
} from './scheduled-task';
export type {
  ScheduledTaskStatus,
  ActionType,
  ScheduledTask,
  CreateScheduledTaskParam,
  UpdateScheduledTaskParam,
  ScheduledTaskListParam,
} from './scheduled-task';

// 导出任务 API - 函数
export {
  countExportTasks,
  listExportTasks,
  getExportTask,
  createExportTask,
  batchCreateExportTasks,
  cancelExportTask,
  deleteExportTask,
  downloadExportFile,
  getExportTaskStats,
  getExportTaskProgress,
  retryExportTask,
  exportUsers as exportUsersTask,
  exportLoginLogs as exportLoginLogsTask,
  exportOperationLogs as exportOperationLogsTask,
  exportAuditLogs,
  exportTaskApi,
} from './exportTask';
export type {
  ExportTaskStatus,
  ExportTaskFormat,
  ExportTask,
  ExportTaskQuery,
  ExportTaskCreateParams,
  BatchExportParams,
  ExportTaskStats,
} from './exportTask';
