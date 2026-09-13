# ERP_New 后端 SaaS 完善计划

> 参考 sesms (FullStack Hero .NET) 的成熟 SaaS 架构，补齐 erp_new (Rust) 缺失的 SaaS 核心能力

---

## 一、现状对比分析

### 1.1 前端结构（已完成）

| 项 | sesms | erp_new | 状态 |
|---|---|---|---|
| Admin 端 | `apps/admin` (平台运营) | `apps/admin` (sesms 复制) | ✅ 已对齐 |
| Tenant 端 | `apps/tenant` (使用单位) | `apps/tenant` (原 admin 改名) | ✅ 已对齐 |
| Portal 端 | `apps/portal-h5` (现场人员) | ❌ 缺失 | 🔜 待建设 |
| 共享包 | 10 个 packages | 10 个 packages | ✅ 已对齐 |
| Capabilities | 能力开关系统 | ✅ 已有 | ✅ |
| i18n | 统一国际化 | ✅ 已有 | ✅ |

### 1.2 后端架构对比

| 维度 | sesms (.NET FSH) | erp_new (Rust) | 差距 |
|---|---|---|---|
| **模块数量** | 22 个业务模块 | 21 个微服务 | 数量接近 |
| **多租户** | Finbuckle + 完整生命周期 | tenant-core (5 文件) | 🔴 严重不足 |
| **计费系统** | Billing 模块 (201 文件) | ❌ 完全缺失 | 🔴 关键缺失 |
| **订阅管理** | Subscription + Plan + Invoice | ❌ 完全缺失 | 🔴 关键缺失 |
| **身份认证** | Identity (409 文件) | auth-core (4 文件) | 🔴 严重不足 |
| **租户配置** | Theme + Settings + Quota | ❌ 缺失 | 🟡 需要补齐 |
| **审计日志** | Auditing (175 文件) | audit-service | 🟡 需评估 |
| **文件管理** | Files (161 文件) | file-service | 🟡 需评估 |
| **通知系统** | Notifications (135 文件) | messaging-service | 🟡 需评估 |
| **工单系统** | Tickets (148 文件) | ❌ 缺失 | 🟡 可后续 |
| **搜索** | Search (127 文件) | ❌ 缺失 | 🟡 可后续 |

### 1.3 核心差距识别

```
sesms 有但 erp_new 没有的：
├── ❌ Billing 模块（计费计划、订阅、发票、用量快照）
├── ❌ Tenant Provisioning（租户创建/初始化/迁移）
├── ❌ Tenant Lifecycle（激活/暂停/过期/续期/宽限期）
├── ❌ Plan Management（套餐定义、功能配额、价格）
├── ❌ Invoice Management（发票生成、支付、对账）
├── ❌ Usage Tracking（用量采集、计量、上报）
├── ❌ Tenant Theme（租户品牌定制）
├── ❌ Tenant Settings（租户级配置）
├── ❌ Feature Flags（按套餐的功能开关）
└── ❌ Quota Enforcement（配额限制执行）
```

---

## 二、实施计划

### Phase 1: 多租户核心增强（2-3 周）

**目标**: 补齐 tenant-core 为生产级多租户基础

#### 1.1 扩展 `crates/tenant-core/`

```rust
// 新增文件结构
src/
├── context.rs          # 已有 - 租户上下文
├── isolation.rs        # 已有 - 数据隔离
├── middleware.rs       # 已有 - gRPC 中间件
├── tenant_id.rs        # 已有 - 租户 ID 类型
├── lifecycle.rs        # 🆕 租户生命周期管理
├── provisioning.rs     # 🆕 租户初始化流程
├── settings.rs         # 🆕 租户设置管理
├── quota.rs            # 🆕 配额定义与检查
├── feature_flags.rs    # 🆕 功能标志
├── events.rs           # 🆕 租户事件
└── error.rs            # 🆕 错误类型
```

#### 1.2 新增 `crates/billing-core/`

```rust
src/
├── lib.rs
├── models/
│   ├── plan.rs         # 计费计划
│   ├── subscription.rs # 订阅
│   ├── invoice.rs      # 发票
│   └── usage.rs        # 用量记录
├── pricing/
│   ├── calculator.rs   # 价格计算
│   └── tiers.rs        # 阶梯定价
├── billing_cycle.rs    # 计费周期
├── payment.rs          # 支付状态
└── error.rs
```

#### 1.3 数据库迁移

```sql
-- 新增表
CREATE TABLE plans (
    id UUID PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    price_monthly DECIMAL(10,2),
    price_yearly DECIMAL(10,2),
    features JSONB,          -- 功能配额
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE subscriptions (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    plan_id UUID NOT NULL REFERENCES plans(id),
    status VARCHAR(20) DEFAULT 'active', -- active/suspended/cancelled/expired
    current_period_start TIMESTAMPTZ,
    current_period_end TIMESTAMPTZ,
    cancel_at_period_end BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE invoices (
    id UUID PRIMARY KEY,
    subscription_id UUID REFERENCES subscriptions(id),
    tenant_id UUID NOT NULL,
    amount DECIMAL(10,2),
    currency VARCHAR(3) DEFAULT 'CNY',
    status VARCHAR(20) DEFAULT 'pending', -- pending/paid/failed/void
    issued_at TIMESTAMPTZ,
    due_at TIMESTAMPTZ,
    paid_at TIMESTAMPTZ
);

CREATE TABLE usage_records (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    metric VARCHAR(50) NOT NULL, -- api_calls/storage/users etc
    quantity DECIMAL(15,4),
    recorded_at TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE tenant_settings (
    tenant_id UUID PRIMARY KEY REFERENCES tenants(id),
    theme JSONB,
    settings JSONB,
    updated_at TIMESTAMPTZ DEFAULT now()
);
```

---

### Phase 2: 租户服务增强（2 周）

**目标**: 扩展 tenant-service 为完整的租户管理平台

#### 2.1 新增 gRPC 服务定义

```protobuf
// protos/tenant/v1/tenant.proto
service TenantService {
  // 基础 CRUD
  rpc CreateTenant(CreateTenantRequest) returns (Tenant);
  rpc GetTenant(GetTenantRequest) returns (Tenant);
  rpc UpdateTenant(UpdateTenantRequest) returns (Tenant);
  rpc ListTenants(ListTenantsRequest) returns (ListTenantsResponse);
  rpc DeleteTenant(DeleteTenantRequest) returns (google.protobuf.Empty);
  
  // 生命周期
  rpc ActivateTenant(ActivateTenantRequest) returns (Tenant);
  rpc SuspendTenant(SuspendTenantRequest) returns (Tenant);
  rpc ExpireTenant(ExpireTenantRequest) returns (Tenant);
  rpc RenewTenant(RenewTenantRequest) returns (Tenant);
  
  // 设置
  rpc GetTenantSettings(GetTenantSettingsRequest) returns (TenantSettings);
  rpc UpdateTenantSettings(UpdateTenantSettingsRequest) returns (TenantSettings);
  rpc GetTenantTheme(GetTenantThemeRequest) returns (TenantTheme);
  rpc UpdateTenantTheme(UpdateTenantThemeRequest) returns (TenantTheme);
  
  // 配额
  rpc CheckQuota(CheckQuotaRequest) returns (QuotaStatus);
  rpc ListQuotas(ListQuotasRequest) returns (QuotaList);
  
  // 功能标志
  rpc HasFeature(HasFeatureRequest) returns (FeatureStatus);
  rpc ListFeatures(ListFeaturesRequest) returns (FeatureList);
}
```

#### 2.2 新增租户事件

```rust
// tenant lifecycle events
pub enum TenantEvent {
    Created { tenant_id: Uuid, name: String },
    Activated { tenant_id: Uuid },
    Suspended { tenant_id: Uuid, reason: String },
    Expired { tenant_id: Uuid },
    Renewed { tenant_id: Uuid, until: DateTime<Utc> },
    PlanChanged { tenant_id: Uuid, from: Uuid, to: Uuid },
    SettingsUpdated { tenant_id: Uuid },
}
```

---

### Phase 3: 计费服务建设（3 周）

**目标**: 新建 billing-service，提供完整计费能力

#### 3.1 服务结构

```
services/billing-service/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── service.rs          # gRPC 服务实现
│   ├── plans.rs            # 计划管理
│   ├── subscriptions.rs    # 订阅管理
│   ├── invoices.rs         # 发票管理
│   ├── usage.rs            # 用量采集
│   ├── billing_cycle.rs    # 计费周期任务
│   └── events.rs           # 事件处理
├── migrations/
└── tests/
```

#### 3.2 核心功能

1. **计划管理**
   - 创建/编辑/删除计费计划
   - 设置功能配额 (API 调用次数、存储、用户数等)
   - 月付/年付价格

2. **订阅管理**
   - 租户订阅计划
   - 升级/降级/取消
   - 自动续期

3. **用量采集**
   - API 调用计量
   - 存储容量计量
   - 活跃用户计量
   - 定期上报

4. **发票生成**
   - 月度自动开票
   - 支付状态跟踪
   - 逾期提醒

---

### Phase 4: API Gateway 增强（1 周）

**目标**: 在网关层实现租户解析、配额检查、用量计量

#### 4.1 新增中间件

```rust
// api-gateway/src/middleware/
├── tenant_resolver.rs    # 从 JWT/Header 解析租户
├── quota_enforcement.rs  # 配额限制检查
├── usage_recorder.rs     # 用量记录
└── feature_gate.rs       # 功能开关检查
```

#### 4.2 请求流程

```
Client Request
    ↓
API Gateway
    ├── JWT 验证 (auth-core)
    ├── 租户解析 (tenant_resolver)
    ├── 租户状态检查 (active/suspended/expired)
    ├── 配额检查 (quota_enforcement)
    ├── 功能开关检查 (feature_gate)
    ├── 用量记录 (usage_recorder)
    ↓
Backend Service
```

---

### Phase 5: 管理后台前端对齐（2 周）

**目标**: 让新的 admin 前端（sesms 复制）能正确调用 erp_new 后端

#### 5.1 API 适配层

```typescript
// packages/api/src/tenant.ts
// 将 erp_new 的 gRPC/REST API 封装为前端友好的接口

export const tenantApi = {
  createTenant: (data) => api.Post('/api/v1/tenants', data),
  listTenants: (params) => api.Get('/api/v1/tenants', { params }),
  activateTenant: (id) => api.Post(`/api/v1/tenants/${id}/activate`),
  suspendTenant: (id, reason) => api.Post(`/api/v1/tenants/${id}/suspend`, { reason }),
  // ...
};
```

#### 5.2 新增页面

- 租户管理列表页
- 租户详情/编辑页
- 订阅管理页
- 计费计划管理页
- 发票列表页
- 租户设置页

---

## 三、优先级排序

| 优先级 | 阶段 | 预估工期 | 业务价值 |
|---|---|---|---|
| **P0** | Phase 1: 多租户核心增强 | 2-3 周 | 基础设施，阻塞其他 |
| **P0** | Phase 2: 租户服务增强 | 2 周 | 平台运营必需 |
| **P1** | Phase 3: 计费服务建设 | 3 周 | 商业化核心 |
| **P1** | Phase 4: API Gateway 增强 | 1 周 | 安全与控制 |
| **P2** | Phase 5: 管理后台前端对齐 | 2 周 | 运营效率 |

---

## 四、技术决策

### 4.1 Rust crate 组织

```
crates/
├── tenant-core      # 多租户核心（扩展）
├── billing-core     # 计费核心（新增）
├── quota-core       # 配额核心（新增）
├── auth-core        # 认证核心（扩展）
├── common           # 公共工具
├── grpc-core        # gRPC 基础设施
└── ...
```

### 4.2 数据库策略

- **方案 A (推荐)**: 共享数据库 + tenant_id 隔离（与现有架构一致）
- **方案 B**: 按租户分库（适合大客户，后续可选）

### 4.3 事件驱动

- 租户状态变更 → 发布事件 → 计费服务/通知服务订阅
- 使用现有 messaging-service 或新增专门的事件总线

---

## 五、风险与缓解

| 风险 | 影响 | 缓解措施 |
|---|---|---|
| Rust 开发速度慢于 .NET | 工期延长 | 优先核心路径，非关键功能可简化 |
| 现有服务耦合度高 | 改动波及面大 | 新增服务而非修改现有，渐进式迁移 |
| 前端 sesms 代码依赖后端 API | 接口不匹配 | 先对齐 API 契约，再适配实现 |
| 多租户数据隔离漏洞 | 数据安全 | 所有查询强制 tenant_id 过滤，自动化测试 |

---

## 六、验收标准

- [ ] 租户可创建/激活/暂停/过期/续期
- [ ] 支持至少 3 种计费计划（免费/标准/企业）
- [ ] 订阅可升级/降级/取消
- [ ] 月度发票自动生成
- [ ] API 调用配额可配置且生效
- [ ] 租户级功能开关可用
- [ ] Admin 前端可管理租户和订阅
- [ ] Tenant 前端仅能看到本单位数据
- [ ] 所有服务构建通过
- [ ] 集成测试覆盖核心 SaaS 流程

---

*计划制定日期: 2026-09-13*
*参考项目: sesms (FullStack Hero .NET)*
*目标项目: erp_new (Rust + Vue/Quasar)*
