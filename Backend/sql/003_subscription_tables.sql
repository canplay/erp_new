-- =============================================================================
-- MyAI Backend Database Schema - Subscription Billing
-- =============================================================================
-- 订阅计费数据库表结构
-- Subscription billing database schema
--
-- 包含:
--   - subscription_plans  订阅计划表
--   - subscriptions       租户订阅表
--   - invoices            发票主表
--   - invoice_items       发票明细表
--
-- 所有 CREATE TABLE 使用 IF NOT EXISTS，支持幂等执行
-- =============================================================================

-- =============================================================================
-- 订阅计划表 (subscription_plans)
-- =============================================================================
CREATE TABLE IF NOT EXISTS subscription_plans (
    id              VARCHAR(64) PRIMARY KEY,
    name            VARCHAR(128) NOT NULL,
    description     TEXT,
    price_cents     BIGINT NOT NULL DEFAULT 0,
    currency        VARCHAR(8) NOT NULL DEFAULT 'CNY',
    interval        VARCHAR(16) NOT NULL DEFAULT 'month',  -- 'month', 'year'
    interval_count  INTEGER NOT NULL DEFAULT 1,
    trial_days      INTEGER NOT NULL DEFAULT 0,
    active          BOOLEAN NOT NULL DEFAULT TRUE,
    features        JSONB DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

COMMENT ON TABLE subscription_plans IS '订阅计划表';
COMMENT ON COLUMN subscription_plans.id IS '计划唯一标识 (如: free, pro, enterprise)';
COMMENT ON COLUMN subscription_plans.price_cents IS '价格 (分)';
COMMENT ON COLUMN subscription_plans.interval IS '计费周期类型 (month/year)';
COMMENT ON COLUMN subscription_plans.interval_count IS '计费周期数量 (1=月付, 12=年付)';
COMMENT ON COLUMN subscription_plans.trial_days IS '试用期天数';

-- =============================================================================
-- 租户订阅表 (subscriptions)
-- =============================================================================
CREATE TABLE IF NOT EXISTS subscriptions (
    id                      BIGSERIAL PRIMARY KEY,
    tenant_id               BIGINT NOT NULL,
    plan_id                 VARCHAR(64) NOT NULL REFERENCES subscription_plans(id),
    status                  VARCHAR(32) NOT NULL DEFAULT 'active',
                            -- 'active', 'past_due', 'canceled', 'trialing'
    current_period_start    TIMESTAMPTZ NOT NULL,
    current_period_end      TIMESTAMPTZ NOT NULL,
    cancel_at_period_end    BOOLEAN NOT NULL DEFAULT FALSE,
    payment_method_id       VARCHAR(128),
    trial_start             TIMESTAMPTZ,
    trial_end               TIMESTAMPTZ,
    canceled_at             TIMESTAMPTZ,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

COMMENT ON TABLE subscriptions IS '租户订阅表';
COMMENT ON COLUMN subscriptions.tenant_id IS '关联的租户ID';
COMMENT ON COLUMN subscriptions.plan_id IS '订阅计划ID';
COMMENT ON COLUMN subscriptions.status IS '订阅状态: active/past_due/canceled/trialing';
COMMENT ON COLUMN subscriptions.cancel_at_period_end IS '是否在周期结束时取消';

-- 索引: 按租户查询订阅
CREATE INDEX IF NOT EXISTS idx_subscriptions_tenant_id ON subscriptions(tenant_id);

-- 索引: 按状态查询
CREATE INDEX IF NOT EXISTS idx_subscriptions_status ON subscriptions(status);

-- 索引: 按租户+状态查询 (最常用查询)
CREATE INDEX IF NOT EXISTS idx_subscriptions_tenant_status ON subscriptions(tenant_id, status);

-- 索引: 按周期结束时间查询 (用于续约提醒)
CREATE INDEX IF NOT EXISTS idx_subscriptions_period_end ON subscriptions(current_period_end);

-- 唯一约束: 每个租户只能有一个活跃订阅
CREATE UNIQUE INDEX IF NOT EXISTS idx_subscriptions_active_unique 
    ON subscriptions(tenant_id) 
    WHERE status IN ('active', 'trialing');

-- =============================================================================
-- 发票主表 (invoices)
-- =============================================================================
CREATE TABLE IF NOT EXISTS invoices (
    id              BIGSERIAL PRIMARY KEY,
    tenant_id       BIGINT NOT NULL,
    subscription_id BIGINT REFERENCES subscriptions(id),
    status          VARCHAR(32) NOT NULL DEFAULT 'draft',
                        -- 'draft', 'open', 'paid', 'void', 'uncollectible'
    amount_cents    BIGINT NOT NULL DEFAULT 0,
    currency        VARCHAR(8) NOT NULL DEFAULT 'CNY',
    period_start    TIMESTAMPTZ,
    period_end      TIMESTAMPTZ,
    paid_at         TIMESTAMPTZ,
    payment_id      VARCHAR(128),
    description     TEXT,
    metadata        JSONB DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

COMMENT ON TABLE invoices IS '发票主表';
COMMENT ON COLUMN invoices.status IS '发票状态: draft/open/paid/void/uncollectible';
COMMENT ON COLUMN invoices.amount_cents IS '发票金额 (分)';

-- 索引: 按租户查询发票
CREATE INDEX IF NOT EXISTS idx_invoices_tenant_id ON invoices(tenant_id);

-- 索引: 按订阅查询发票
CREATE INDEX IF NOT EXISTS idx_invoices_subscription_id ON invoices(subscription_id);

-- 索引: 按状态查询
CREATE INDEX IF NOT EXISTS idx_invoices_status ON invoices(status);

-- 索引: 按创建时间排序
CREATE INDEX IF NOT EXISTS idx_invoices_created_at ON invoices(created_at DESC);

-- =============================================================================
-- 发票明细表 (invoice_items)
-- =============================================================================
CREATE TABLE IF NOT EXISTS invoice_items (
    id              BIGSERIAL PRIMARY KEY,
    invoice_id      BIGINT NOT NULL REFERENCES invoices(id) ON DELETE CASCADE,
    description     TEXT NOT NULL,
    amount_cents    BIGINT NOT NULL DEFAULT 0,
    quantity        INTEGER NOT NULL DEFAULT 1,
    currency        VARCHAR(8) NOT NULL DEFAULT 'CNY',
    metadata        JSONB DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

COMMENT ON TABLE invoice_items IS '发票明细表';

-- 索引: 按发票ID查询明细
CREATE INDEX IF NOT EXISTS idx_invoice_items_invoice_id ON invoice_items(invoice_id);

-- =============================================================================
-- 订阅审计日志表 (subscription_events)
-- =============================================================================
CREATE TABLE IF NOT EXISTS subscription_events (
    id              BIGSERIAL PRIMARY KEY,
    subscription_id BIGINT NOT NULL REFERENCES subscriptions(id),
    event_type      VARCHAR(64) NOT NULL,
                    -- 'created', 'renewed', 'canceled', 'payment_failed', etc.
    old_value       JSONB,
    new_value       JSONB,
    description     TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

COMMENT ON TABLE subscription_events IS '订阅事件审计日志';

-- 索引: 按订阅ID查询事件
CREATE INDEX IF NOT EXISTS idx_subscription_events_subscription_id ON subscription_events(subscription_id);

-- 索引: 按事件类型查询
CREATE INDEX IF NOT EXISTS idx_subscription_events_type ON subscription_events(event_type);

-- =============================================================================
-- 默认订阅计划数据
-- =============================================================================
INSERT INTO subscription_plans (id, name, description, price_cents, currency, interval, interval_count, trial_days, active, features)
VALUES 
    ('free', 'Free', '免费版本，适合个人和小团队体验', 0, 'CNY', 'month', 1, 0, TRUE, 
     '{"max_users": 5, "max_projects": 3, "support": "community"}'::jsonb),
    ('pro', 'Pro', '专业版本，适合成长型团队', 9900, 'CNY', 'month', 1, 14, TRUE, 
     '{"max_users": 50, "max_projects": 20, "support": "email"}'::jsonb),
    ('enterprise', 'Enterprise', '企业版本，提供完整功能和专属支持', 49900, 'CNY', 'month', 1, 30, TRUE, 
     '{"max_users": null, "max_projects": null, "support": "dedicated"}'::jsonb)
ON CONFLICT (id) DO NOTHING;

-- =============================================================================
-- 权限授予
-- =============================================================================
DO $$
BEGIN
    -- 为应用角色授予订阅相关表的权限
    IF EXISTS (SELECT FROM pg_roles WHERE rolname = 'myai_app') THEN
        GRANT ALL ON ALL TABLES IN SCHEMA public TO myai_app;
        GRANT ALL ON ALL SEQUENCES IN SCHEMA public TO myai_app;
    END IF;
END $$;
