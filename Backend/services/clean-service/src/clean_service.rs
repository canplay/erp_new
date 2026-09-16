//! 清运服务 gRPC 处理器
//! gRPC handlers for clean service

use sqlx::PgPool;
use std::sync::Arc;

use common::AppError;
use common::AppResult;
use crate::models::{InvoiceQuery, Invoice, OrderQuery, Order, FormalBillQuery, FormalBill, PaymentWebQuery, PaymentWeb};
use crate::repository::{InvoiceRepository, OrderRepository, StaffRepository};
use crate::services::StatisticsService;

/// gRPC 应用状态
#[derive(Clone)]
pub struct AppState {pub invoice_repository: Arc<InvoiceRepository>, pub order_repository: Arc<OrderRepository>, pub staff_repository: Arc<StaffRepository>, pub statistics_service: Arc<StatisticsService>}

impl AppState {/// 从数据库连接池创建应用状态
 #[must_use]
 // 审计修复 (H8-架构评审): 使用传入的共享池, 不再让各仓储自建连接池
 pub fn new(pool: PgPool) -> Self {
 Self {
 invoice_repository: Arc::new(InvoiceRepository::new(pool.clone())), order_repository: Arc::new(OrderRepository::new(pool.clone())), staff_repository: Arc::new(StaffRepository::new(pool.clone())), statistics_service: Arc::new(StatisticsService::new(pool))}
 }
}

/// gRPC 服务实现
#[derive(Clone)]
pub struct CleanGrpcServer {state: Arc<AppState>}

impl CleanGrpcServer {/// 创建新的 gRPC 服务器实例
 #[must_use]
 pub fn new(pool: PgPool) -> Self {
 Self {
 state: Arc::new(AppState::new(pool))}
 }

 /// 获取应用状态引用
 #[must_use]
 pub fn state(&self) -> Arc<AppState> {self.state.clone()}
}

// ============ 数据访问方法 ============

/// 获取发票数量
pub async fn get_invoice_count(
 query: &InvoiceQuery,
 state: &AppState,
) -> Result<i64, AppError> {state
 .invoice_repository
 .count(query)
 .await}

/// 获取发票列表
pub async fn get_invoice_list(
 query: &InvoiceQuery,
 state: &AppState,
) -> AppResult<Vec<Invoice>> {state
 .invoice_repository
 .list(query)
 .await}

/// 获取订单数量
pub async fn get_order_count(
 query: &OrderQuery,
 state: &AppState,
) -> Result<i64, AppError> {state
 .order_repository
 .count(query)
 .await}

/// 获取订单列表
pub async fn get_order_list(
 query: &OrderQuery,
 state: &AppState,
) -> AppResult<Vec<Order>> {state
 .order_repository
 .list(query)
 .await}

/// 获取正式账单数量
pub async fn get_formal_bill_count(
 query: &FormalBillQuery,
 state: &AppState,
) -> Result<i64, AppError> {state
 .order_repository
 .count_formal_bill(query)
 .await}

/// 获取正式账单列表
pub async fn get_formal_bill_list(
 query: &FormalBillQuery,
 state: &AppState,
) -> AppResult<Vec<FormalBill>> {state
 .order_repository
 .list_formal_bill(query)
 .await}

/// 获取网络支付数量
pub async fn get_payment_web_count(
 query: &PaymentWebQuery,
 state: &AppState,
) -> Result<i64, AppError> {state
 .order_repository
 .count_payment_web(query)
 .await}

/// 获取网络支付列表
pub async fn get_payment_web_list(
 query: &PaymentWebQuery,
 state: &AppState,
) -> AppResult<Vec<PaymentWeb>> {state
 .order_repository
 .list_payment_web(query)
 .await}
