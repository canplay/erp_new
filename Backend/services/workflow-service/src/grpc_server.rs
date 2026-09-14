//! Workflow Service gRPC Server
//!
//! 实现 workflow.proto 中定义的 gRPC 服务 trait

use std::net::SocketAddr;
use grpc_proto::workflow::workflow_service_server::WorkflowServiceServer;
use std::collections::HashMap;
use tonic::{Request, Response, Status};

use grpc_proto::workflow::{
    CancelInstanceRequest, CancelInstanceResponse, CompleteTaskRequest, CompleteTaskResponse,
    CreateWorkflowRequest, CreateWorkflowResponse, DeleteWorkflowRequest, DeleteWorkflowResponse,
    GetInstanceRequest, GetInstanceResponse, GetWorkflowRequest, GetWorkflowResponse,
    InstanceStatus as ProtoInstanceStatus, ListInstancesRequest, ListInstancesResponse,
    ListTasksRequest, ListTasksResponse, ListWorkflowsRequest, ListWorkflowsResponse,
    NodeType as ProtoNodeType, StartWorkflowRequest, StartWorkflowResponse,
    TaskStatus as ProtoTaskStatus, UpdateWorkflowRequest, UpdateWorkflowResponse,
    WorkflowDefinition, WorkflowInstance as ProtoWorkflowInstance, WorkflowNode,
    WorkflowStatus as ProtoWorkflowStatus, WorkflowTask, workflow_service_server::WorkflowService,
};

/// `WorkflowService` 实现
#[derive(Clone)]
pub struct WorkflowGrpcServer {
    pool: sqlx::PgPool,
}

impl WorkflowGrpcServer {
    /// 创建新的 `WorkflowService` 实例
    #[must_use]
    pub fn new(pool: &sqlx::PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    /// 将数据库状态转换为 proto 状态
    fn workflow_status_to_proto(status: &str) -> ProtoWorkflowStatus {
        match status {
            "draft" | "DRAFT" => ProtoWorkflowStatus::WorkflowDraft,
            "active" | "ACTIVE" => ProtoWorkflowStatus::WorkflowActive,
            "suspended" | "SUSPENDED" => ProtoWorkflowStatus::WorkflowSuspended,
            "archived" | "ARCHIVED" => ProtoWorkflowStatus::WorkflowArchived,
            _ => ProtoWorkflowStatus::WorkflowDraft,
        }
    }

    fn instance_status_to_proto(status: &str) -> ProtoInstanceStatus {
        match status {
            "running" | "RUNNING" => ProtoInstanceStatus::InstanceRunning,
            "completed" | "COMPLETED" => ProtoInstanceStatus::InstanceCompleted,
            "cancelled" | "CANCELLED" => ProtoInstanceStatus::InstanceCancelled,
            "failed" | "FAILED" => ProtoInstanceStatus::InstanceFailed,
            _ => ProtoInstanceStatus::InstanceRunning,
        }
    }

    fn task_status_to_proto(status: &str) -> ProtoTaskStatus {
        match status {
            "pending" | "PENDING" => ProtoTaskStatus::TaskPending,
            "completed" | "COMPLETED" => ProtoTaskStatus::TaskCompleted,
            "rejected" | "REJECTED" => ProtoTaskStatus::TaskRejected,
            "skipped" | "SKIPPED" => ProtoTaskStatus::TaskSkipped,
            _ => ProtoTaskStatus::TaskPending,
        }
    }

}

/// 工作流 ID 与名称（start_workflow 查询结果）
struct WorkflowIdName {
    id: String,
    name: String,
}

#[tonic::async_trait]
impl WorkflowService for WorkflowGrpcServer {
    // ============== 工作流定义相关 ==============

    async fn list_workflows(
        &self,
        request: Request<ListWorkflowsRequest>,
    ) -> Result<Response<ListWorkflowsResponse>, Status> {
        let req = request.into_inner();

        let page = if req.page > 0 { req.page } else { 1 };
        let page_size = if req.page_size > 0 { req.page_size } else { 20 };
        let offset = (page - 1) * page_size;

        // 将 WorkflowStatus 枚举转换为字符串
        let status_str = match req.status() {
            ProtoWorkflowStatus::WorkflowDraft => "draft",
            ProtoWorkflowStatus::WorkflowActive => "active",
            ProtoWorkflowStatus::WorkflowSuspended => "suspended",
            ProtoWorkflowStatus::WorkflowArchived => "archived",
        };

        let rows = sqlx::query!(
            r"SELECT id, name, description, definition, status, version,
       created_by, created_at, updated_at
       FROM workflows
       WHERE ($1::text IS NULL OR $1 = '' OR status = $1)
       ORDER BY created_at DESC
       LIMIT $2 OFFSET $3",
            status_str,
            i64::from(page_size),
            i64::from(offset),
        )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Status::internal(format!("Database error: {e}")))?;

        let workflows: Vec<WorkflowDefinition> = rows
            .iter()
            .map(|row| {
                let nodes: Vec<WorkflowNode> = row
                    .definition
                    .get("nodes")
                    .and_then(|n| n.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|n| {
                                Some(WorkflowNode {
                                    id: n.get("id")?.as_str()?.to_string(),
                                    name: n.get("name")?.as_str()?.to_string(),
                                    node_type: ProtoNodeType::NodeTask as i32,
                                    position_x: n.get("x")?.as_i64().unwrap_or(0) as i32,
                                    position_y: n.get("y")?.as_i64().unwrap_or(0) as i32,
                                    config: std::collections::HashMap::new(),
                                    outputs: vec![],
                                })
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                WorkflowDefinition {
                    id: row.id.parse::<i64>().unwrap_or(0),
                    name: row.name.clone(),
                    description: row.description.clone().unwrap_or_default(),
                    status: Self::workflow_status_to_proto(&row.status) as i32,
                    version: row.version.to_string(),
                    nodes,
                    variables: std::collections::HashMap::new(),
                    created_at: row.created_at.timestamp(),
                    updated_at: row.updated_at.timestamp(),
                }
            })
            .collect();

        let total: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM workflows WHERE ($1::text IS NULL OR $1 = '' OR status = $1)",
            status_str,
        )
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Status::internal(format!("Database error: {e}")))?
            .unwrap_or(0);

        Ok(Response::new(ListWorkflowsResponse { workflows, total }))
    }

    async fn get_workflow(
        &self,
        request: Request<GetWorkflowRequest>,
    ) -> Result<Response<GetWorkflowResponse>, Status> {
        let req = request.into_inner();

        let row = sqlx::query!(
            r"SELECT id, name, description, definition, status, version,
       created_by, created_at, updated_at
       FROM workflows WHERE id = $1",
            req.id.to_string(),
        )
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| Status::internal(format!("Database error: {e}")))?;

        match row {
            Some(row) => {
                let nodes: Vec<WorkflowNode> = row
                    .definition
                    .get("nodes")
                    .and_then(|n| n.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|n| {
                                Some(WorkflowNode {
                                    id: n.get("id")?.as_str()?.to_string(),
                                    name: n.get("name")?.as_str()?.to_string(),
                                    node_type: ProtoNodeType::NodeTask as i32,
                                    position_x: n.get("x")?.as_i64().unwrap_or(0) as i32,
                                    position_y: n.get("y")?.as_i64().unwrap_or(0) as i32,
                                    config: std::collections::HashMap::new(),
                                    outputs: vec![],
                                })
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                let workflow = WorkflowDefinition {
                    id: row.id.parse::<i64>().unwrap_or(0),
                    name: row.name.clone(),
                    description: row.description.clone().unwrap_or_default(),
                    status: Self::workflow_status_to_proto(&row.status) as i32,
                    version: row.version.to_string(),
                    nodes,
                    variables: std::collections::HashMap::new(),
                    created_at: row.created_at.timestamp(),
                    updated_at: row.updated_at.timestamp(),
                };

                Ok(Response::new(GetWorkflowResponse {
                    workflow: Some(workflow),
                }))
            }
            None => Err(Status::not_found("Workflow not found")),
        }
    }

    async fn create_workflow(
        &self,
        request: Request<CreateWorkflowRequest>,
    ) -> Result<Response<CreateWorkflowResponse>, Status> {
        let req = request.into_inner();

        let id = uuid::Uuid::new_v4().as_u64_pair().0 as i64;
        let definition = serde_json::json!({
            "nodes": req.nodes.iter().map(|n| {
                serde_json::json!({
                    "id": n.id,
                    "name": n.name,
                    "x": n.position_x,
                    "y": n.position_y,
                })
            }).collect::<Vec<_>>(),
            "variables": req.variables,
        });

        sqlx::query!(
            r"INSERT INTO workflows (id, name, description, definition, status, version, created_by, created_at, updated_at)
               VALUES ($1, $2, $3, $4, 'draft', 1, 'system', NOW(), NOW())",
            id.to_string(),
            &req.name,
            &req.description,
            &definition,
        )
            .execute(&self.pool)
            .await
            .map_err(|e| Status::internal(format!("Database error: {e}")))?;

        Ok(Response::new(CreateWorkflowResponse { id, name: req.name }))
    }

    async fn update_workflow(
        &self,
        request: Request<UpdateWorkflowRequest>,
    ) -> Result<Response<UpdateWorkflowResponse>, Status> {
        let req = request.into_inner();

        let definition = serde_json::json!({
            "nodes": req.nodes.iter().map(|n| {
                serde_json::json!({
                    "id": n.id,
                    "name": n.name,
                    "x": n.position_x,
                    "y": n.position_y,
                })
            }).collect::<Vec<_>>(),
            "variables": req.variables,
        });

        let result = sqlx::query!(
            r"UPDATE workflows
               SET name = COALESCE(NULLIF($2, ''), name),
                   description = COALESCE($3, description),
                   definition = COALESCE($4, definition),
                   updated_at = NOW()
               WHERE id = $1",
            req.id.to_string(),
            &req.name,
            &req.description,
            &definition,
        )
            .execute(&self.pool)
            .await
            .map_err(|e| Status::internal(format!("Database error: {e}")))?;

        if result.rows_affected() > 0 {
            Ok(Response::new(UpdateWorkflowResponse {
                id: req.id,
                name: req.name,
            }))
        } else {
            Err(Status::not_found("Workflow not found"))
        }
    }

    async fn delete_workflow(
        &self,
        request: Request<DeleteWorkflowRequest>,
    ) -> Result<Response<DeleteWorkflowResponse>, Status> {
        let req = request.into_inner();

        let result = sqlx::query!(
            "DELETE FROM workflows WHERE id = $1",
            req.id.to_string(),
        )
            .execute(&self.pool)
            .await
            .map_err(|e| Status::internal(format!("Database error: {e}")))?;

        Ok(Response::new(DeleteWorkflowResponse {
            success: result.rows_affected() > 0,
        }))
    }

    // ============== 工作流实例相关 ==============

    async fn start_workflow(
        &self,
        request: Request<StartWorkflowRequest>,
    ) -> Result<Response<StartWorkflowResponse>, Status> {
        let req = request.into_inner();

        // 获取工作流信息
        let workflow = sqlx::query_as!(
            WorkflowIdName,
            "SELECT id, name FROM workflows WHERE id = $1 AND status = 'active'",
            req.workflow_id.to_string(),
        )
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| Status::internal(format!("Database error: {e}")))?;

        let (workflow_id, workflow_name) = workflow
            .map(|w| (w.id, w.name))
            .ok_or_else(|| Status::not_found("Workflow not found or not active"))?;

        let instance_id = uuid::Uuid::new_v4().as_u64_pair().0 as i64;
        let now = chrono::Utc::now();

        // 将 HashMap 转换为 JSON 对象
        let variables = serde_json::to_value(&req.variables).unwrap_or_else(|_| serde_json::json!({}));

        sqlx::query!(
            r"INSERT INTO workflow_instances
               (id, workflow_id, workflow_version, status, variables, started_by, started_at)
               VALUES ($1, $2, 1, 'running', $3, $4, $5)",
            instance_id.to_string(),
            &workflow_id,
            &variables,
            "0", // started_by 字段从 business_key 获取
            now,
        )
            .execute(&self.pool)
            .await
            .map_err(|e| Status::internal(format!("Database error: {e}")))?;

        Ok(Response::new(StartWorkflowResponse {
            instance_id,
            workflow_name,
        }))
    }

    async fn get_instance(
        &self,
        request: Request<GetInstanceRequest>,
    ) -> Result<Response<GetInstanceResponse>, Status> {
        let req = request.into_inner();

        let row = sqlx::query!(
            r"SELECT id, workflow_id, workflow_version, status, current_node_id,
       variables, started_by, started_at, completed_at
       FROM workflow_instances WHERE id = $1",
            req.id.to_string(),
        )
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| Status::internal(format!("Database error: {e}")))?;

        match row {
            Some(row) => {
                let variables: HashMap<String, String> = row
                    .variables
                    .map(|v| serde_json::from_value(v).unwrap_or_default())
                    .unwrap_or_default();

                let instance = ProtoWorkflowInstance {
                    id: row.id.parse::<i64>().unwrap_or(0),
                    workflow_id: row.workflow_id.parse::<i64>().unwrap_or(0),
                    workflow_name: String::new(),
                    business_key: String::new(),
                    status: Self::instance_status_to_proto(&row.status) as i32,
                    variables,
                    current_node: row.current_node_id.unwrap_or_default(),
                    started_by: row.started_by.parse::<i64>().unwrap_or(0),
                    started_at: row.started_at.timestamp(),
                    finished_at: row.completed_at.map_or(0, |t| t.timestamp()),
                };

                Ok(Response::new(GetInstanceResponse {
                    instance: Some(instance),
                }))
            }
            None => Err(Status::not_found("Instance not found")),
        }
    }

    async fn list_instances(
        &self,
        request: Request<ListInstancesRequest>,
    ) -> Result<Response<ListInstancesResponse>, Status> {
        let req = request.into_inner();

        let page = if req.page > 0 { req.page } else { 1 };
        let page_size = if req.page_size > 0 { req.page_size } else { 20 };
        let offset = (page - 1) * page_size;

        // 将 InstanceStatus 枚举转换为字符串
        let status_str = match req.status() {
            ProtoInstanceStatus::InstanceRunning => "running",
            ProtoInstanceStatus::InstanceCompleted => "completed",
            ProtoInstanceStatus::InstanceCancelled => "cancelled",
            ProtoInstanceStatus::InstanceFailed => "failed",
        };

        let workflow_filter = (req.workflow_id != 0).then(|| req.workflow_id.to_string());

        let rows = sqlx::query!(
            r"SELECT id, workflow_id, workflow_version, status, current_node_id,
       variables, started_by, started_at, completed_at
       FROM workflow_instances
       WHERE ($1::text IS NULL OR workflow_id = $1)
         AND ($2::text IS NULL OR $2 = '' OR status = $2)
       ORDER BY started_at DESC
       LIMIT $3 OFFSET $4",
            workflow_filter.as_deref(),
            status_str,
            i64::from(page_size),
            i64::from(offset),
        )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Status::internal(format!("Database error: {e}")))?;

        let instances: Vec<ProtoWorkflowInstance> = rows
            .iter()
            .map(|row| {
                let variables: HashMap<String, String> = row
                    .variables
                    .clone()
                    .map(|v| serde_json::from_value(v).unwrap_or_default())
                    .unwrap_or_default();

                ProtoWorkflowInstance {
                    id: row.id.parse::<i64>().unwrap_or(0),
                    workflow_id: row.workflow_id.parse::<i64>().unwrap_or(0),
                    workflow_name: String::new(),
                    business_key: String::new(),
                    status: Self::instance_status_to_proto(&row.status) as i32,
                    variables,
                    current_node: row.current_node_id.clone().unwrap_or_default(),
                    started_by: row.started_by.parse::<i64>().unwrap_or(0),
                    started_at: row.started_at.timestamp(),
                    finished_at: row.completed_at.map_or(0, |t| t.timestamp()),
                }
            })
            .collect();

        let total: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM workflow_instances WHERE ($1::text IS NULL OR workflow_id = $1)",
            workflow_filter.as_deref(),
        )
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Status::internal(format!("Database error: {e}")))?
            .unwrap_or(0);

        Ok(Response::new(ListInstancesResponse { instances, total }))
    }

    // ============== 任务相关 ==============

    async fn list_tasks(
        &self,
        request: Request<ListTasksRequest>,
    ) -> Result<Response<ListTasksResponse>, Status> {
        let req = request.into_inner();

        let page = if req.page > 0 { req.page } else { 1 };
        let page_size = if req.page_size > 0 { req.page_size } else { 20 };
        let offset = (page - 1) * page_size;

        let instance_filter = (req.instance_id != 0).then(|| req.instance_id.to_string());

        let rows = sqlx::query!(
            r"SELECT id, instance_id, node_id, node_name, assignee, status,
       form_data, started_at, completed_at
       FROM task_records
       WHERE ($1::text IS NULL OR instance_id = $1)
         AND ($2 = '' OR assignee = $2)
       ORDER BY started_at DESC
       LIMIT $3 OFFSET $4",
            instance_filter.as_deref(),
            &req.assignee,
            i64::from(page_size),
            i64::from(offset),
        )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Status::internal(format!("Database error: {e}")))?;

        let tasks: Vec<WorkflowTask> = rows
            .iter()
            .map(|row| {
                WorkflowTask {
                    id: row.id.parse::<i64>().unwrap_or(0),
                    instance_id: row.instance_id.parse::<i64>().unwrap_or(0),
                    node_id: row.node_id.clone(),
                    name: row.node_name.clone(),
                    assignee: row.assignee.clone().unwrap_or_default(),
                    status: Self::task_status_to_proto(&row.status) as i32,
                    form_data: row
                        .form_data
                        .clone()
                        .map(|v| v.to_string())
                        .unwrap_or_default(),
                    created_at: row.started_at.map_or(0, |t| t.timestamp()),
                    completed_at: row.completed_at.map_or(0, |t| t.timestamp()),
                }
            })
            .collect();

        let total: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM task_records WHERE ($1::text IS NULL OR instance_id = $1)",
            instance_filter.as_deref(),
        )
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Status::internal(format!("Database error: {e}")))?
            .unwrap_or(0);

        Ok(Response::new(ListTasksResponse { tasks, total }))
    }

    async fn complete_task(
        &self,
        request: Request<CompleteTaskRequest>,
    ) -> Result<Response<CompleteTaskResponse>, Status> {
        let req = request.into_inner();

        // 将 action 转换为 status
        let new_status = match req.action.as_str() {
            "approve" | "complete" => "completed",
            "reject" => "rejected",
            _ => "completed",
        };

        let result = sqlx::query!(
            r"UPDATE task_records
               SET status = $2, completed_at = NOW()
               WHERE id = $1",
            req.task_id.to_string(),
            new_status,
        )
            .execute(&self.pool)
            .await
            .map_err(|e| Status::internal(format!("Database error: {e}")))?;

        Ok(Response::new(CompleteTaskResponse {
            success: result.rows_affected() > 0,
            next_node: String::new(),
        }))
    }

    async fn cancel_instance(
        &self,
        request: Request<CancelInstanceRequest>,
    ) -> Result<Response<CancelInstanceResponse>, Status> {
        let req = request.into_inner();

        let result = sqlx::query!(
            r"UPDATE workflow_instances
               SET status = 'cancelled', completed_at = NOW()
               WHERE id = $1",
            req.instance_id.to_string(),
        )
            .execute(&self.pool)
            .await
            .map_err(|e| Status::internal(format!("Database error: {e}")))?;

        Ok(Response::new(CancelInstanceResponse {
            success: result.rows_affected() > 0,
        }))
    }
}

impl common::service_bootstrap::GrpcServiceBuilder for WorkflowGrpcServer {
    fn build_grpc_server(&self, grpc_addr: &str) -> Result<tokio::task::JoinHandle<()>, Box<dyn std::error::Error + Send + Sync>> {
        use tonic::transport::Server;

        let addr: SocketAddr = grpc_addr.parse().map_err(|e| format!("invalid grpc addr: {e}"))?;
        let server = WorkflowServiceServer::new(WorkflowGrpcServer::new(&self.pool));
        let handle = tokio::spawn(async move {
            if let Err(e) = Server::builder()
                .add_service(server).serve(addr).await {
                tracing::error!("gRPC server error: {}", e);
            }
        });
        Ok(handle)
    }
}
