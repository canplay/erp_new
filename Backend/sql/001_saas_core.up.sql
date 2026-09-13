-- Phase 1: 多租户核心增强 - 数据库迁移
-- 创建时间: 2026-09-13

-- ============================================
-- 1. 计费计划表
-- ============================================
CREATE TABLE IF NOT EXISTS plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    description TEXT,
    plan_type VARCHAR(20) NOT NULL DEFAULT 'free' CHECK (plan_type IN ('free', 'standard', 'enterprise', 'custom')),
    status VARCHAR(20) NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'active', 'archived')),
    price_monthly DECIMAL(10, 2) NOT NULL DEFAULT 0,
    price_yearly DECIMAL(10, 2) NOT NULL DEFAULT 0,
    currency VARCHAR(3) NOT NULL DEFAULT 'CNY',
    features JSONB NOT NULL DEFAULT '[]'::jsonb,
    quotas JSONB NOT NULL DEFAULT '{}'::jsonb,
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_public BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_plans_status ON plans(status);
CREATE INDEX idx_plans_type ON plans(plan_type);
CREATE INDEX idx_plans_public ON plans(is_public) WHERE is_public = true;

-- 插入默认套餐
INSERT INTO plans (name, plan_type, status, price_monthly, price_yearly, features, quotas, sort_order, is_public) VALUES
('Free', 'free', 'active', 0, 0, 
 '[{"name":"basic_chat","enabled":true},{"name":"file_sharing","enabled":true,"value":"10MB"}]'::jsonb,
 '{"max_users":5,"max_storage_gb":1,"monthly_api_calls":10000}'::jsonb, 1, true),

('Standard', 'standard', 'active', 99, 999,
 '[{"name":"advanced_chat","enabled":true},{"name":"voice_video","enabled":true},{"name":"file_sharing","enabled":true,"value":"100MB"}]'::jsonb,
 '{"max_users":50,"max_storage_gb":10,"monthly_api_calls":100000}'::jsonb, 2, true),

('Enterprise', 'enterprise', 'active', 499, 4999,
 '[{"name":"all_features","enabled":true},{"name":"priority_support","enabled":true},{"name":"custom_branding","enabled":true}]'::jsonb,
 '{"max_users":500,"max_storage_gb":100,"monthly_api_calls":1000000}'::jsonb, 3, true);

-- ============================================
-- 2. 订阅表
-- ============================================
CREATE TABLE IF NOT EXISTS subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    plan_id UUID NOT NULL REFERENCES plans(id),
    status VARCHAR(20) NOT NULL DEFAULT 'trialing' CHECK (status IN ('active', 'cancelled', 'expired', 'trialing', 'past_due', 'unpaid')),
    current_period_start TIMESTAMPTZ NOT NULL DEFAULT now(),
    current_period_end TIMESTAMPTZ NOT NULL DEFAULT now() + INTERVAL '14 days',
    cancel_at_period_end BOOLEAN NOT NULL DEFAULT false,
    canceled_at TIMESTAMPTZ,
    trial_end TIMESTAMPTZ,
    quantity INTEGER NOT NULL DEFAULT 1,
    unit_price DECIMAL(10, 2) NOT NULL DEFAULT 0,
    currency VARCHAR(3) NOT NULL DEFAULT 'CNY',
    next_billing_date TIMESTAMPTZ,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT fk_subscription_tenant FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE
);

CREATE INDEX idx_subscriptions_tenant ON subscriptions(tenant_id);
CREATE INDEX idx_subscriptions_status ON subscriptions(status);
CREATE INDEX idx_subscriptions_plan ON subscriptions(plan_id);
CREATE INDEX idx_subscriptions_next_billing ON subscriptions(next_billing_date) WHERE next_billing_date IS NOT NULL;

-- ============================================
-- 3. 发票表
-- ============================================
CREATE TABLE IF NOT EXISTS invoices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    invoice_number VARCHAR(50) NOT NULL UNIQUE,
    subscription_id UUID REFERENCES subscriptions(id),
    tenant_id UUID NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'draft' CHECK (status IN ('draft', 'pending', 'paid', 'failed', 'void', 'refunded')),
    subtotal DECIMAL(10, 2) NOT NULL DEFAULT 0,
    tax_amount DECIMAL(10, 2) NOT NULL DEFAULT 0,
    total DECIMAL(10, 2) NOT NULL DEFAULT 0,
    currency VARCHAR(3) NOT NULL DEFAULT 'CNY',
    period_start TIMESTAMPTZ NOT NULL,
    period_end TIMESTAMPTZ NOT NULL,
    issued_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    due_at TIMESTAMPTZ NOT NULL DEFAULT now() + INTERVAL '7 days',
    paid_at TIMESTAMPTZ,
    line_items JSONB NOT NULL DEFAULT '[]'::jsonb,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT fk_invoice_tenant FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE
);

CREATE INDEX idx_invoices_tenant ON invoices(tenant_id);
CREATE INDEX idx_invoices_status ON invoices(status);
CREATE INDEX idx_invoices_subscription ON invoices(subscription_id);
CREATE INDEX idx_invoices_due ON invoices(due_at) WHERE status = 'pending';

-- ============================================
-- 4. 用量记录表
-- ============================================
CREATE TABLE IF NOT EXISTS usage_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    metric VARCHAR(50) NOT NULL,
    quantity DECIMAL(15, 4) NOT NULL DEFAULT 0,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT fk_usage_tenant FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE
);

CREATE INDEX idx_usage_tenant_metric ON usage_records(tenant_id, metric);
CREATE INDEX idx_usage_recorded_at ON usage_records(recorded_at);
CREATE INDEX idx_usage_tenant_period ON usage_records(tenant_id, metric, recorded_at);

-- ============================================
-- 5. 租户设置表
-- ============================================
CREATE TABLE IF NOT EXISTS tenant_settings (
    tenant_id UUID PRIMARY KEY REFERENCES tenants(id) ON DELETE CASCADE,
    theme JSONB NOT NULL DEFAULT '{}'::jsonb,
    settings JSONB NOT NULL DEFAULT '{}'::jsonb,
    features JSONB NOT NULL DEFAULT '{}'::jsonb,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ============================================
-- 6. 租户生命周期事件表
-- ============================================
CREATE TABLE IF NOT EXISTS tenant_lifecycle_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    event_type VARCHAR(50) NOT NULL,
    from_state VARCHAR(30),
    to_state VARCHAR(30),
    reason TEXT,
    operator_id BIGINT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT fk_lifecycle_tenant FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE
);

CREATE INDEX idx_lifecycle_tenant ON tenant_lifecycle_events(tenant_id);
CREATE INDEX idx_lifecycle_type ON tenant_lifecycle_events(event_type);
CREATE INDEX idx_lifecycle_created ON tenant_lifecycle_events(created_at);

-- ============================================
-- 7. 租户迁移记录表
-- ============================================
CREATE TABLE IF NOT EXISTS tenant_migrations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    version VARCHAR(50) NOT NULL,
    name VARCHAR(200) NOT NULL,
    applied_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    execution_time_ms BIGINT NOT NULL DEFAULT 0,
    success BOOLEAN NOT NULL DEFAULT true,
    error TEXT,
    CONSTRAINT fk_migration_tenant FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE
);

CREATE INDEX idx_migrations_tenant ON tenant_migrations(tenant_id);
CREATE INDEX idx_migrations_version ON tenant_migrations(tenant_id, version);

-- ============================================
-- 8. 更新 tenants 表（添加新字段）
-- ============================================
ALTER TABLE tenants ADD COLUMN IF NOT EXISTS lifecycle_state VARCHAR(30) NOT NULL DEFAULT 'provisioning';
ALTER TABLE tenants ADD COLUMN IF NOT EXISTS plan_id UUID REFERENCES plans(id);
ALTER TABLE tenants ADD COLUMN IF NOT EXISTS subscription_id UUID;
ALTER TABLE tenants ADD COLUMN IF NOT EXISTS grace_period_end TIMESTAMPTZ;
ALTER TABLE tenants ADD COLUMN IF NOT EXISTS provisioned_at TIMESTAMPTZ;
ALTER TABLE tenants ADD COLUMN IF NOT EXISTS last_active_at TIMESTAMPTZ;

CREATE INDEX idx_tenants_lifecycle ON tenants(lifecycle_state);
CREATE INDEX idx_tenants_plan ON tenants(plan_id);
