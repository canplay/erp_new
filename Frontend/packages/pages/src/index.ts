/**
 * @file index.ts
 * @description Shared pages barrel export — only 7 components actually imported by apps
 */

export { default as AuditsList } from './audit/PermissionChangeLogPage.vue';
export { default as AuthLogin } from './LoginPage/Main.vue';
export { default as AuthRegister } from './LoginPage.vue';
export { default as BillingView } from './xlt/BillingRulesPage.vue';
export { default as FilesManager } from './FileManagerPage.vue';
export { default as OcrWorkbench } from './IndexPage.vue';
export { default as UsersView } from './user/UserListPage.vue';
