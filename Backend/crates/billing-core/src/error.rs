//! 计费错误类型

use thiserror::Error;

#[derive(Debug, Error)]
pub enum BillingError {
    #[error("Plan not found: {0}")]
    PlanNotFound(String),

    #[error("Subscription not found: {0}")]
    SubscriptionNotFound(String),

    #[error("Invoice not found: {0}")]
    InvoiceNotFound(String),

    #[error("Payment failed: {0}")]
    PaymentFailed(String),

    #[error("Invalid billing period: {0}")]
    InvalidPeriod(String),

    #[error("Plan already exists: {0}")]
    DuplicatePlan(String),

    #[error("Subscription already exists for tenant: {0}")]
    DuplicateSubscription(String),

    #[error("Usage record error: {0}")]
    UsageError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Calculation error: {0}")]
    CalculationError(String),
}

pub type BillingResult<T> = Result<T, BillingError>;
