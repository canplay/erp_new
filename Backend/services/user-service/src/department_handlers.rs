//! 部门管理 HTTP Handlers

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

use crate::http_handlers::HttpAppState;
use crate::repository::DepartmentRepositoryError;
use crate::helpers::{json_success, json_ok, json_error, json_error_fmt, json_success_msg, json_ok_msg, json_error_msg, json_error_msg_fmt};
use common::AppError;

// ============ 请求/响应结构 ============

/// 部门查询参数
#[derive(Debug, Deserialize)]
pub struct DepartmentQueryParams {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub keyword: Option<String>,
}

/// 创建部门请求
#[derive(Debug, Deserialize)]
pub struct CreateDepartmentRequest {
    pub name: String,
    pub code: Option<String>,
    pub parent_id: Option<i64>,
    pub leader_id: Option<i64>,
    pub description: Option<String>,
    pub sort_order: Option<i32>,
}

/// 更新部门请求
#[derive(Debug, Deserialize)]
pub struct UpdateDepartmentRequest {
    pub name: Option<String>,
    pub code: Option<String>,
    pub parent_id: Option<i64>,
    pub leader_id: Option<i64>,
    pub description: Option<String>,
    pub sort_order: Option<i32>,
    pub status: Option<i32>,
}

/// 移动部门请求
#[derive(Debug, Deserialize)]
pub struct MoveDepartmentRequest {
    pub new_parent_id: Option<i64>,
}

/// 部门响应
#[derive(Debug, Serialize)]
pub struct DepartmentResponse {
    pub id: i64,
    pub name: String,
    pub code: Option<String>,
    pub parent_id: Option<i64>,
    pub level: i32,
    pub sort_order: i32,
    pub leader_id: Option<i64>,
    pub leader_name: Option<String>,
    pub description: Option<String>,
    pub status: i32,
    pub created_at: String,
}

/// 部门列表响应
#[derive(Debug, Serialize)]
pub struct DepartmentListResponse {
    pub list: Vec<DepartmentResponse>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

/// 部门树节点响应
#[derive(Debug, Serialize)]
pub struct DepartmentTreeResponse {
    pub id: i64,
    pub name: String,
    pub code: Option<String>,
    pub level: i32,
    pub sort_order: i32,
    pub leader_id: Option<i64>,
    pub leader_name: Option<String>,
    pub user_count: i64,
    pub children: Vec<Self>,
}

// ============ 处理器实现 ============

/// 获取部门列表
pub async fn list_departments(
    State(state): State<HttpAppState>,
    Query(params): Query<DepartmentQueryParams>,
) -> impl IntoResponse {
    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(10).clamp(1, 100);

    match state
        .department_repository
        .list(page, page_size, params.keyword.as_deref())
        .await
    {
        Ok(result) => {
            let departments: Vec<DepartmentResponse> = result
                .departments
                .into_iter()
                .map(|d| DepartmentResponse {
                    id: d.id,
                    name: d.name,
                    code: d.code,
                    parent_id: d.parent_id,
                    level: d.level,
                    sort_order: d.sort_order,
                    leader_id: d.leader_id,
                    leader_name: d.leader_name,
                    description: None,
                    status: d.status,
                    created_at: d.created_at.to_rfc3339(),
                })
                .collect();

            (
                StatusCode::OK,
                json_success(serde_json::json!({
                    "list": departments,
                    "total": result.total,
                    "page": page,
                    "page_size": page_size
                })),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("查询部门列表失败: {e}" );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("查询部门列表失败" ),
            )
                .into_response()
        }
    }
}

/// 获取部门树
pub async fn get_department_tree(State(state): State<HttpAppState>) -> impl IntoResponse {
    match state.department_repository.get_tree().await {
        Ok(tree) => {
            // 递归转换树节点
            fn convert_node(node: crate::repository::DepartmentTreeNode) -> DepartmentTreeResponse {
                DepartmentTreeResponse {
                    id: node.id,
                    name: node.name,
                    code: node.code,
                    level: node.level,
                    sort_order: node.sort_order,
                    leader_id: node.leader_id,
                    leader_name: node.leader_name,
                    user_count: node.user_count,
                    children: node.children.into_iter().map(convert_node).collect(),
                }
            }

            let tree_response: Vec<DepartmentTreeResponse> =
                tree.into_iter().map(convert_node).collect();

            (
                StatusCode::OK,
                json_success(tree_response),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("获取部门树失败: {e}" );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("获取部门树失败" ),
            )
                .into_response()
        }
    }
}

/// 获取部门详情
pub async fn get_department(
    State(state): State<HttpAppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    match state.department_repository.find_by_id(id).await {
        Ok(Some(dept)) => (
            StatusCode::OK,
            json_success(serde_json::json!({
                "id": dept.id,
                "name": dept.name,
                "code": dept.code,
                "parent_id": dept.parent_id,
                "level": dept.level,
                "sort_order": dept.sort_order,
                "leader_id": dept.leader_id,
                "leader_name": dept.leader_name,
                "description": dept.description,
                "status": dept.status,
                "created_at": dept.created_at.to_rfc3339(),
                "updated_at": dept.updated_at.to_rfc3339()
            })),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            json_error("部门不存在" ),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("查询部门失败: {e}" );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("查询部门失败" ),
            )
                .into_response()
        }
    }
}

/// 创建部门
pub async fn create_department(
    State(state): State<HttpAppState>,
    Json(req): Json<CreateDepartmentRequest>,
) -> impl IntoResponse {
    if req.name.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            json_error("部门名称不能为空" ),
        )
            .into_response();
    }

    match state
        .department_repository
        .create(
            &req.name,
            req.code.as_deref(),
            req.parent_id,
            req.leader_id,
            req.description.as_deref(),
            req.sort_order,
        )
        .await
    {
        Ok(id) => (
            StatusCode::CREATED,
            json_success(serde_json::json!({
                "id": id,
                "name": req.name
            })),
        )
            .into_response(),
        Err(DepartmentRepositoryError::AlreadyExists) => (
            StatusCode::CONFLICT,
            json_error("部门代码已存在" ),
        )
            .into_response(),
        Err(DepartmentRepositoryError::MaxLevelExceeded) => (
            StatusCode::BAD_REQUEST,
            json_error("部门层级不能超过5级" ),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("创建部门失败: {e}" );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("创建部门失败" ),
            )
                .into_response()
        }
    }
}

/// 更新部门
pub async fn update_department(
    State(state): State<HttpAppState>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateDepartmentRequest>,
) -> impl IntoResponse {
    match state
        .department_repository
        .update(
            id,
            req.name,
            req.code,
            req.parent_id,
            req.leader_id,
            req.description,
            req.sort_order,
            req.status,
        )
        .await
    {
        Ok(Some(dept)) => (
            StatusCode::OK,
            json_success(serde_json::json!({
                "id": dept.id,
                "name": dept.name,
                "code": dept.code
            })),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            json_error("部门不存在" ),
        )
            .into_response(),
        Err(DepartmentRepositoryError::CircularReference) => (
            StatusCode::BAD_REQUEST,
            json_error("不能将自己或子部门设为父部门" ),
        )
            .into_response(),
        Err(DepartmentRepositoryError::MaxLevelExceeded) => (
            StatusCode::BAD_REQUEST,
            json_error("部门层级不能超过5级" ),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("更新部门失败: {e}" );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("更新部门失败" ),
            )
                .into_response()
        }
    }
}

/// 删除部门
pub async fn delete_department(
    State(state): State<HttpAppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    match state.department_repository.delete(id).await {
        Ok(true) => (
            StatusCode::OK,
            json_ok_msg("部门删除成功" ),
        )
            .into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            json_error("部门不存在" ),
        )
            .into_response(),
        Err(DepartmentRepositoryError::HasChildDepartments) => (
            StatusCode::CONFLICT,
            json_error("部门有子部门，无法删除" ),
        )
            .into_response(),
        Err(DepartmentRepositoryError::HasAssociatedUsers) => (
            StatusCode::CONFLICT,
            json_error("部门有用户关联，无法删除" ),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("删除部门失败: {e}" );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("删除部门失败" ),
            )
                .into_response()
        }
    }
}

/// 移动部门
pub async fn move_department(
    State(state): State<HttpAppState>,
    Path(id): Path<i64>,
    Json(req): Json<MoveDepartmentRequest>,
) -> impl IntoResponse {
    match state
        .department_repository
        .move_department(id, req.new_parent_id)
        .await
    {
        Ok(true) => (
            StatusCode::OK,
            json_ok_msg("部门移动成功" ),
        )
            .into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            json_error("部门不存在" ),
        )
            .into_response(),
        Err(DepartmentRepositoryError::CircularReference) => (
            StatusCode::BAD_REQUEST,
            json_error("不能将自己或子部门设为父部门" ),
        )
            .into_response(),
        Err(DepartmentRepositoryError::MaxLevelExceeded) => (
            StatusCode::BAD_REQUEST,
            json_error("部门层级不能超过5级" ),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("移动部门失败: {e}" );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("移动部门失败" ),
            )
                .into_response()
        }
    }
}

/// 获取部门下的用户
pub async fn get_department_users(
    State(state): State<HttpAppState>,
    Path(id): Path<i64>,
    Query(params): Query<DepartmentQueryParams>,
) -> impl IntoResponse {
    let page = params.page.unwrap_or(1).max(1);
    let page_size = params.page_size.unwrap_or(10).clamp(1, 100);

    match state
        .department_repository
        .get_users(id, page, page_size)
        .await
    {
        Ok((user_ids, total)) => (
            StatusCode::OK,
            json_success(serde_json::json!({
                "user_ids": user_ids,
                "total": total,
                "page": page,
                "page_size": page_size
            })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("获取部门用户失败: {e}" );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json_error("获取部门用户失败" ),
            )
                .into_response()
        }
    }
}

// ============ 路由构建 ============

/// 创建部门管理路由
pub fn create_department_router(state: HttpAppState) -> Router {
    Router::new()
        .route("/" , axum::routing::get(list_departments))
        .route("/" , axum::routing::post(create_department))
        .route("/tree" , axum::routing::get(get_department_tree))
        .route("/{id}" , axum::routing::get(get_department))
        .route("/{id}" , axum::routing::put(update_department))
        .route("/{id}" , axum::routing::delete(delete_department))
        .route("/{id}/move" , axum::routing::post(move_department))
        .route("/{id}/users" , axum::routing::get(get_department_users))
        .with_state(state)
}
