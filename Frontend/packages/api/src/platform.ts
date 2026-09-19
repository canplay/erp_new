// 复用共享 alova 封装（packages/api），不再重复定义 class M
import { api, normalize, unwrapArray } from '@erp-new-frontend-monorepo/api';

export interface TenantDto {
  id: string;
  name: string;
  adminEmail: string;
  isActive: boolean;
  expiresOnUtc: string;
  createdAtUtc: string;
}

export interface TenantStatusDto {
  id: string;
  isActive: boolean;
  expiresOnUtc: string;
  validUpto?: string;
  [key: string]: unknown;
}

export interface ProvisioningDto {
  tenantId: string;
  status: string;
  progress: number;
  message?: string;
}

export interface ThemeDto {
  primaryColor: string;
  logoUrl?: string | null;
  faviconUrl?: string | null;
  customCss?: string | null;
}

export interface ThemeUpdateDto {
  primaryColor?: string;
  logoUrl?: string | null;
  faviconUrl?: string | null;
  customCss?: string | null;
}

export interface BillingPlanDto {
  id: string;
  name: string;
  description: string;
  price: number;
  currency: string;
  interval: string;
  features: string[];
  isActive: boolean;
  maxUsers: number;
  maxDevices: number;
}

export interface SubscriptionDto {
  id: string;
  tenantId: string;
  planId: string;
  planName: string;
  status: string;
  currentPeriodStart: string;
  currentPeriodEnd: string;
  cancelAtPeriodEnd: boolean;
}

export interface InvoiceDto {
  id: string;
  subscriptionId: string;
  invoiceNumber: string;
  subtotalAmount: number;
  currency: string;
  status: string | number;
  createdAtUtc?: string;
  issuedAtUtc?: string;
}

export interface InvoiceDetailDto {
  id?: string;
  tenantId?: string;
  invoiceNumber?: string;
  periodYear?: number;
  periodMonth?: number;
  currency?: string;
  subtotalAmount?: number;
  status?: string | number;
  createdAtUtc?: string;
  issuedAtUtc?: string;
  dueAtUtc?: string;
  paidAtUtc?: string;
  notes?: string;
  lineItems?: Array<{
    id: string;
    kind?: string | number;
    description?: string;
    quantity?: number;
    unitPrice?: number;
    amount?: number;
  }>;
}

export interface UsageDto {
  tenantId: string;
  currentUsers: number;
  currentDevices: number;
  periodStart: string;
  periodEnd: string;
}

export interface GenerateInvoiceDto {
  subscriptionId: string;
  periodStart: string;
  periodEnd: string;
}

export interface CreatePlanDto {
  name: string;
  description: string;
  price: number;
  currency: string;
  interval: string;
  features: string[];
  maxUsers: number;
  maxDevices: number;
}

export interface UpdatePlanDto {
  name?: string;
  description?: string;
  price?: number;
  currency?: string;
  interval?: string;
  features?: string[];
  maxUsers?: number;
  maxDevices?: number;
  isActive?: boolean;
}

export interface CreateSubscriptionDto {
  tenantId: string;
  planId: string;
}

export interface SessionDto {
  id: string;
  userId: string;
  userName: string;
  ip: string;
  userAgent: string;
  device: string;
  location?: string;
  startedAt: string;
  lastActivityAt: string;
  isCurrent: boolean;
}

export interface WebhookSubscriptionDto {
  id: string;
  url: string;
  secret: string;
  eventTypes: string[];
  isActive: boolean;
  createdAtUtc: string;
}

export interface WebhookDeliveryDto {
  id: string;
  createdAtUtc?: string;
  deliveredAtUtc?: string;
  timestampUtc?: string;
  status?: string | number;
  requestBody?: string;
  request?: string;
  payload?: string;
}

export const tenantsApi = {
  list: () => api.Get<TenantDto[]>('/api/v1/tenants').then((r) => unwrapArray<TenantDto>(r)),
  create: (data: { name: string; id: string; adminEmail: string }) =>
    api.Post('/api/v1/tenants', data),
  toggle: (id: string, isActive: boolean) =>
    api.Post(`/api/v1/tenants/${id}/activation`, { isActive }),
  renew: (id: string) => api.Post(`/api/v1/tenants/${id}/renew`, {}),
  meStatus: () => api.Get<TenantStatusDto>('/api/v1/tenants/me/status'),
  tenantStatus: (id: string) => api.Get<TenantStatusDto>(`/api/v1/tenants/${id}/status`),
  adjustValidity: (id: string, data: { expiresOnUtc: string }) =>
    api.Post(`/api/v1/tenants/${id}/adjust-validity`, data),
  theme: () => api.Get<ThemeDto>('/api/v1/tenants/theme'),
  updateTheme: (data: ThemeUpdateDto) => api.Put('/api/v1/tenants/theme', data),
  resetTheme: () => api.Post('/api/v1/tenants/theme/reset', {}),
  provisioning: (tenantId: string) =>
    api.Get<ProvisioningDto>(`/api/v1/tenants/${tenantId}/provisioning`),
  retryProvisioning: (tenantId: string) =>
    api.Post(`/api/v1/tenants/${tenantId}/provisioning/retry`, {}),
  getMigrations: () => api.Get('/api/v1/tenants/migrations'),
};

export const billingApi = {
  plans: () =>
    api
      .Get<BillingPlanDto[]>('/api/v1/billing/plans', { includeInactive: true })
      .then((r) => unwrapArray<BillingPlanDto>(r)),
  subscriptions: () =>
    api
      .Get<SubscriptionDto[]>('/api/v1/billing/subscriptions')
      .then((r) => unwrapArray<SubscriptionDto>(r)),
  invoices: (page = 1, size = 20) =>
    api
      .Get<InvoiceDto[]>('/api/v1/billing/invoices', { pageNumber: page, pageSize: size })
      .then((r) => normalize<InvoiceDto>(r, page, size)),
  invoiceDetail: (id: string) => api.Get<InvoiceDetailDto>(`/api/v1/billing/invoices/${id}`),
  invoicePdf: (id: string) => api.Get(`/api/v1/billing/invoices/${id}/pdf`),
  usage: () => api.Get<UsageDto>('/api/v1/billing/usage'),
  captureUsage: () => api.Post('/api/v1/billing/usage/snapshots/capture', {}),
  generateInvoice: (data: GenerateInvoiceDto) =>
    api.Post('/api/v1/billing/invoices/generate', data),
  issueInvoice: (id: string) => api.Post(`/api/v1/billing/invoices/${id}/issue`, {}),
  payInvoice: (id: string) => api.Post(`/api/v1/billing/invoices/${id}/pay`, {}),
  voidInvoice: (id: string) => api.Post(`/api/v1/billing/invoices/${id}/void`, {}),
  createPlan: (data: CreatePlanDto) => api.Post('/api/v1/billing/plans', data),
  updatePlan: (id: string, data: UpdatePlanDto) => api.Put(`/api/v1/billing/plans/${id}`, data),
  createSubscription: (data: CreateSubscriptionDto) =>
    api.Post('/api/v1/billing/subscriptions', data),
};

export const sessionsApi = {
  all: (page = 1, size = 20) =>
    api
      .Get<SessionDto[]>('/api/v1/identity/sessions', { page, size })
      .then((r) => normalize<SessionDto>(r, page, size)),
  revoke: (id: string) => api.Delete(`/api/v1/identity/sessions/${id}`),
  revokeAll: () => api.Post('/api/v1/identity/sessions/revoke-all', {}),
};

export const webhooksApi = {
  subscriptions: (page = 1, size = 20) =>
    api
      .Get<WebhookSubscriptionDto[]>('/api/v1/webhooks/subscriptions', {
        pageNumber: page,
        pageSize: size,
      })
      .then((r) => normalize<WebhookSubscriptionDto>(r, page, size)),
  create: (data: { url: string; secret?: string; eventTypes?: string[] }) =>
    api.Post('/api/v1/webhooks/subscriptions', data),
  remove: (id: string) => api.Delete(`/api/v1/webhooks/subscriptions/${id}`),
  deliveries: (subscriptionId: string) =>
    api
      .Get<WebhookDeliveryDto[]>(`/api/v1/webhooks/subscriptions/${subscriptionId}/deliveries`)
      .then((r) => unwrapArray<WebhookDeliveryDto>(r)),
  test: (id: string) => api.Post(`/api/v1/webhooks/subscriptions/${id}/test`, {}),
};

export interface HealthEntry {
  name: string;
  status: string;
  description?: string | null;
  durationMs: number;
  details?: Record<string, unknown> | null;
}

export interface HealthResult {
  status: string;
  results: HealthEntry[];
}

export const healthApi = {
  ready: () => api.Get<HealthResult>('/health/ready'),
  live: () => api.Get<HealthResult>('/health/live'),
};
