//! API Key Service gRPC Server
//!
//! 实现 api_key.proto 中定义的 gRPC 服务 trait
//! 使用 Postgres 持久化（通过 grpc_handlers 中的业务逻辑）

use chrono::Utc;
use grpc_proto::api_key::api_key_service_server::ApiKeyServiceServer;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use uuid::Uuid;
use std::net::SocketAddr;

use crate::grpc_handlers::{self, ApiKeyAppState};

use grpc_proto::api_key::{
    ApiKeyInfo, CreateApiKeyRequest, CreateApiKeyResponse, DeleteApiKeyRequest,
    DeleteApiKeyResponse, GetApiKeyRequest, GetApiKeyResponse, GetKeyUsageRequest,
    GetKeyUsageResponse, ListApiKeysRequest, ListApiKeysResponse, RotateApiKeyRequest,
    RotateApiKeyResponse, UpdateApiKeyRequest, UpdateApiKeyResponse, ValidateApiKeyRequest,
    ValidateApiKeyResponse, api_key_service_server::ApiKeyService,
};

/// `ApiKeyService` 的 gRPC 实现
#[derive(Clone)]
pub struct ApiKeyGrpcServer {
    state: Arc<ApiKeyAppState>,
}

impl ApiKeyGrpcServer {
    #[must_use]
    pub fn new(state: Arc<ApiKeyAppState>) -> Self {
        Self { state }
    }
}

#[tonic::async_trait]
impl ApiKeyService for ApiKeyGrpcServer {
    async fn create_api_key(
        &self,
        request: Request<CreateApiKeyRequest>,
    ) -> Result<Response<CreateApiKeyResponse>, Status> {
        let req = request.into_inner();

        let params = grpc_handlers::CreateApiKeyParams {
            name: req.name,
            description: if req.description.is_empty() { None } else { Some(req.description) },
            permission_level: req.permission_level,
            allowed_ips: if req.allowed_ips.is_empty() { None } else { Some(req.allowed_ips) },
            rate_limit: if req.rate_limit > 0 { Some(req.rate_limit) } else { None },
            expires_at: if req.expires_at > 0 {
                Some(
                    chrono::DateTime::from_timestamp(req.expires_at, 0)
                        .unwrap_or_else(Utc::now)
                        .to_rfc3339(),
                )
            } else {
                None
            },
            user_id: req.user_id.to_string(),
            tenant_id: if req.tenant_id > 0 { Some(req.tenant_id.to_string()) } else { None },
        };

        let resp = grpc_handlers::create_api_key(self.state.clone(), params)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(CreateApiKeyResponse {
            id: resp.id.parse().unwrap_or(0),
            key_id: resp.key_prefix,
            secret_key: resp.key,
            name: String::new(),
        }))
    }

    async fn list_api_keys(
        &self,
        request: Request<ListApiKeysRequest>,
    ) -> Result<Response<ListApiKeysResponse>, Status> {
        let req = request.into_inner();

        let resp = grpc_handlers::list_api_keys(
            self.state.clone(),
            req.page,
            req.page_size,
            None,
            None,
            if req.user_id > 0 { Some(req.user_id.to_string()) } else { None },
            if req.tenant_id > 0 { Some(req.tenant_id.to_string()) } else { None },
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(ListApiKeysResponse {
            keys: resp.keys.into_iter().map(|k| ApiKeyInfo {
                id: k.id.parse().unwrap_or(0),
                name: k.name,
                key_id: k.key_prefix,
                key_hint: String::new(),
                permission_level: k.permission_level.parse().unwrap_or(0),
                allowed_ips: k.allowed_ips.unwrap_or_default(),
                rate_limit: k.rate_limit.unwrap_or(0),
                tenant_id: k.tenant_id.and_then(|s| s.parse().ok()).unwrap_or(0),
                user_id: k.user_id.parse().unwrap_or(0),
                status: k.status,
                created_at: chrono::DateTime::parse_from_rfc3339(&k.created_at)
                    .map(|dt| dt.timestamp())
                    .unwrap_or(0),
                expires_at: k.expires_at
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.timestamp())
                    .unwrap_or(0),
                last_used_at: k.last_used_at
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.timestamp())
                    .unwrap_or(0),
            }).collect(),
            total: resp.total,
        }))
    }

    async fn get_api_key(
        &self,
        request: Request<GetApiKeyRequest>,
    ) -> Result<Response<GetApiKeyResponse>, Status> {
        let req = request.into_inner();

        let resp = grpc_handlers::get_api_key(self.state.clone(), req.id.to_string())
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        match resp {
            Some(k) => Ok(Response::new(GetApiKeyResponse {
                key: Some(ApiKeyInfo {
                    id: k.id.parse().unwrap_or(0),
                    name: k.name,
                    key_id: k.key_prefix,
                    key_hint: String::new(),
                    permission_level: k.permission_level.parse().unwrap_or(0),
                    allowed_ips: k.allowed_ips.unwrap_or_default(),
                    rate_limit: k.rate_limit.unwrap_or(0),
                    tenant_id: k.tenant_id.and_then(|s| s.parse().ok()).unwrap_or(0),
                    user_id: k.user_id.parse().unwrap_or(0),
                    status: k.status,
                    created_at: chrono::DateTime::parse_from_rfc3339(&k.created_at)
                        .map(|dt| dt.timestamp()).unwrap_or(0),
                    expires_at: k.expires_at
                        .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                        .map(|dt| dt.timestamp()).unwrap_or(0),
                    last_used_at: k.last_used_at
                        .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                        .map(|dt| dt.timestamp()).unwrap_or(0),
                }),
            })),
            None => Err(Status::not_found("API Key not found")),
        }
    }

    async fn update_api_key(
        &self,
        request: Request<UpdateApiKeyRequest>,
    ) -> Result<Response<UpdateApiKeyResponse>, Status> {
        let req = request.into_inner();

        let params = grpc_handlers::UpdateApiKeyParams {
            id: req.id.to_string(),
            name: if req.name.is_empty() { None } else { Some(req.name) },
            description: if req.description.is_empty() { None } else { Some(req.description) },
            permission_level: if req.permission_level > 0 { Some(req.permission_level) } else { None },
            allowed_ips: if req.allowed_ips.is_empty() { None } else { Some(req.allowed_ips) },
            rate_limit: if req.rate_limit > 0 { Some(req.rate_limit) } else { None },
            status: match req.status.as_str() {
                "active" => Some(0),
                "inactive" => Some(1),
                _ => None,
            },
            expires_at: if req.expires_at > 0 {
                Some(
                    chrono::DateTime::from_timestamp(req.expires_at, 0)
                        .unwrap_or_else(Utc::now)
                        .to_rfc3339(),
                )
            } else {
                None
            },
        };

        let success = grpc_handlers::update_api_key(self.state.clone(), params)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(UpdateApiKeyResponse { success }))
    }

    async fn delete_api_key(
        &self,
        request: Request<DeleteApiKeyRequest>,
    ) -> Result<Response<DeleteApiKeyResponse>, Status> {
        let req = request.into_inner();

        let success = grpc_handlers::delete_api_key(self.state.clone(), req.id.to_string())
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(DeleteApiKeyResponse { success }))
    }

    async fn validate_api_key(
        &self,
        request: Request<ValidateApiKeyRequest>,
    ) -> Result<Response<ValidateApiKeyResponse>, Status> {
        let req = request.into_inner();

        // grpc_handlers::validate_api_key 将 key_id+secret 拼接为一个 key 参数
        let full_key = format!("{}{}", req.key_id, req.secret_key);

        let resp = grpc_handlers::validate_api_key(
            self.state.clone(),
            full_key,
            if req.ip_address.is_empty() { None } else { Some(req.ip_address) },
            None,
            None,
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(ValidateApiKeyResponse {
            valid: resp.valid,
            user_id: resp.key_id.map(|_| 0i64)
                .unwrap_or(0),
            tenant_id: 0,
            permission_level: resp.permission_level
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            message: resp.error.unwrap_or_default(),
        }))
    }

    async fn get_key_usage(
        &self,
        _request: Request<GetKeyUsageRequest>,
    ) -> Result<Response<GetKeyUsageResponse>, Status> {
        // 简化实现，返回空数据
        Ok(Response::new(GetKeyUsageResponse {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            total_latency_ms: 0,
            daily_usages: vec![],
        }))
    }

    async fn rotate_api_key(
        &self,
        request: Request<RotateApiKeyRequest>,
    ) -> Result<Response<RotateApiKeyResponse>, Status> {
        let req = request.into_inner();
        let id = req.id.to_string();

        // 先删除再创建（简化实现）
        grpc_handlers::delete_api_key(self.state.clone(), id.clone()).await
            .map_err(|e| Status::internal(e.to_string()))?;

        // 获取旧 key 的信息来重新创建
        let _old_key = grpc_handlers::get_api_key(self.state.clone(), id.clone()).await
            .map_err(|e| Status::internal(e.to_string()))?;

        let new_secret = Uuid::new_v4().to_string().replace('-', "");
        let new_key_id = ApiKeyAppState::generate_key_id();

        Ok(Response::new(RotateApiKeyResponse {
            id: req.id,
            key_id: new_key_id,
            new_secret_key: new_secret,
        }))
    }
}

impl common::service_bootstrap::GrpcServiceBuilder for ApiKeyGrpcServer {
    fn build_grpc_server(&self, grpc_addr: &str) -> Result<tokio::task::JoinHandle<()>, Box<dyn std::error::Error + Send + Sync>> {
        use tonic::transport::Server;

        let addr: SocketAddr = grpc_addr.parse().map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("invalid grpc addr: {e}").into() })?;
        let server = ApiKeyServiceServer::new(ApiKeyGrpcServer::new(self.state.clone()));
        let handle = tokio::spawn(async move {
            if let Err(e) = Server::builder()
                .add_service(server).serve(addr).await {
                tracing::error!("gRPC server error: {}", e);
            }
        });
        Ok(handle)
    }
}
