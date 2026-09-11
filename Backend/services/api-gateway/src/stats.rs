//! 统计信息处理器
//!
//! 提供系统统计信息的聚合接口
//!
//! 修复说明 (2026-08-03)：原实现通过 HTTP 调用 `{svc}/api/stats/...` 获取统计，
//! 但 user-service / audit-service 的 HTTP 服务只挂载了健康检查路由，这些统计端点
//! 从未存在 → 所有指标静默降级为 0。现改为直接调用网关已接好的 gRPC 客户端
//! （ListUsers / ListLoginLogs），统计真实数据。

use axum::{Json, Router, extract::State, routing::get};
use chrono::{Datelike, TimeZone};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::AppState;

/// 统计数据响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticsResponse {
    /// 总用户数
    pub total_users: i64,
    /// 活跃用户数（本月有登录）
    pub active_users: i64,
    /// 今日新增用户
    pub new_users_today: i64,
    /// 本月登录尝试次数
    pub login_attempts: i64,
    /// 失败登录次数
    pub failed_logins: i64,
    /// 成功登录次数
    pub successful_logins: i64,
}

/// 用户增长数据点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserGrowthPoint {
    /// 日期标签
    pub label: String,
    /// 用户数
    pub value: i64,
}

/// 登录类型分布数据点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginTypePoint {
    /// 登录类型
    pub label: String,
    /// 次数
    pub value: i64,
}

/// 仪表盘统计响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStats {
    /// 基础统计
    pub statistics: StatisticsResponse,
    /// 用户增长趋势（近7天）
    pub user_growth: Vec<UserGrowthPoint>,
    /// 登录类型分布
    pub login_types: Vec<LoginTypePoint>,
}

/// 当前 UTC 时间
fn now_ts() -> i64 {
    chrono::Utc::now().timestamp()
}

/// 今天 00:00:00 UTC 的 epoch 秒
fn today_start_ts() -> i64 {
    let now = chrono::Utc::now();
    chrono::Utc
        .with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0, 0)
        .single()
        .map(|dt| dt.timestamp())
        .unwrap_or_else(now_ts)
}

/// 本月 1 日 00:00:00 UTC 的 epoch 秒
fn month_start_ts() -> i64 {
    let now = chrono::Utc::now();
    chrono::Utc
        .with_ymd_and_hms(now.year(), now.month(), 1, 0, 0, 0)
        .single()
        .map(|dt| dt.timestamp())
        .unwrap_or_else(now_ts)
}

/// epoch 秒 → "YYYY-MM-DD HH:mm:ss" 标签
fn fmt_label(ts: i64) -> String {
    let dt = chrono::DateTime::from_timestamp(ts, 0)
        .unwrap_or_else(|| chrono::DateTime::from_timestamp(0, 0).unwrap());
    dt.format("%m-%d").to_string()
}

/// 获取仪表盘统计数据（供 lib.rs 路由调用）
pub async fn get_dashboard_stats(
    State(state): State<Arc<AppState>>,
) -> Json<common::ApiResponse<DashboardStats>> {
    use common::ApiResponse;

    let today_start = today_start_ts();
    let month_start = month_start_ts();
    let now = now_ts();

    // ---------- 用户统计：gRPC ListUsers ----------
    let mut total_users = 0i64;
    let mut new_users_today = 0i64;
    let mut user_growth: Vec<UserGrowthPoint> = Vec::new();
    let mut login_attempts = 0i64;
    let mut failed_logins = 0i64;
    let mut successful_logins = 0i64;
    let mut active_user_ids = std::collections::HashSet::new();
    let mut login_type_map: std::collections::HashMap<String, i64> = std::collections::HashMap::new();

    if let Ok(mut user_client) = state.grpc_clients.read().await.user_client().await {
        // Paginated fetching: 500 per page, max 5 pages = 2500 users
        let mut all_users = Vec::new();
        let mut page: i32 = 1;
        let page_size: i32 = 500;
        let max_pages = 5;
        loop {
            match user_client.list_users(page, page_size, String::new()).await {
                Ok(resp) => {
                    let count = resp.users.len() as i64;
                    all_users.extend(resp.users);
                    total_users = resp.total;
                    if count < page_size as i64 || page >= max_pages {
                        break;
                    }
                    page += 1;
                }
                Err(e) => {
                    tracing::warn!("【统计】ListUsers 失败: {e}");
                    break;
                }
            }
        }
        // Process all_users
        let mut day_count: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
        for u in &all_users {
            let created = u.created_at;
            if created >= today_start {
                new_users_today += 1;
            }
            *day_count.entry(fmt_label(created)).or_insert(0) += 1;
        }
        // 近 7 天增长（含今天）
        for i in (0..7).rev() {
            let day = today_start - i * 86400;
            user_growth.push(UserGrowthPoint {
                label: fmt_label(day),
                value: *day_count.get(&fmt_label(day)).unwrap_or(&0),
            });
        }
    } else {
        tracing::warn!("【统计】user-service gRPC 不可用");
    }

    // ---------- 登录统计：gRPC ListLoginLogs（本月） ----------
    // audit-service 的 status 语义：0=失败 1=成功 _=全部（内部映射 DB login_status）
    if let Ok(mut audit_client) = state.grpc_clients.read().await.audit_client().await {
        let start = chrono::DateTime::from_timestamp(month_start, 0)
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_default();
        let end = chrono::DateTime::from_timestamp(now, 0)
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_default();

        // 全部登录尝试
        match audit_client
            .list_login_logs(1, 1, 0, String::new(), -1, start.clone(), end.clone())
            .await
        {
            Ok(resp) => login_attempts = resp.total,
            Err(e) => tracing::warn!("【统计】ListLoginLogs(全部) 失败: {e}"),
        }
        // 失败
        match audit_client
            .list_login_logs(1, 1, 0, String::new(), 0, start.clone(), end.clone())
            .await
        {
            Ok(resp) => failed_logins = resp.total,
            Err(e) => tracing::warn!("【统计】ListLoginLogs(失败) 失败: {e}"),
        }
        // 成功
        match audit_client
            .list_login_logs(1, 1, 0, String::new(), 1, start.clone(), end.clone())
            .await
        {
            Ok(resp) => successful_logins = resp.total,
            Err(e) => tracing::warn!("【统计】ListLoginLogs(成功) 失败: {e}"),
        }
        // 活跃用户（本月成功登录的去重 user_id）与登录类型分布（分页拉取，避免全表加载）
        let mut page: i32 = 1;
        let page_size: i32 = 500;
        loop {
            match audit_client
                .list_login_logs(page, page_size, 0, String::new(), 1, start.clone(), end.clone())
                .await
            {
                Ok(resp) => {
                    for l in &resp.logs {
                        if l.user_id > 0 {
                            active_user_ids.insert(l.user_id);
                        }
                        *login_type_map.entry(l.login_method.clone()).or_insert(0) += 1;
                    }
                    if (resp.logs.len() as i32) < page_size || ((page as i64) * (page_size as i64)) >= resp.total {
                        break;
                    }
                    page += 1;
                }
                Err(e) => {
                    tracing::warn!("【统计】ListLoginLogs(活跃) 失败: {e}");
                    break;
                }
            }
        }
    } else {
        tracing::warn!("【统计】audit-service gRPC 不可用");
    }

    let active_users = active_user_ids.len() as i64;
    let mut login_types: Vec<LoginTypePoint> = login_type_map
        .into_iter()
        .map(|(label, value)| LoginTypePoint { label, value })
        .collect();
    login_types.sort_by(|a, b| b.value.cmp(&a.value));

    let dashboard_stats = DashboardStats {
        statistics: StatisticsResponse {
            total_users,
            active_users,
            new_users_today,
            login_attempts,
            failed_logins,
            successful_logins,
        },
        user_growth,
        login_types,
    };

    tracing::info!(
        "【统计】仪表盘 - 用户总数: {total_users}, 今日新增: {new_users_today}, 登录次数: {login_attempts}, 活跃用户: {active_users}"
    );

    Json(ApiResponse::success(dashboard_stats))
}


/// 创建统计路由
pub fn create_stats_router() -> Router<Arc<AppState>> {
    Router::new().route("/dashboard", get(get_dashboard_stats))
}
