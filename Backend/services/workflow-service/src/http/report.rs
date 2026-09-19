use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};

use super::{AppState, CreateReportRequest, ListQuery, PaginatedData};
use crate::repository::event_repository::ReportRepository;
use crate::repository::Report;

/// 报表响应
#[derive(Debug, serde::Serialize)]
pub struct ReportResponse {
    pub id: String,
    pub name: String,
    pub report_type: String,
    pub status: String,
    pub created_at: String,
    pub generated_at: Option<String>,
}

/// 获取报表列表
pub async fn list_reports(
    State(state): State<AppState>,
    Query(params): Query<ListQuery>,
) -> impl IntoResponse {
    let page = i64::from(params.page.unwrap_or(1));
    let page_size = i64::from(params.page_size.unwrap_or(10));

    let Ok((reports, total)) = state.report_repo.list(page, page_size).await else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "message": "查询失败"
            })),
        );
    };

    let list: Vec<ReportResponse> = reports
        .into_iter()
        .map(|r| ReportResponse {
            id: r.id,
            name: r.name,
            report_type: r.report_type,
            status: r.status,
            created_at: r.created_at.to_rfc3339(),
            generated_at: r.generated_at.map(|dt| dt.to_rfc3339()),
        })
        .collect();

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "data": PaginatedData {
                list,
                total,
                page,
                page_size,
            }
        })),
    )
}

/// 获取单个报表
pub async fn get_report(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    match state.report_repo.find_by_id(&id).await {
        Ok(Some(report)) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "data": ReportResponse {
                    id: report.id,
                    name: report.name,
                    report_type: report.report_type,
                    status: report.status,
                    created_at: report.created_at.to_rfc3339(),
                    generated_at: report.generated_at.map(|dt| dt.to_rfc3339()),
                }
            })),
        ),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "success": false,
                "message": "报表不存在"
            })),
        ),
        Err(e) => {
            tracing::error!("查询报表失败: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "message": "查询失败"
                })),
            )
        }
    }
}

/// 创建报表
pub async fn create_report(
    State(state): State<AppState>,
    Json(req): Json<CreateReportRequest>,
) -> impl IntoResponse {
    let report = Report::new(
        req.name,
        req.report_type,
        req.query_params,
        "system".to_string(),
    );

    match state.report_repo.create(&report).await {
        Ok(()) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "success": true,
                "data": {
                    "id": report.id,
                    "name": report.name
                }
            })),
        ),
        Err(e) => {
            tracing::error!("创建报表失败: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "message": "创建失败"
                })),
            )
        }
    }
}

/// 生成报表
pub async fn generate_report(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    tracing::info!("【HTTP】生成报表: {id}");

    match state.report_repo.find_by_id(&id).await {
        Ok(Some(report)) => {
            tracing::info!(
                "【HTTP】报表生成中: {}, type={}",
                report.name,
                report.report_type
            );

            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "success": true,
                    "message": "报表生成中",
                    "data": {
                        "report_id": report.id,
                        "report_type": report.report_type,
                        "status": "generating"
                    }
                })),
            )
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "success": false,
                "message": "报表不存在"
            })),
        ),
        Err(e) => {
            tracing::error!("【HTTP】获取报表失败: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "message": "生成失败"
                })),
            )
        }
    }
}

/// 下载报表
pub async fn download_report(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    tracing::info!("【HTTP】下载报表: {id}");

    match state.report_repo.find_by_id(&id).await {
        Ok(Some(report)) => {
            if report.result.is_none() {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "success": false,
                        "message": "报表尚未生成"
                    })),
                );
            }

            tracing::info!(
                "【HTTP】报表下载准备: {}, generated_at={:?}",
                report.name,
                report.generated_at
            );

            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "success": true,
                    "message": "报表准备就绪",
                    "data": {
                        "report_id": report.id,
                        "name": report.name,
                        "report_type": report.report_type,
                        "result": report.result,
                        "generated_at": report.generated_at.map(|dt| dt.to_rfc3339())
                    }
                })),
            )
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "success": false,
                "message": "报表不存在"
            })),
        ),
        Err(e) => {
            tracing::error!("【HTTP】获取报表失败: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "success": false,
                    "message": "下载失败"
                })),
            )
        }
    }
}
