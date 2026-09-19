// admin 业务 API 封装
// 共享 API 来自 @erp-new-frontend-monorepo/api

export { getAlova } from '@erp-new-frontend-monorepo/boot';

// ==================== 从 packages/api 导入的共享 API ====================

export {
  // 工具函数
  api,
  normalize,
  unwrapArray,
  // Identity
  identityApi,
  impersonationApi,
  // Platform
  tenantsApi,
  billingApi,
  sessionsApi,
  webhooksApi,
  healthApi,
  // Catalog
  catalogApi,
  // Tickets
  ticketsApi,
  // Regulatory
  regulatoryApi,
  // Groups
  groupsApi,
} from '@erp-new-frontend-monorepo/api';

export type {
  // 基础类型
  PagedResult,
  // Identity
  GrantDto,
  IdentityUser,
  IdentityRole,
  PermissionCatalogEntry,
  CurrentUserDto,
  // Platform
  TenantDto,
  TenantStatusDto,
  ProvisioningDto,
  ThemeDto,
  ThemeUpdateDto,
  BillingPlanDto,
  SubscriptionDto,
  InvoiceDto,
  InvoiceDetailDto,
  UsageDto,
  GenerateInvoiceDto,
  CreatePlanDto,
  UpdatePlanDto,
  CreateSubscriptionDto,
  SessionDto,
  WebhookSubscriptionDto,
  WebhookDeliveryDto,
  HealthEntry,
  HealthResult,
  // Catalog
  BrandDto,
  CategoryDto,
  MoneyDto,
  ProductDto,
  CreateProductDto,
  UpdateProductDto,
  // Tickets
  TicketDto,
  CommentDto,
  CreateTicketDto,
  UpdateTicketDto,
  // Regulatory
  RegulatoryReceivePayload,
  RegulatoryFeedbackPayload,
  RegulatoryReceiveRecordItem,
  RegulatoryFeedbackRecordItem,
  RegulatoryExportRecordItem,
  RegulatoryExportRecordsResult,
  // Groups
  GroupItem,
  GroupMemberDto,
  CreateGroupResponse,
  DeleteResponse,
  AddMemberResponse,
  RemoveMemberResponse,
} from '@erp-new-frontend-monorepo/api';

// ==================== 兼容类型 ====================

/** @deprecated 使用 IdentityRole 替代 */
export interface AdminRole {
  id: string;
  name: string;
  description?: string;
  permissions?: string[];
}

export interface AdminNotification {
  id: string;
  type: string;
  title: string;
  body?: string | null;
  link?: string | null;
  source: string;
  metadataJson: string;
  readAtUtc?: string | null;
  createdAtUtc: string;
}
