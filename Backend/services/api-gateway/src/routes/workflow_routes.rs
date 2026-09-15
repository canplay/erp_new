//! 工作流路由 — 真实 gRPC 调用 workflow-service

use std::sync::Arc;
use axum::{Router, extract::{Query, Path, State}, routing::{get, post, put}, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use crate::AppState;
use crate::routes::helpers::*;

#[derive(Deserialize)]
struct WfQuery { page: Option<i32>, page_size: Option<i32>, keyword: Option<String> }

/// 获取 workflow-service gRPC 客户端
async fn get_wf_client(state: &Arc<AppState>) -> Result<crate::grpc_clients::WorkflowGrpcClient, Json<Value>> {
    state.grpc_clients.read().await.workflow_client().await
        .map_err(|e| json_error(&format!("workflow-service 不可用: {e}")))
}

/// proto WorkflowDefinition → JSON
fn workflow_to_json(w: &grpc_proto::workflow::WorkflowDefinition) -> Value {
    json!({
        "id": w.id,
        "name": w.name,
        "description": w.description,
        "status": w.status,
        "version": w.version,
        "variables": w.variables,
        "created_at": w.created_at,
        "updated_at": w.updated_at,
    })
}

/// proto WorkflowInstance → JSON
fn instance_to_json(i: &grpc_proto::workflow::WorkflowInstance) -> Value {
    json!({
        "id": i.id,
        "workflow_id": i.workflow_id,
        "workflow_name": i.workflow_name,
        "status": i.status,
        "started_by": i.started_by,
        "started_at": i.started_at,
        "finished_at": i.finished_at,
    })
}

/// proto WorkflowTask → JSON
fn task_to_json(t: &grpc_proto::workflow::WorkflowTask) -> Value {
    json!({
        "id": t.id,
        "instance_id": t.instance_id,
        "node_id": t.node_id,
        "assignee": t.assignee,
        "status": t.status,
        "created_at": t.created_at,
        "completed_at": t.completed_at,
    })
}

async fn list_workflows(
    State(state): State<Arc<AppState>>,
    Query(q): Query<WfQuery>,
) -> Json<Value> {
    let mut client = match get_wf_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.list_workflows(q.page.unwrap_or(1), q.page_size.unwrap_or(20), q.keyword.clone().unwrap_or_default(), -1).await {
        Ok(resp) => json_success(json!({
            "list": resp.workflows.iter().map(workflow_to_json).collect::<Vec<_>>(),
            "total": resp.total,
            "page": q.page.unwrap_or(1),
            "page_size": q.page_size.unwrap_or(20),
        })),
        Err(e) => json_error(&format!("查询失败: {e}")),
    }
}

async fn create_workflow(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let mut client = match get_wf_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.create_workflow(
        body["name"].as_str().unwrap_or("").to_string(),
        body["description"].as_str().unwrap_or("").to_string(),
        vec![],
        std::collections::HashMap::new(),
    ).await {
        Ok(resp) => json_success(json!({"id": resp.id})),
        Err(e) => json_error(&format!("创建失败: {e}")),
    }
}

async fn get_workflow(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    let id_val = id.parse::<i64>().unwrap_or(0);
    let mut client = match get_wf_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.get_workflow(id_val).await {
        Ok(resp) => {
            if let Some(w) = resp.workflow {
                json_success(workflow_to_json(&w))
            } else {
                json_error("工作流不存在")
            }
        }
        Err(e) => json_error(&format!("查询失败: {e}")),
    }
}

async fn update_workflow(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let id_val = id.parse::<i64>().unwrap_or(0);
    let mut client = match get_wf_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.update_workflow(
        id_val,
        body["name"].as_str().unwrap_or("").to_string(),
        body["description"].as_str().unwrap_or("").to_string(),
        vec![],
        std::collections::HashMap::new(),
        -1,
    ).await {
        Ok(_) => json_ok(),
        Err(e) => json_error(&format!("更新失败: {e}")),
    }
}

async fn delete_workflow(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    let id_val = id.parse::<i64>().unwrap_or(0);
    let mut client = match get_wf_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.delete_workflow(id_val).await {
        Ok(_) => json_ok(),
        Err(e) => json_error(&format!("删除失败: {e}")),
    }
}

async fn list_instances(
    State(state): State<Arc<AppState>>,
    Query(q): Query<WfQuery>,
) -> Json<Value> {
    let mut client = match get_wf_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.list_instances(0, q.page.unwrap_or(1), q.page_size.unwrap_or(20), -1, String::new()).await {
        Ok(resp) => json_success(json!({
            "list": resp.instances.iter().map(instance_to_json).collect::<Vec<_>>(),
            "total": resp.total,
        })),
        Err(e) => json_error(&format!("查询失败: {e}")),
    }
}

async fn get_instance(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    let id_val = id.parse::<i64>().unwrap_or(0);
    let mut client = match get_wf_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.get_instance(id_val).await {
        Ok(resp) => {
            if let Some(i) = resp.instance {
                json_success(instance_to_json(&i))
            } else {
                json_error("实例不存在")
            }
        }
        Err(e) => json_error(&format!("查询失败: {e}")),
    }
}

async fn start_workflow(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    let id_val = id.parse::<i64>().unwrap_or(0);
    let mut client = match get_wf_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.start_workflow(id_val, String::new(), std::collections::HashMap::new()).await {
        Ok(resp) => json_success(json!({"instance_id": resp.instance_id})),
        Err(e) => json_error(&format!("启动失败: {e}")),
    }
}

async fn cancel_instance(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    let id_val = id.parse::<i64>().unwrap_or(0);
    let mut client = match get_wf_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.cancel_instance(id_val, String::new()).await {
        Ok(_) => json_ok(),
        Err(e) => json_error(&format!("取消失败: {e}")),
    }
}

async fn list_tasks(
    State(state): State<Arc<AppState>>,
    Query(q): Query<WfQuery>,
) -> Json<Value> {
    let mut client = match get_wf_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.list_tasks(0, String::new(), -1, q.page.unwrap_or(1), q.page_size.unwrap_or(20)).await {
        Ok(resp) => json_success(json!({
            "list": resp.tasks.iter().map(task_to_json).collect::<Vec<_>>(),
            "total": resp.total,
        })),
        Err(e) => json_error(&format!("查询失败: {e}")),
    }
}

async fn complete_task(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let id_val = id.parse::<i64>().unwrap_or(0);
    let mut client = match get_wf_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.complete_task(id_val, body["result"].as_str().unwrap_or("approve").to_string(), std::collections::HashMap::new(), String::new()).await {
        Ok(_) => json_ok(),
        Err(e) => json_error(&format!("完成任务失败: {e}")),
    }
}

async fn reject_task(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let id_val = id.parse::<i64>().unwrap_or(0);
    let mut client = match get_wf_client(&state).await { Ok(c) => c, Err(r) => return r };
    match client.complete_task(id_val, body["reason"].as_str().unwrap_or("reject").to_string(), std::collections::HashMap::new(), String::new()).await {
        Ok(_) => json_ok(),
        Err(e) => json_error(&format!("拒绝任务失败: {e}")),
    }
}

// ============ 工作流节点/连线（workflow.proto 暂未提供 RPC，保留占位） ============

async fn list_workflow_nodes(Path(_wf_id): Path<String>) -> AppResult<Json<Value>> {
    tracing::warn!("list_workflow_nodes: 功能未实现");
    Err(AppError::NotImplemented("list_workflow_nodes: 功能未实现".to_string()))
}
async fn create_workflow_node() -> Json<Value> { json_success(json!({"id": ""})) }
async fn update_workflow_node(Path(_id): Path<String>) -> Json<Value> { json_ok() }
async fn delete_workflow_node(Path(_id): Path<String>) -> Json<Value> { json_ok() }

async fn list_workflow_edges(Path(_wf_id): Path<String>) -> Json<Value> {
    json_success(json!({"list": [], "total": 0}))
}
async fn create_workflow_edge() -> Json<Value> { json_success(json!({"id": ""})) }
async fn update_workflow_edge(Path(_id): Path<String>) -> Json<Value> { json_ok() }
async fn delete_workflow_edge(Path(_id): Path<String>) -> Json<Value> { json_ok() }

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/workflows", get(list_workflows).post(create_workflow))
        .route("/api/workflows/{id}", get(get_workflow).put(update_workflow).delete(delete_workflow))
        .route("/api/workflows/{id}/start", post(start_workflow))
        .route("/api/workflows/instances", post(create_workflow).get(list_instances))
        .route("/api/workflows/instances/{id}", post(get_instance))
        .route("/api/workflows/tasks/{id}/complete", post(complete_task))
        .route("/api/workflows/tasks/{id}/reject", post(reject_task))
        .route("/api/workflows/{id}/publish", put(update_workflow))
        .route("/api/workflows/{id}/instances", get(list_instances))
        .route("/api/workflow-instances", get(list_instances))
        .route("/api/workflow-instances/{id}", get(get_instance).delete(cancel_instance))
        .route("/api/workflow-instances/{id}/cancel", post(cancel_instance))
        .route("/api/workflow-tasks", get(list_tasks))
        .route("/api/workflow-tasks/{id}", get(get_instance).put(complete_task))
        .route("/api/workflow-tasks/{id}/complete", post(complete_task))
        .route("/api/workflow-tasks/{id}/reject", post(reject_task))
        .route("/api/workflows/{wfId}/nodes", get(list_workflow_nodes))
        .route("/api/workflows/nodes", post(create_workflow_node))
        .route("/api/workflows/nodes/{id}", put(update_workflow_node).delete(delete_workflow_node))
        .route("/api/workflows/{wfId}/edges", get(list_workflow_edges))
        .route("/api/workflows/edges", post(create_workflow_edge))
        .route("/api/workflows/edges/{id}", put(update_workflow_edge).delete(delete_workflow_edge))
}
