//! gRPC Service Handlers for Feedback Service
//!
//! 提供反馈管理的 gRPC 接口

use std::net::SocketAddr;
use grpc_proto::feedback::feedback_service_server::FeedbackServiceServer;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tonic::Status;

use crate::repository::{Feedback, FeedbackQueryParams, FeedbackRepository, PaginatedFeedback};

/// Feedback 应用状态
#[derive(Clone)]
pub struct FeedbackAppState {
    pub repository: FeedbackRepository,
}

/// 反馈信息 gRPC 响应结构
#[derive(Debug, Clone)]
pub struct FeedbackInfo {
    pub id: i64,
    pub user_id: i64,
    pub user_name: Option<String>,
    pub r#type: String,
    pub title: String,
    pub content: String,
    pub contact: Option<String>,
    pub status: String,
    pub handler_id: Option<i64>,
    pub handler_name: Option<String>,
    pub handler_reply: Option<String>,
    pub handler_time: Option<DateTime<Utc>>,
    pub rating: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Feedback> for FeedbackInfo {
    fn from(f: Feedback) -> Self {
        Self {
            id: f.id,
            user_id: f.user_id,
            user_name: f.user_name,
            r#type: f.r#type,
            title: f.title,
            content: f.content,
            contact: f.contact,
            status: f.status,
            handler_id: f.handler_id,
            handler_name: f.handler_name,
            handler_reply: f.handler_reply,
            handler_time: f.handler_time,
            rating: f.rating,
            created_at: f.created_at,
            updated_at: f.updated_at,
        }
    }
}

/// 分页反馈响应
#[derive(Debug, Clone)]
pub struct PaginatedFeedbackInfo {
    pub list: Vec<FeedbackInfo>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

impl From<PaginatedFeedback> for PaginatedFeedbackInfo {
    fn from(p: PaginatedFeedback) -> Self {
        Self {
            list: p.list.into_iter().map(FeedbackInfo::from).collect(),
            total: p.total,
            page: p.page,
            page_size: p.page_size,
        }
    }
}

/// 统计信息响应
#[derive(Debug, Clone)]
pub struct StatisticsInfo {
    pub total: i64,
    pub pending: i64,
    pub processing: i64,
    pub resolved: i64,
    pub rejected: i64,
    pub avg_response_time: f64,
    pub satisfaction_rate: f64,
}

// ============== 反馈管理接口实现 ==============

/// 获取反馈列表
pub async fn list_feedback(
    state: Arc<FeedbackAppState>,
    page: Option<i32>,
    page_size: Option<i32>,
    feedback_type: Option<String>,
    status: Option<String>,
    keyword: Option<String>,
) -> Result<PaginatedFeedbackInfo, Status> {
    let params = FeedbackQueryParams {
        r#type: feedback_type,
        status,
        keyword,
        handler_id: None,
        start_date: None,
        end_date: None,
        page,
        page_size,
    };

    state
        .repository
        .list(&params)
        .await
        .map(PaginatedFeedbackInfo::from)
        .map_err(|e| Status::internal(format!("Database error: {e}")))
}

/// 获取反馈详情
pub async fn get_feedback(
    state: Arc<FeedbackAppState>,
    id: i64,
) -> Result<Option<FeedbackInfo>, Status> {
    state
        .repository
        .find_by_id(id)
        .await
        .map(|opt| opt.map(FeedbackInfo::from))
        .map_err(|e| Status::internal(format!("Database error: {e}")))
}

/// 创建反馈
pub async fn create_feedback(
    state: Arc<FeedbackAppState>,
    user_id: i64,
    feedback_type: String,
    title: String,
    content: String,
    contact: Option<String>,
) -> Result<i64, Status> {
    state
        .repository
        .create(
            user_id,
            &feedback_type,
            &title,
            &content,
            contact.as_deref(),
        )
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}")))
}

/// 处理反馈
pub async fn handle_feedback(
    state: Arc<FeedbackAppState>,
    id: i64,
    status: String,
    reply: String,
) -> Result<bool, Status> {
    state
        .repository
        .handle(id, &status, &reply)
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}")))
}

/// 转交反馈
pub async fn transfer_feedback(
    state: Arc<FeedbackAppState>,
    id: i64,
    handler_id: i64,
) -> Result<bool, Status> {
    state
        .repository
        .transfer(id, handler_id)
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}")))
}

/// 回复反馈
pub async fn reply_feedback(
    state: Arc<FeedbackAppState>,
    id: i64,
    reply: String,
) -> Result<bool, Status> {
    state
        .repository
        .add_reply(id, &reply)
        .await
        .map(|()| true)
        .map_err(|e| Status::internal(format!("Database error: {e}")))
}

/// 关闭反馈
pub async fn close_feedback(state: Arc<FeedbackAppState>, id: i64) -> Result<bool, Status> {
    state
        .repository
        .handle(id, "closed", "")
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}")))
}

/// 删除反馈
pub async fn delete_feedback(state: Arc<FeedbackAppState>, id: i64) -> Result<bool, Status> {
    state
        .repository
        .delete(id)
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}")))
}

/// 批量处理反馈
pub async fn batch_handle_feedback(
    state: Arc<FeedbackAppState>,
    ids: Vec<i64>,
    status: String,
    reply: Option<String>,
) -> Result<i64, Status> {
    state
        .repository
        .batch_handle(&ids, &status, reply.as_deref())
        .await
        .map(|count| count as i64)
        .map_err(|e| Status::internal(format!("Database error: {e}")))
}

/// 获取统计信息
pub async fn get_statistics(
    state: Arc<FeedbackAppState>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<StatisticsInfo, Status> {
    let params = FeedbackQueryParams {
        r#type: None,
        status: None,
        keyword: None,
        handler_id: None,
        start_date,
        end_date,
        page: None,
        page_size: None,
    };

    state
        .repository
        .get_statistics(&params)
        .await
        .map(|s| StatisticsInfo {
            total: s.total,
            pending: s.pending,
            processing: s.processing,
            resolved: s.resolved,
            rejected: s.rejected,
            avg_response_time: s.avg_response_time,
            satisfaction_rate: s.satisfaction_rate,
        })
        .map_err(|e| Status::internal(format!("Database error: {e}")))
}

// ============== 导出服务实现 ==============

impl FeedbackAppState {
    /// 创建新的应用状态
    #[must_use]
    pub const fn new(repository: FeedbackRepository) -> Self {
        Self { repository }
    }

    /// 获取仓储引用
    #[must_use]
    pub const fn repository(&self) -> &FeedbackRepository {
        &self.repository
    }
}

/// Feedback gRPC 服务实现
#[derive(Clone)]
pub struct FeedbackGrpcService {
    state: Arc<FeedbackAppState>,
}

impl FeedbackGrpcService {
    /// 创建新的 gRPC 服务
    #[must_use]
    pub const fn new(state: Arc<FeedbackAppState>) -> Self {
        Self { state }
    }

    /// 获取状态引用
    #[must_use]
    pub const fn state(&self) -> &Arc<FeedbackAppState> {
        &self.state
    }
}

impl common::service_bootstrap::GrpcServiceBuilder for FeedbackGrpcService {
    fn build_grpc_server(&self, grpc_addr: &str) -> Result<tokio::task::JoinHandle<()>, Box<dyn std::error::Error + Send + Sync>> {
        use tonic::transport::Server;

        let addr: SocketAddr = grpc_addr.parse().expect("invalid grpc addr");
        let server = FeedbackServiceServer::new(FeedbackGrpcService::new(self.state.clone()));
        let handle = tokio::spawn(async move {
            if let Err(e) = Server::builder()
                .add_service(server).serve(addr).await {
                tracing::error!("gRPC server error: {}", e);
            }
        });
        Ok(handle)
    }
}
