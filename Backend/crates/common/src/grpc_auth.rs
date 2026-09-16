//! gRPC 服务间鉴权
//!
//! 审计修复 (2026-08-04): 原实现 21 个服务 gRPC 全部无鉴权且绑定 0.0.0.0,
//! 任意网络可达者可直接调用敏感 RPC。本模块提供共享令牌鉴权:
//!
//! - 服务端: 每个服务挂载 `grpc_auth_interceptor`, 校验 `x-grpc-token` 请求头
//! - 客户端: grpc-core `connect_grpc` 统一注入 `GRPC_AUTH_TOKEN` 环境变量值
//!
//! 行为约定:
//! - `GRPC_AUTH_TOKEN` 未配置: 放行并告警(兼容存量部署; 生产必须由 Helm Secret 注入)
//! - 已配置: 请求头缺失或值不匹配 → 拒绝 (unauthenticated)

/// gRPC 服务端鉴权拦截器
///
/// 通过 `GRPC_AUTH_TOKEN` 环境变量配置共享令牌。
/// 令牌未配置时仅告警放行(存量部署兼容), 生产环境必须配置。
pub fn grpc_auth_interceptor(
    req: tonic::Request<()>,
) -> Result<tonic::Request<()>, tonic::Status> {
    // Secret中的值可能包含尾部CRLF, trim避免比较失败
    let expected = std::env::var("GRPC_AUTH_TOKEN" ).unwrap_or_default().trim().to_string();
    if expected.is_empty() {
        tracing::warn!("GRPC_AUTH_TOKEN 未配置, gRPC 服务间鉴权未启用(生产必须配置)" );
        return Ok(req);
    }
    match req
        .metadata()
        .get("x-grpc-token" )
        .and_then(|v| v.to_str().ok())
    {
        Some(token) if token == expected => Ok(req),
        _ => Err(tonic::Status::unauthenticated(
            "missing or invalid gRPC auth token" ,
        )),
    }
}
