//!
//! 工作流处理器
//! 
//! 处理 HTTP 请求

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error};

use crate::models::*;
use crate::engine::WorkflowEngine;
use auth_core::middleware::AuthenticatedUser;
use crate::helpers::{json_success, json_ok, json_error, json_error_fmt, json_success_msg, json_ok_msg, json_error_msg, json_error_msg_fmt};

/// 应用状态
#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::PgPool,
    pub engine: Arc<RwLock<WorkflowEngine>>,
}
/// 列出工作流
pub async fn list_workflows(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    let offset = (page - 1) * page_size;

    let workflows: Vec<Workflow> = sqlx::query_as!(
        Workflow,
        r#"
        SELECT id, name, description, definition, status, version, 
               created_by, created_at, updated_at
        FROM workflows
        WHERE ($1::text IS NULL OR status = $1)
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
        query.status,
        page_size,
        offset
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let total: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM workflows WHERE ($1::text IS NULL OR status = $1)",
        query.status.as_deref(),
    )
    .fetch_one(&state.pool)
    .await
    .ok()
    .flatten()
    .unwrap_or(0);

    json_success({
            "items": workflows,
            "total": total,
            "page": page,
            "page_size": page_size
        })
}

/// 创建工作流
pub async fn create_workflow(
    State(state): State<AppState>,
    Json(payload): Json<CreateWorkflowRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    // 从请求头或认证中获取创建者，默认为 "system"
    // 在实际生产环境中，应从 AuthState 或扩展中获取当前用户
    let created_by = std::env::var("WORKFLOW_CREATED_BY").unwrap_or_else(|_| "system".to_string());
    
    let workflow = Workflow::new(
        payload.name,
        payload.description,
        created_by,
    );

    sqlx::query!(
        r#"
        INSERT INTO workflows (id, name, description, definition, status, version, created_by, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
        &workflow.id,
        &workflow.name,
        workflow.description.as_deref(),
        &workflow.definition,
        &workflow.status,
        workflow.version,
        &workflow.created_by,
        workflow.created_at,
        workflow.updated_at
    )
    .execute(&state.pool)
    .await
    .map_err(|e| {
        error!("创建工作流失败: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, json_error_msg("创建工作流失败"))
    })?;

    info!("创建工作流: {}", workflow.id);

    Ok((StatusCode::CREATED, json_success_msg({ "id": workflow.id }, "工作流创建成功")))
}

/// 获取工作流详情
pub async fn get_workflow(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let workflow: Option<Workflow> = sqlx::query_as!(
        Workflow,
        r#"
        SELECT id, name, description, definition, status, version,
               created_by, created_at, updated_at
        FROM workflows
        WHERE id = $1
        "#,
        &id
    )
    .fetch_optional(&state.pool)
    .await
    .unwrap_or(None);

    match workflow {
        Some(w) => json_success(w),
        None => json_error_msg("工作流不存在"),
    }
}

/// 更新工作流
pub async fn update_workflow(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateWorkflowRequest>,
) -> impl IntoResponse {
    let result = sqlx::query!(
        r#"
        UPDATE workflows 
        SET name = COALESCE($2, name),
            description = COALESCE($3, description),
            definition = COALESCE($4, definition),
            status = COALESCE($5, status),
            updated_at = NOW()
        WHERE id = $1
        "#,
        &id,
        payload.name.as_deref(),
        payload.description.as_deref(),
        payload.definition.as_ref(),
        payload.status.as_deref()
    )
    .execute(&state.pool)
    .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => json_ok_msg("工作流更新成功"),
        _ => json_error_msg("工作流不存在"),
    }
}

/// 删除工作流
pub async fn delete_workflow(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let result = sqlx::query!("DELETE FROM workflows WHERE id = $1", &id)
        .execute(&state.pool)
        .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => json_ok_msg("工作流删除成功"),
        _ => json_error_msg("工作流不存在"),
    }
}

/// 列出实例
pub async fn list_instances(
    State(state): State<AppState>,
    Path(workflow_id): Path<String>,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    let offset = (page - 1) * page_size;

    let instances: Vec<WorkflowInstance> = sqlx::query_as!(
        WorkflowInstance,
        r#"
        SELECT id, workflow_id, workflow_version, status, current_node_id,
               variables, started_by, started_at, completed_at
        FROM workflow_instances
        WHERE workflow_id = $1
        ORDER BY started_at DESC
        LIMIT $2 OFFSET $3
        "#,
        &workflow_id,
        page_size,
        offset
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let total: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM workflow_instances WHERE workflow_id = $1",
        &workflow_id
    )
    .fetch_one(&state.pool)
    .await
    .ok()
    .flatten()
    .unwrap_or(0);

    json_success({
            "items": instances,
            "total": total,
            "page": page,
            "page_size": page_size
        })
}

/// 启动实例
pub async fn start_instance(
    State(state): State<AppState>,
    Path(workflow_id): Path<String>,
    Json(payload): Json<StartInstanceRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    // 获取工作流定义
    let workflow: Option<Workflow> = sqlx::query_as!(
        Workflow,
        r#"
        SELECT id, name, description, definition, status, version,
               created_by, created_at, updated_at
        FROM workflows
        WHERE id = $1
        "#,
        &workflow_id
    )
    .fetch_optional(&state.pool)
    .await
    .unwrap_or(None);

    let workflow = workflow.ok_or_else(|| {
        (StatusCode::NOT_FOUND, json_error_msg("工作流不存在"))
    })?;

    // 从环境变量获取启动者，默认为 "system"
    let started_by = std::env::var("WORKFLOW_STARTED_BY").unwrap_or_else(|_| "system".to_string());
    
    let mut instance = WorkflowInstance::new(
        workflow_id.clone(),
        workflow.version,
        started_by,
    );
    instance.variables = payload.variables.unwrap_or(serde_json::json!({}));

    sqlx::query!(
        r#"
        INSERT INTO workflow_instances (id, workflow_id, workflow_version, status, current_node_id, variables, started_by, started_at, completed_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
        &instance.id,
        &instance.workflow_id,
        instance.workflow_version,
        &instance.status,
        instance.current_node_id.as_deref(),
        &instance.variables,
        &instance.started_by,
        instance.started_at,
        instance.completed_at
    )
    .execute(&state.pool)
    .await
    .map_err(|e| {
        error!("启动实例失败: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, json_error_msg("启动实例失败"))
    })?;

    info!("启动工作流实例: {}", instance.id);

    // 触发工作流引擎执行
    let engine = state.engine.read().await;
    match engine.start_instance(&workflow_id, workflow.definition.clone(), started_by, payload.variables).await {
        Ok(_) => {
            info!("工作流引擎执行成功: {}", instance.id);
        }
        Err(e) => {
            error!("工作流引擎执行失败: {}", e);
            // 实例已创建，引擎执行失败不影响返回
        }
    }

    Ok((StatusCode::CREATED, json_success_msg({ "id": instance.id }, "实例启动成功")))
}

/// 获取实例详情
pub async fn get_instance(
    State(state): State<AppState>,
    Path(instance_id): Path<String>,
) -> impl IntoResponse {
    let instance: Option<WorkflowInstance> = sqlx::query_as!(
        WorkflowInstance,
        r#"
        SELECT id, workflow_id, workflow_version, status, current_node_id,
               variables, started_by, started_at, completed_at
        FROM workflow_instances
        WHERE id = $1
        "#,
        &instance_id
    )
    .fetch_optional(&state.pool)
    .await
    .unwrap_or(None);

    match instance {
        Some(i) => json_success(i),
        None => json_error_msg("实例不存在"),
    }
}

/// 执行动作
pub async fn execute_action(
    State(state): State<AppState>,
    Path(instance_id): Path<String>,
    Json(payload): Json<ExecuteActionRequest>,
) -> impl IntoResponse {
    info!("执行工作流动作: {} - {}", instance_id, payload.action);

    // 获取实例信息
    let instance: Option<WorkflowInstance> = sqlx::query_as!(
        WorkflowInstance,
        r#"
        SELECT id, workflow_id, workflow_version, status, current_node_id,
               variables, started_by, started_at, completed_at
        FROM workflow_instances
        WHERE id = $1
        "#,
        &instance_id
    )
    .fetch_optional(&state.pool)
    .await
    .unwrap_or(None);

    let instance = match instance {
        Some(i) => i,
        None => {
            return json_error_msg("实例不存在");
        }
    };

    // 根据动作类型调用工作流引擎
    let engine = state.engine.read().await;
    match payload.action.as_str() {
        "approve" | "complete" => {
            // 完成任务并流转到下一个节点
            if let Some(current_node) = &instance.current_node_id {
                match engine.complete_node(&instance_id, current_node, payload.variables.clone()).await {
                    Ok(next_node) => {
                        info!("节点完成，下一节点: {:?}", next_node);
                    }
                    Err(e) => {
                        error!("节点完成失败: {}", e);
                        return json_error_msg(&format!("节点完成失败: {);
                    }
                }
            }
        }
        "reject" => {
            // 拒绝任务，取消实例
            if let Err(e) = engine.cancel_instance(&instance_id, payload.comment.clone()).await {
                error!("取消实例失败: {}", e);
                return json_error_msg(&format!("取消实例失败: {);
            }
        }
        "cancel" => {
            // 取消实例
            if let Err(e) = engine.cancel_instance(&instance_id, payload.comment.clone()).await {
                error!("取消实例失败: {}", e);
                return json_error_msg(&format!("取消实例失败: {);
            }
        }
        _ => {
            info!("未知动作类型: {}", payload.action);
        }
    }

    json_ok_msg("动作执行成功")
}

/// 列出任务
pub async fn list_tasks(
    State(state): State<AppState>,
    Path(instance_id): Path<String>,
) -> impl IntoResponse {
    let tasks: Vec<TaskRecord> = sqlx::query_as!(
        TaskRecord,
        r#"
        SELECT id, instance_id, node_id, node_name, assignee, status, 
               comment, form_data, started_at, completed_at, timeout_at
        FROM task_records
        WHERE instance_id = $1
        ORDER BY started_at DESC
        "#,
        &instance_id
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    json_success(tasks)
}

/// 完成任务
pub async fn complete_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    sqlx::query!(
        "UPDATE task_records SET status = 'completed', completed_at = NOW() WHERE id = $1",
        &task_id
    )
    .execute(&state.pool)
    .await
    .ok();

    json_ok_msg("任务完成")
}

/// 拒绝任务
pub async fn reject_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    sqlx::query!(
        "UPDATE task_records SET status = 'rejected', completed_at = NOW() WHERE id = $1",
        &task_id
    )
    .execute(&state.pool)
    .await
    .ok();

    json_ok_msg("任务已拒绝")
}

/// 列出节点
pub async fn list_nodes(
    State(state): State<AppState>,
    Path(workflow_id): Path<String>,
) -> impl IntoResponse {
    let nodes: Vec<WorkflowNode> = sqlx::query_as!(
        WorkflowNode,
        r#"
        SELECT id, workflow_id, name, node_type, position_x, position_y,
               config, timeout, auto_complete, created_at
        FROM workflow_nodes
        WHERE workflow_id = $1
        ORDER BY position_x, position_y
        "#,
        &workflow_id
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    json_success(nodes)
}

/// 创建节点
pub async fn create_node(
    State(state): State<AppState>,
    Path(workflow_id): Path<String>,
    Json(payload): Json<CreateNodeRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    // 解析 node_type，默认为 Task
    let node_type = match payload.node_type.to_lowercase().as_str() {
        "start" => NodeType::Start,
        "end" => NodeType::End,
        "approval" => NodeType::Approval,
        "condition" => NodeType::Condition,
        "parallel" => NodeType::Parallel,
        "merge" => NodeType::Merge,
        _ => NodeType::Task,
    };
    
    let mut node = WorkflowNode::new(
        workflow_id,
        payload.name,
        node_type,
        payload.position_x,
        payload.position_y,
    );
    
    // 设置可选字段
    node.config = payload.config.unwrap_or(serde_json::json!({}));
    node.timeout = payload.timeout;
    node.auto_complete = payload.auto_complete;

    sqlx::query!(
        r#"
        INSERT INTO workflow_nodes (id, workflow_id, name, node_type, position_x, position_y, config, timeout, auto_complete, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#,
        &node.id,
        &node.workflow_id,
        &node.name,
        &node.node_type,
        node.position_x,
        node.position_y,
        &node.config,
        node.timeout,
        node.auto_complete,
        node.created_at
    )
    .execute(&state.pool)
    .await
    .map_err(|e| {
        error!("创建节点失败: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, json_error_msg("创建节点失败"))
    })?;

    Ok((StatusCode::CREATED, json_success_msg({ "id": node.id }, "节点创建成功")))
}

/// 更新节点
pub async fn update_node(
    State(state): State<AppState>,
    Path((_, node_id)): Path<(String, String)>,
    Json(payload): Json<UpdateNodeRequest>,
) -> impl IntoResponse {
    sqlx::query!(
        r#"
        UPDATE workflow_nodes
        SET name = COALESCE($2, name),
            config = COALESCE($3, config),
            timeout = COALESCE($4, timeout),
            auto_complete = COALESCE($5, auto_complete)
        WHERE id = $1
        "#,
        &node_id,
        payload.name.as_deref(),
        payload.config.as_ref(),
        payload.timeout,
        payload.auto_complete
    )
    .execute(&state.pool)
    .await
    .ok();

    json_ok_msg("节点更新成功")
}

/// 删除节点
pub async fn delete_node(
    State(state): State<AppState>,
    Path((_, node_id)): Path<(String, String)>,
) -> impl IntoResponse {
    sqlx::query!("DELETE FROM workflow_nodes WHERE id = $1", &node_id)
        .execute(&state.pool)
        .await
        .ok();

    json_ok_msg("节点删除成功")
}

/// 列出边
pub async fn list_edges(
    State(state): State<AppState>,
    Path(workflow_id): Path<String>,
) -> impl IntoResponse {
    let edges: Vec<WorkflowEdge> = sqlx::query_as!(
        WorkflowEdge,
        r#"
        SELECT id, workflow_id, source_node_id, target_node_id, edge_type,
               condition, label, priority, created_at
        FROM workflow_edges
        WHERE workflow_id = $1
        ORDER BY priority
        "#,
        &workflow_id
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    json_success(edges)
}

/// 创建边
pub async fn create_edge(
    State(state): State<AppState>,
    Path(workflow_id): Path<String>,
    Json(payload): Json<CreateEdgeRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let mut edge = WorkflowEdge::new(
        workflow_id,
        payload.source_node_id,
        payload.target_node_id,
    );

    if let Some(edge_type) = payload.edge_type {
        edge.edge_type = edge_type;
    }
    edge.condition = payload.condition;
    edge.label = payload.label;

    sqlx::query!(
        r#"
        INSERT INTO workflow_edges (id, workflow_id, source_node_id, target_node_id, edge_type, condition, label, priority, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
        &edge.id,
        &edge.workflow_id,
        &edge.source_node_id,
        &edge.target_node_id,
        &edge.edge_type,
        edge.condition.as_deref(),
        edge.label.as_deref(),
        edge.priority,
        edge.created_at
    )
    .execute(&state.pool)
    .await
    .map_err(|e| {
        error!("创建边失败: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, json_error_msg("创建边失败"))
    })?;

    Ok((StatusCode::CREATED, json_success_msg({ "id": edge.id }, "边创建成功")))
}

/// 更新边
pub async fn update_edge(
    State(state): State<AppState>,
    Path((_, edge_id)): Path<(String, String)>,
    Json(payload): Json<UpdateEdgeRequest>,
) -> impl IntoResponse {
    let result = sqlx::query!(
        r#"
        UPDATE workflow_edges
        SET edge_type = COALESCE($2, edge_type),
            condition = COALESCE($3, condition),
            label = COALESCE($4, label),
            priority = COALESCE($5, priority)
        WHERE id = $1
        "#,
        &edge_id,
        payload.edge_type.as_deref(),
        payload.condition.as_deref(),
        payload.label.as_deref(),
        payload.priority
    )
    .execute(&state.pool)
    .await;

    match result {
        Ok(r) if r.rows_affected() > 0 => json_ok_msg("边更新成功"),
        _ => json_error_msg("边不存在"),
    }
}

/// 删除边
pub async fn delete_edge(
    State(state): State<AppState>,
    Path((_, edge_id)): Path<(String, String)>,
) -> impl IntoResponse {
    sqlx::query!("DELETE FROM workflow_edges WHERE id = $1", &edge_id)
        .execute(&state.pool)
        .await
        .ok();

    json_ok_msg("边删除成功")
}
