//! 发票管理

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 发票状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvoiceStatus {
    /// 草稿
    Draft,
    /// 待支付
    Pending,
    /// 已支付
    Paid,
    /// 支付失败
    Failed,
    /// 已作废
    Void,
    /// 已退款
    Refunded,
}

impl InvoiceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft" ,
            Self::Pending => "pending" ,
            Self::Paid => "paid" ,
            Self::Failed => "failed" ,
            Self::Void => "void" ,
            Self::Refunded => "refunded" ,
        }
    }

    pub fn is_paid(&self) -> bool {
        matches!(self, Self::Paid)
    }
}

/// 发票
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    pub id: Uuid,
    pub invoice_number: String,
    pub subscription_id: Uuid,
    pub tenant_id: Uuid,
    pub status: InvoiceStatus,
    /// 小计（不含税）
    pub subtotal: Decimal,
    /// 税额
    pub tax_amount: Decimal,
    /// 总计
    pub total: Decimal,
    /// 货币
    pub currency: String,
    /// 账单周期开始
    pub period_start: DateTime<Utc>,
    /// 账单周期结束
    pub period_end: DateTime<Utc>,
    /// 出票日期
    pub issued_at: DateTime<Utc>,
    /// 到期日期
    pub due_at: DateTime<Utc>,
    /// 支付日期
    pub paid_at: Option<DateTime<Utc>>,
    /// 行项目
    pub line_items: Vec<InvoiceLineItem>,
    /// 备注
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Invoice {
    /// 创建新发票
    pub fn new(
        subscription_id: Uuid,
        tenant_id: Uuid,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            invoice_number: format!("INV-{}" , Utc::now().format("%Y%m%d%H%M%S" )),
            subscription_id,
            tenant_id,
            status: InvoiceStatus::Draft,
            subtotal: Decimal::ZERO,
            tax_amount: Decimal::ZERO,
            total: Decimal::ZERO,
            currency: "CNY".to_string(),
            period_start,
            period_end,
            issued_at: Utc::now(),
            due_at: Utc::now() + chrono::Duration::days(7),
            paid_at: None,
            line_items: Vec::new(),
            notes: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// 添加行项目
    pub fn add_line_item(&mut self, item: InvoiceLineItem) {
        self.line_items.push(item);
        self.recalculate();
    }

    /// 重新计算总额
    pub fn recalculate(&mut self) {
        self.subtotal = self.line_items.iter().map(|item| item.amount).sum();
        // 简化：假设税率 6%
        self.tax_amount = (self.subtotal * Decimal::try_from(0.06).unwrap_or(Decimal::ZERO)).round_dp(2);
        self.total = self.subtotal + self.tax_amount;
        self.updated_at = Utc::now();
    }

    /// 标记为已支付
    pub fn mark_paid(&mut self) {
        self.status = InvoiceStatus::Paid;
        self.paid_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// 标记为已作废
    pub fn mark_void(&mut self) {
        self.status = InvoiceStatus::Void;
        self.updated_at = Utc::now();
    }

    /// 检查是否逾期
    pub fn is_overdue(&self) -> bool {
        self.status == InvoiceStatus::Pending && Utc::now() > self.due_at
    }
}

/// 发票行项目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceLineItem {
    pub description: String,
    pub quantity: i32,
    pub unit_price: Decimal,
    pub amount: Decimal,
}

impl InvoiceLineItem {
    /// 创建新的行项目
    pub fn new(description: impl Into<String>, quantity: i32, unit_price: Decimal) -> Self {
        let amount = unit_price * Decimal::from(quantity);
        Self {
            description: description.into(),
            quantity,
            unit_price,
            amount,
        }
    }
}
