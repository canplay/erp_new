//! JWT 认证中间件
//!
//! 负责解析、验证 JWT token，并将用户信息注入到请求头中。
//!
//! # 功能
//! - 验证 JWT 签名、过期时间、issuer、audience
//! - 将 user_id / username / role / tenant_id 注入请求头
//! - 公开路径豁免鉴权
//! - 支持 WebSocket 握手（token 通过 URL 查询参数传递）


use axum::{
    extract::Request,
    http::{HeaderValue, StatusCode, header::HeaderName},
    middleware::Next,
    response::{IntoResponse, Response},
};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};

// 请求头名称常量
const HEADER_USER_ID: &str = "x-user-id";
const HEADER_USER_NAME: &str = "x-user-name";
const HEADER_USER_ROLE: &str = "x-user-role";
const HEADER_USER_TOKEN: &str = "x-user-token";

/// JWT Claims 结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: i64,        // user_id
    pub username: String,
    pub role: String,
    pub exp: usize,   // 过期时间
    pub iat: usize,   // 签发时间
    #[serde(default)]
    pub iss: Option<String>,    // 签发者
    #[serde(default)]
    pub aud: Option<String>,    // 受众
    #[serde(default)]
    pub tenant_id: Option<String>, // 租户 ID (可选, 多租户场景)
    #[serde(default)]
    pub roles: Vec<String>,        // 用户角色列表
    #[serde(default)]
    pub permissions: Vec<String>,  // 用户权限列表
}

/// JWT 鉴权状态
#[derive(Clone)]
pub struct AuthState {
    pub jwt_secret: String,
    pub jwt_issuer: String,
    pub jwt_audience: String,
}

impl AuthState {
    /// 创建新的鉴权状态
    #[must_use]
    pub const fn new(jwt_secret: String, jwt_issuer: String, jwt_audience: String) -> Self {
        Self {
            jwt_secret,
            jwt_issuer,
            jwt_audience,
        }
    }

    /// 验证 JWT token
    pub fn verify_token(&self, token: &str) -> Result<JwtClaims, String> {
        let mut validation = Validation::new(Algorithm::HS256);

        // 设置验证参数
        if !self.jwt_issuer.is_empty() {
            validation.set_issuer(&[&self.jwt_issuer]);
        }
        if !self.jwt_audience.is_empty() {
            validation.set_audience(&[&self.jwt_audience]);
        }

        // 校验过期时间 (修复: 原 validate_exp=false 导致过期 token 永久有效)
        validation.validate_exp = true;

        let token_data = decode::<JwtClaims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &validation,
        )
        .map_err(|e| format!("Token 解析失败: {e}" ))?;

        Ok(token_data.claims)
    }
}

/// 检查路径是否为公开路径（不需要鉴权）
fn is_public_path(path: &str) -> bool {
    path.starts_with("/health" )
        || path.starts_with("/ready" )
        || path.starts_with("/api/health" )
        || path == "/"
        || path.starts_with("/api/user/login" )
        || path.starts_with("/api/user/register" )
        // 内部监控端点（Prometheus、运维等）
        || path.starts_with("/metrics" )
        || path.starts_with("/rate-limit" )
        || path.starts_with("/trace" )
        // 兼容旧前端调用路径（/api/auth/* → /api/user/*）
        || path.starts_with("/api/auth/login" )
        || path.starts_with("/api/auth/register" )
        // Token 刷新接口（无需 JWT，使用 refresh_token）
        || path.starts_with("/api/auth/refresh" )
        // ebike 共享单车运营方登录（运营方推送数据需先登录获取 token）
        || path.starts_with("/api/v1/ebike/login" )
        // WebSocket 握手（浏览器 WebSocket API 无法在握手时带 Authorization 头,
        // token 通过 URL 查询参数 ?token= 传递并在 ws handler 内验证, 故豁免中间件）
        || path.starts_with("/ws/" )
}

/// JWT 鉴权中间件
///
/// 在验证 JWT 后，将用户信息注入到请求头中传递给后端服务：
/// - X-User-Id: 用户 ID
/// - X-User-Name: 用户名
/// - X-User-Role: 用户角色
/// - X-User-Token: 原始 Token（可选）
pub async fn auth_middleware(mut request: Request, next: Next) -> Response {
    // 公开路径不需要鉴权
    let path = request.uri().path();

    if is_public_path(path) {
        tracing::debug!("公开路径，跳过鉴权: {path}" );
        return next.run(request).await;
    }

    // 获取 Authorization header
    let auth_header = request
        .headers()
        .get("authorization" )
        .and_then(|v| v.to_str().ok());

    let token = match auth_header {
        Some(header) if header.starts_with("Bearer " ) => &header[7..],
        _ => {
            tracing::warn!("缺少认证令牌: {path}" );
            return Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .header("Content-Type" , "application/json" )
                .body(r#"{"success":false,"error":"缺少有效的认证令牌" }"#.into())
                .unwrap_or_else(|e| {
                    tracing::error!(error = %e, "构造 401 响应失败" );
                    (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error" ).into_response()
                });
        }
    };

    // 从扩展中获取鉴权状态
    let auth_state = request.extensions().get::<AuthState>().cloned();

    let claims = if let Some(state) = auth_state {
        match state.verify_token(token) {
            Ok(claims) => claims,
            Err(e) => {
                tracing::warn!("Token 验证失败 [{path}]: {e}" );
                return Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .header("Content-Type" , "application/json" )
                    .body(r#"{"success":false,"error":"无效或过期的令牌" }"#.into())
                    .unwrap_or_else(|e| {
                        tracing::error!(error = %e, "构造 401 响应失败" );
                        (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error" ).into_response()
                    });
            }
        }
    } else {
        // 没有配置鉴权状态，跳过验证（开发模式）
        tracing::debug!("未配置 JWT 鉴权，跳过验证: {path}" );
        return next.run(request).await;
    };

    // 将用户信息注入到请求头，传递给后端服务
    let user_id = claims.sub.to_string();
    let user_name = claims.username.clone();
    let user_role = claims.role.clone();
    let user_token = token.to_string();

    // 获取可变请求头
    // 审计修复 (C5): 多租户上下文透传——客户端 X-Tenant-Id 头原样转发给后端服务,
    // 由各业务服务按租户 ID 过滤数据。缺失时从 JWT 租户声明兜底(单租户部署不受影响)。
    // 注意: 必须先提取 tenant_id, 再取 headers_mut, 避免借用冲突
    let tenant_id = request
        .headers()
        .get("x-tenant-id" )
        .and_then(|v| v.to_str().ok())
        .map(str::to_string)
        .or_else(|| {
            request
                .extensions()
                .get::<JwtClaims>()
                .and_then(|c| c.tenant_id.clone())
        });

    // 注入用户信息头（如果不存在则添加）
    let headers = request.headers_mut();
    headers.insert(
        HeaderName::from_static(HEADER_USER_ID),
        HeaderValue::from_str(&user_id).unwrap_or_else(|_| HeaderValue::from_static("0" )),
    );
    headers.insert(
        HeaderName::from_static(HEADER_USER_NAME),
        HeaderValue::from_str(&user_name).unwrap_or_else(|_| HeaderValue::from_static("" )),
    );
    headers.insert(
        HeaderName::from_static(HEADER_USER_ROLE),
        HeaderValue::from_str(&user_role).unwrap_or_else(|_| HeaderValue::from_static("" )),
    );
    headers.insert(
        HeaderName::from_static(HEADER_USER_TOKEN),
        HeaderValue::from_str(&user_token).unwrap_or_else(|_| HeaderValue::from_static("" )),
    );

    // 透传租户上下文(已在函数头部提取)
    if let Some(tid) = tenant_id {
        headers.insert(
            HeaderName::from_static("x-tenant-id" ),
            HeaderValue::from_str(&tid).unwrap_or_else(|_| HeaderValue::from_static("" )),
        );
    }

    tracing::debug!(
        "用户已认证: user_id={user_id}, username={user_name}, role={user_role}"
    );

    // 注入 JwtClaims 到 Extension，供后续 handler 使用
    request.extensions_mut().insert(claims);

    next.run(request).await
}
