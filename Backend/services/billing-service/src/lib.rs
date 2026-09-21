//! billing-service 库
//!
//! 历史说明: 原计费实现（plans/subscriptions/invoices/usage v1 表）已随 schema v2
//! 重构移除 —— 订阅计费真实现位于 pay-service（services/subscription.rs，
//! 使用 subscription_plans/subscriptions BIGINT 表）。本服务当前仅保留
//! HTTP 健康检查 + gRPC no-op 骨架，等待与 pay-service 合并或下线（见后端待办）。

use sqlx::PgPool;

/// 计费服务应用状态（保留数据库连接池供健康检查/未来扩展）
#[derive(Clone)]
pub struct BillingAppState {
    #[allow(dead_code)] // 供未来计费实现使用；main.rs 仅构造不读取
    pub pool: PgPool,
}

impl BillingAppState {
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
