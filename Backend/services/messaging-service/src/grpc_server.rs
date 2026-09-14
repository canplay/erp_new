//! Messaging Service gRPC Server
//!
//! 实现 message.proto 中定义的 gRPC 服务 trait

use std::net::SocketAddr;
use grpc_proto::message::message_service_server::MessageServiceServer;
use std::sync::Arc;
use tonic::{Request, Response, Status};

// 导入 proto 生成的服务和消息
use grpc_proto::message::{
    CreateTemplateRequest, CreateTemplateResponse, DeleteMessageRequest, DeleteMessageResponse,
    DeleteTemplateRequest, DeleteTemplateResponse, GetMessageRequest, GetMessageResponse,
    GetUnreadCountRequest, GetUnreadCountResponse, ListMessagesRequest, ListMessagesResponse,
    ListTemplatesRequest, ListTemplatesResponse, MarkAsReadRequest, MarkAsReadResponse, Message,
    MessageData, SendMessageRequest, SendMessageResponse, UpdateTemplateRequest,
    UpdateTemplateResponse, message_service_server::MessageService,
};

use crate::repository::{PostgresAnnouncementRepository, PostgresMessageRepository};

/// 消息应用状态（共享数据）
pub type AppState = Arc<MessagingState>;

/// 消息状态结构
pub struct MessagingState {
    pub pool: sqlx::PgPool,
    pub message_repo: Arc<PostgresMessageRepository>,
    pub announcement_repo: Arc<PostgresAnnouncementRepository>,
}

impl MessagingState {
    #[must_use]
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self {
            message_repo: Arc::new(PostgresMessageRepository::new(pool.clone())),
            announcement_repo: Arc::new(PostgresAnnouncementRepository::new(pool.clone())),
            pool,
        }
    }
}

/// `MessageService` 实现
#[derive(Clone)]
pub struct MessagingGrpcServer {
    state: Arc<MessagingState>,
}

impl MessagingGrpcServer {
    /// 创建新的 `MessageService` 实例
    #[must_use]
    pub const fn new(state: Arc<MessagingState>) -> Self {
        Self { state }
    }
}

/// 将数据库记录转换为 proto Message
fn message_to_proto(m: &MessageRow) -> Message {
    Message {
        id: m.id,
        user_id: m.receiver_id.unwrap_or(m.sender_id.unwrap_or(0)),
        r#type: m.message_type.clone(),
        title: m.title.clone(),
        content: m.content.clone(),
        sender: m.sender_id.map(|id| id.to_string()).unwrap_or_default(),
        is_read: m.is_read,
        read_at: m.read_at.map_or(0, |t| t.timestamp()),
        created_at: m.created_at.timestamp(),
        data: Some(MessageData::default()),
    }
}

/// 消息行（从数据库查询）
#[derive(Debug, Clone, sqlx::FromRow)]
struct MessageRow {
    id: i64,
    title: String,
    content: String,
    sender_id: Option<i64>,
    receiver_id: Option<i64>,
    message_type: String,
    is_read: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    read_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// 模板行（从数据库查询）
#[derive(Debug, Clone, sqlx::FromRow)]
struct TemplateRow {
    id: i64,
    name: String,
    template_type: String,
    title_template: String,
    content_template: String,
    is_active: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[tonic::async_trait]
impl MessageService for MessagingGrpcServer {
    // ============== 消息相关 ==============

    async fn list_messages(
        &self,
        request: Request<ListMessagesRequest>,
    ) -> Result<Response<ListMessagesResponse>, Status> {
        let req = request.into_inner();

        let page = if req.page > 0 { i64::from(req.page) } else { 1 };
        let page_size = if req.page_size > 0 { i64::from(req.page_size) } else { 20 };
        let offset = (page - 1) * page_size;

        let type_filter = if req.r#type.is_empty() {
            None
        } else {
            Some(req.r#type.as_str())
        };

        let query_result = sqlx::query_as!(
            MessageRow,
            r#"SELECT m.id, m.title, m.content, m.sender_id, mu.user_id AS receiver_id,
               m.type AS message_type, (mu.is_read <> 0) AS "is_read!",
               COALESCE(m.created_at, NOW()) AS "created_at!", mu.read_time AS read_at
               FROM sys_message m
               JOIN sys_message_user mu ON m.id = mu.message_id
               WHERE mu.user_id = $1 AND mu.is_deleted = 0 AND ($2::text IS NULL OR m.type = $2)
               ORDER BY m.created_at DESC
               LIMIT $3 OFFSET $4"#,
            req.user_id,
            type_filter,
            page_size,
            offset,
        )
        .fetch_all(&self.state.pool)
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}")))?;

        // 查询总数
        let total = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM sys_message m JOIN sys_message_user mu ON m.id = mu.message_id WHERE mu.user_id = $1 AND mu.is_deleted = 0",
            req.user_id,
        )
        .fetch_one(&self.state.pool)
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}")))?
        .unwrap_or(0);

        // 查询未读数
        let unread_count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM sys_message_user WHERE user_id = $1 AND is_read = 0 AND is_deleted = 0",
            req.user_id,
        )
        .fetch_one(&self.state.pool)
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}")))?
        .unwrap_or(0);

        let messages: Vec<Message> = query_result.iter().map(message_to_proto).collect();

        Ok(Response::new(ListMessagesResponse {
            messages,
            total,
            unread_count,
        }))
    }

    async fn get_message(
        &self,
        request: Request<GetMessageRequest>,
    ) -> Result<Response<GetMessageResponse>, Status> {
        let req = request.into_inner();

        let result = sqlx::query_as!(
            MessageRow,
            r#"SELECT m.id, m.title, m.content, m.sender_id, mu.user_id AS receiver_id,
               m.type AS message_type, (mu.is_read <> 0) AS "is_read!",
               COALESCE(m.created_at, NOW()) AS "created_at!", mu.read_time AS read_at
               FROM sys_message m
               JOIN sys_message_user mu ON m.id = mu.message_id
               WHERE m.id = $1 AND mu.user_id = $2 AND mu.is_deleted = 0"#,
            req.id,
            req.user_id,
        )
        .fetch_optional(&self.state.pool)
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}")))?;

        match result {
            Some(m) => Ok(Response::new(GetMessageResponse {
                message: Some(message_to_proto(&m)),
            })),
            None => Err(Status::not_found("Message not found")),
        }
    }

    async fn send_message(
        &self,
        request: Request<SendMessageRequest>,
    ) -> Result<Response<SendMessageResponse>, Status> {
        let req = request.into_inner();

        // 确定发送目标用户
        let target_ids: Vec<i64> = if !req.user_ids.is_empty() {
            req.user_ids
        } else if req.user_id > 0 {
            vec![req.user_id]
        } else {
            return Err(Status::invalid_argument("No target user specified"));
        };

        let mut tx = self
            .state
            .pool
            .begin()
            .await
            .map_err(|e| Status::internal(format!("Transaction error: {e}")))?;

        // 插入消息主记录
        let message_row = sqlx::query!(
            r#"INSERT INTO sys_message (type, title, content, sender_id, sender_name, priority,
                attachment_urls, target_type, target_ids, expire_time, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW(), NOW())
               RETURNING id"#,
            &req.r#type,
            &req.title,
            &req.content,
            req.sender_id,
            "",
            0i16,
            &serde_json::Value::Null,
            "",
            &serde_json::Value::Null,
            None::<chrono::DateTime<chrono::Utc>>,
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}")))?;

        let message_id = message_row.id;

        // 为每个目标用户创建消息关系记录
        for user_id in &target_ids {
            sqlx::query!(
                r#"INSERT INTO sys_message_user (message_id, user_id, is_read, is_deleted, is_archived, created_at)
                   VALUES ($1, $2, 0, 0, 0, NOW())"#,
                message_id,
                user_id,
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| Status::internal(format!("Database error: {e}")))?;
        }

        tx.commit()
            .await
            .map_err(|e| Status::internal(format!("Transaction commit error: {e}")))?;

        Ok(Response::new(SendMessageResponse {
            id: message_id,
            sent_count: target_ids.len() as i32,
        }))
    }

    async fn mark_as_read(
        &self,
        request: Request<MarkAsReadRequest>,
    ) -> Result<Response<MarkAsReadResponse>, Status> {
        let req = request.into_inner();

        if req.message_ids.is_empty() {
            // 标记全部已读
            let result = sqlx::query!(
                "UPDATE sys_message_user SET is_read = 1, read_time = NOW() WHERE user_id = $1 AND is_read = 0",
                req.user_id,
            )
            .execute(&self.state.pool)
            .await
            .map_err(|e| Status::internal(format!("Database error: {e}")))?;

            Ok(Response::new(MarkAsReadResponse {
                count: result.rows_affected() as i32,
            }))
        } else {
            // 标记指定消息已读
            let result = sqlx::query!(
                "UPDATE sys_message_user SET is_read = 1, read_time = NOW() WHERE message_id = ANY($1) AND user_id = $2",
                &req.message_ids,
                req.user_id,
            )
            .execute(&self.state.pool)
            .await
            .map_err(|e| Status::internal(format!("Database error: {e}")))?;

            Ok(Response::new(MarkAsReadResponse {
                count: result.rows_affected() as i32,
            }))
        }
    }

    async fn delete_message(
        &self,
        request: Request<DeleteMessageRequest>,
    ) -> Result<Response<DeleteMessageResponse>, Status> {
        let req = request.into_inner();

        let result = sqlx::query!(
            "UPDATE sys_message_user SET is_deleted = 1 WHERE message_id = ANY($1) AND user_id = $2",
            &req.message_ids,
            req.user_id,
        )
        .execute(&self.state.pool)
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}")))?;

        Ok(Response::new(DeleteMessageResponse {
            count: result.rows_affected() as i32,
        }))
    }

    async fn get_unread_count(
        &self,
        request: Request<GetUnreadCountRequest>,
    ) -> Result<Response<GetUnreadCountResponse>, Status> {
        let req = request.into_inner();

        let type_filter = if req.r#type.is_empty() {
            None
        } else {
            Some(req.r#type.as_str())
        };

        let count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM sys_message m
               JOIN sys_message_user mu ON m.id = mu.message_id
               WHERE mu.user_id = $1 AND mu.is_read = 0 AND mu.is_deleted = 0
                 AND ($2::text IS NULL OR m.type = $2)"#,
            req.user_id,
            type_filter,
        )
        .fetch_one(&self.state.pool)
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}")))?
        .unwrap_or(0);

        Ok(Response::new(GetUnreadCountResponse { count }))
    }

    // ============== 模板相关 ==============

    async fn list_templates(
        &self,
        request: Request<ListTemplatesRequest>,
    ) -> Result<Response<ListTemplatesResponse>, Status> {
        let req = request.into_inner();

        let page = if req.page > 0 { i64::from(req.page) } else { 1 };
        let page_size = if req.page_size > 0 { i64::from(req.page_size) } else { 20 };
        let offset = (page - 1) * page_size;

        let type_filter = if req.r#type.is_empty() {
            None
        } else {
            Some(req.r#type.as_str())
        };

        let template_rows = sqlx::query_as!(
            TemplateRow,
            r#"SELECT id, name, template_type, title_template, content_template,
               COALESCE(is_active, false) AS "is_active!",
               COALESCE(created_at, NOW()) AS "created_at!",
               COALESCE(updated_at, NOW()) AS "updated_at!"
               FROM message_templates
               WHERE ($1::text IS NULL OR template_type = $1)
               ORDER BY created_at DESC
               LIMIT $2 OFFSET $3"#,
            type_filter,
            page_size,
            offset,
        )
        .fetch_all(&self.state.pool)
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}")))?;

        let templates: Vec<grpc_proto::message::MessageTemplate> = template_rows
            .into_iter()
            .map(|row| grpc_proto::message::MessageTemplate {
                id: row.id,
                name: row.name,
                r#type: row.template_type,
                title_template: row.title_template,
                content_template: row.content_template,
                variables: std::collections::HashMap::new(),
                is_active: row.is_active,
                created_at: row.created_at.timestamp(),
                updated_at: row.updated_at.timestamp(),
            })
            .collect();

        let total = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM message_templates WHERE ($1::text IS NULL OR template_type = $1)",
            type_filter,
        )
        .fetch_one(&self.state.pool)
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}")))?
        .unwrap_or(0);

        Ok(Response::new(ListTemplatesResponse { templates, total }))
    }

    async fn create_template(
        &self,
        request: Request<CreateTemplateRequest>,
    ) -> Result<Response<CreateTemplateResponse>, Status> {
        let req = request.into_inner();

        let result = sqlx::query!(
            r#"INSERT INTO message_templates (name, template_type, title_template, content_template, is_active, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, NOW(), NOW())
               RETURNING id"#,
            &req.name,
            &req.r#type,
            &req.title_template,
            &req.content_template,
            req.is_active,
        )
        .fetch_one(&self.state.pool)
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}")))?;

        let id = result.id;

        Ok(Response::new(CreateTemplateResponse { id, name: req.name }))
    }

    async fn update_template(
        &self,
        request: Request<UpdateTemplateRequest>,
    ) -> Result<Response<UpdateTemplateResponse>, Status> {
        let req = request.into_inner();

        let result = sqlx::query!(
            r#"UPDATE message_templates
               SET name = COALESCE(NULLIF($2, ''), name),
                   template_type = COALESCE(NULLIF($3, ''), template_type),
                   title_template = COALESCE(NULLIF($4, ''), title_template),
                   content_template = COALESCE(NULLIF($5, ''), content_template),
                   is_active = $6,
                   updated_at = NOW()
               WHERE id = $1
               RETURNING id"#,
            req.id,
            &req.name,
            &req.r#type,
            &req.title_template,
            &req.content_template,
            req.is_active,
        )
        .fetch_optional(&self.state.pool)
        .await
        .map_err(|e| Status::internal(format!("Database error: {e}")))?;

        match result {
            Some(row) => Ok(Response::new(UpdateTemplateResponse {
                id: row.id,
                name: req.name,
            })),
            None => Err(Status::not_found("Template not found")),
        }
    }

    async fn delete_template(
        &self,
        request: Request<DeleteTemplateRequest>,
    ) -> Result<Response<DeleteTemplateResponse>, Status> {
        let req = request.into_inner();

        let result = sqlx::query!("DELETE FROM message_templates WHERE id = $1", req.id)
            .execute(&self.state.pool)
            .await
            .map_err(|e| Status::internal(format!("Database error: {e}")))?;

        Ok(Response::new(DeleteTemplateResponse {
            success: result.rows_affected() > 0,
        }))
    }
}

impl common::service_bootstrap::GrpcServiceBuilder for MessagingGrpcServer {
    fn build_grpc_server(&self, grpc_addr: &str) -> Result<tokio::task::JoinHandle<()>, Box<dyn std::error::Error + Send + Sync>> {
        use tonic::transport::Server;

        let addr: SocketAddr = grpc_addr.parse().map_err(|e| format!("invalid grpc addr: {e}"))?;
        let server = MessageServiceServer::new(MessagingGrpcServer::new(self.state.clone()));
        let handle = tokio::spawn(async move {
            if let Err(e) = Server::builder()
                .add_service(server).serve(addr).await {
                tracing::error!("gRPC server error: {}", e);
            }
        });
        Ok(handle)
    }
}
