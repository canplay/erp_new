use std::sync::Arc;
use tonic::Status;

use crate::grpc::info::*;
use crate::grpc::pagination::*;
use crate::grpc::service::WorkflowGrpcService;
use crate::repository::workflow_repository::WorkflowRepository;

/// 获取工作流详情
pub async fn get_workflow(
    state: Arc<WorkflowGrpcService>,
    id: String,
) -> Result<Option<WorkflowInfo>, Status> {
    tracing::info!("【gRPC】获取工作流: {id}");

    match state.state().repository.find_by_id(&id).await {
        Ok(Some(workflow)) => {
            tracing::info!("【gRPC】工作流已找到: {}", workflow.name);
            Ok(Some(workflow.into()))
        }
        Ok(None) => {
            tracing::warn!("【gRPC】工作流不存在: {id}");
            Ok(None)
        }
        Err(e) => {
            tracing::error!("【gRPC】获取工作流失败: {e}");
            Err(Status::internal(format!("获取工作流失败: {e}")))
        }
    }
}

/// 获取工作流列表
pub async fn list_workflows(
    state: Arc<WorkflowGrpcService>,
    page: i32,
    page_size: i32,
    status: Option<String>,
) -> Result<PaginatedWorkflowsInfo, Status> {
    tracing::info!(
        "【gRPC】获取工作流列表: page={page}, page_size={page_size}, status={status:?}"
    );

    let page = page.max(1);
    let page_size = page_size.clamp(1, 100);

    match state
        .state()
        .repository
        .list(status.as_deref(), i64::from(page), i64::from(page_size))
        .await
    {
        Ok((workflows, total)) => {
            let workflow_infos: Vec<WorkflowInfo> =
                workflows.into_iter().map(std::convert::Into::into).collect();
            tracing::info!("【gRPC】获取到 {} 个工作流", workflow_infos.len());
            Ok(PaginatedWorkflowsInfo {
                workflows: workflow_infos,
                total,
                page: i64::from(page),
                page_size: i64::from(page_size),
            })
        }
        Err(e) => {
            tracing::error!("【gRPC】获取工作流列表失败: {e}");
            Err(Status::internal(format!("获取工作流列表失败: {e}")))
        }
    }
}

/// 创建工作流
pub async fn create_workflow(
    state: Arc<WorkflowGrpcService>,
    name: String,
    description: Option<String>,
    created_by: String,
) -> Result<String, Status> {
    tracing::info!(
        "【gRPC】创建工作流: name={name}, created_by={created_by}"
    );

    let workflow = crate::repository::Workflow::new(name, description, created_by);
    let _workflow_id = workflow.id.clone();

    match state.state().repository.create(&workflow).await {
        Ok(()) => {
            tracing::info!("【gRPC】工作流创建成功: {}", workflow.id);
            Ok(workflow.id.clone())
        }
        Err(e) => {
            tracing::error!("【gRPC】创建工作流失败: {e}");
            Err(Status::internal(format!("创建工作流失败: {e}")))
        }
    }
}

/// 更新工作流
pub async fn update_workflow(
    state: Arc<WorkflowGrpcService>,
    id: String,
    name: Option<String>,
    description: Option<String>,
    definition: Option<String>,
    status: Option<String>,
) -> Result<bool, Status> {
    tracing::info!("【gRPC】更新工作流: id={id}");

    match state.state().repository.find_by_id(&id).await {
        Ok(Some(mut workflow)) => {
            if let Some(n) = name {
                workflow.name = n;
            }
            if let Some(d) = description {
                workflow.description = Some(d);
            }
            if let Some(def) = definition
                && let Ok(parsed) = serde_json::from_str(&def) {
                    workflow.definition = parsed;
                }
            if let Some(s) = status {
                workflow.status = s;
            }
            workflow.updated_at = chrono::Utc::now();

            match state.state().repository.update(&workflow).await {
                Ok(()) => {
                    tracing::info!("【gRPC】工作流更新成功: {id}");
                    Ok(true)
                }
                Err(e) => {
                    tracing::error!("【gRPC】更新工作流失败: {e}");
                    Err(Status::internal(format!("更新工作流失败: {e}")))
                }
            }
        }
        Ok(None) => {
            tracing::warn!("【gRPC】工作流不存在: {id}");
            Err(Status::not_found(format!("工作流不存在: {id}")))
        }
        Err(e) => {
            tracing::error!("【gRPC】获取工作流失败: {e}");
            Err(Status::internal(format!("获取工作流失败: {e}")))
        }
    }
}

/// 发布工作流
pub async fn publish_workflow(state: Arc<WorkflowGrpcService>, id: String) -> Result<bool, Status> {
    tracing::info!("【gRPC】发布工作流: {id}");

    match state.state().repository.find_by_id(&id).await {
        Ok(Some(mut workflow)) => {
            workflow.status = "published".to_string();
            workflow.updated_at = chrono::Utc::now();

            match state.state().repository.update(&workflow).await {
                Ok(()) => {
                    tracing::info!("【gRPC】工作流发布成功: {id}");
                    Ok(true)
                }
                Err(e) => {
                    tracing::error!("【gRPC】发布工作流失败: {e}");
                    Err(Status::internal(format!("发布工作流失败: {e}")))
                }
            }
        }
        Ok(None) => {
            tracing::warn!("【gRPC】工作流不存在: {id}");
            Err(Status::not_found(format!("工作流不存在: {id}")))
        }
        Err(e) => {
            tracing::error!("【gRPC】获取工作流失败: {e}");
            Err(Status::internal(format!("获取工作流失败: {e}")))
        }
    }
}

/// 删除工作流
pub async fn delete_workflow(state: Arc<WorkflowGrpcService>, id: String) -> Result<bool, Status> {
    tracing::info!("【gRPC】删除工作流: {id}");

    match state.state().repository.delete(&id).await {
        Ok(()) => {
            tracing::info!("【gRPC】工作流删除成功: {id}");
            Ok(true)
        }
        Err(e) => {
            tracing::error!("【gRPC】删除工作流失败: {e}");
            Err(Status::internal(format!("删除工作流失败: {e}")))
        }
    }
}

// ============== 工作流实例管理接口 ==============

/// 启动工作流实例
pub async fn start_workflow_instance(
    state: Arc<WorkflowGrpcService>,
    workflow_id: String,
    started_by: String,
    variables: Option<String>,
) -> Result<String, Status> {
    tracing::info!(
        "【gRPC】启动工作流实例: workflow_id={workflow_id}, started_by={started_by}"
    );

    match state.state().repository.find_by_id(&workflow_id).await {
        Ok(Some(workflow)) => {
            if workflow.status != "published" {
                return Err(Status::failed_precondition("工作流未发布"));
            }

            let instance = crate::repository::WorkflowInstance {
                id: uuid::Uuid::new_v4().to_string(),
                workflow_id: workflow_id.clone(),
                workflow_version: workflow.version,
                status: "running".to_string(),
                current_node_id: None,
                variables: variables
                    .and_then(|v| serde_json::from_str(&v).ok())
                    .unwrap_or(serde_json::json!({})),
                started_by,
                started_at: chrono::Utc::now(),
                completed_at: None,
            };

            let instance_id = instance.id.clone();

            match state.state().repository.create_instance(&instance).await {
                Ok(()) => {
                    tracing::info!("【gRPC】实例启动成功: {instance_id}");
                    Ok(instance_id)
                }
                Err(e) => {
                    tracing::error!("【gRPC】启动实例失败: {e}");
                    Err(Status::internal(format!("启动实例失败: {e}")))
                }
            }
        }
        Ok(None) => {
            tracing::warn!("【gRPC】工作流不存在: {workflow_id}");
            Err(Status::not_found(format!("工作流不存在: {workflow_id}")))
        }
        Err(e) => {
            tracing::error!("【gRPC】获取工作流失败: {e}");
            Err(Status::internal(format!("获取工作流失败: {e}")))
        }
    }
}

/// 获取实例列表
pub async fn list_instances(
    state: Arc<WorkflowGrpcService>,
    workflow_id: Option<String>,
    page: i32,
    page_size: i32,
    status: Option<String>,
) -> Result<PaginatedInstancesInfo, Status> {
    tracing::info!(
        "【gRPC】获取实例列表: workflow_id={workflow_id:?}, page={page}"
    );

    let page = page.max(1);
    let page_size = page_size.clamp(1, 100);

    match state
        .state()
        .repository
        .list_instances(
            workflow_id.as_deref(),
            status.as_deref(),
            i64::from(page),
            i64::from(page_size),
        )
        .await
    {
        Ok((instances, total)) => {
            let instance_infos: Vec<WorkflowInstanceInfo> =
                instances.into_iter().map(std::convert::Into::into).collect();
            tracing::info!("【gRPC】获取到 {} 个实例", instance_infos.len());
            Ok(PaginatedInstancesInfo {
                instances: instance_infos,
                total,
                page: i64::from(page),
                page_size: i64::from(page_size),
            })
        }
        Err(e) => {
            tracing::error!("【gRPC】获取实例列表失败: {e}");
            Err(Status::internal(format!("获取实例列表失败: {e}")))
        }
    }
}

/// 获取实例详情
pub async fn get_instance(
    state: Arc<WorkflowGrpcService>,
    id: String,
) -> Result<Option<WorkflowInstanceInfo>, Status> {
    tracing::info!("【gRPC】获取实例: {id}");

    match state.state().repository.get_instance(&id).await {
        Ok(Some(instance)) => {
            tracing::info!("【gRPC】实例已找到: {}", instance.id);
            Ok(Some(instance.into()))
        }
        Ok(None) => {
            tracing::warn!("【gRPC】实例不存在: {id}");
            Ok(None)
        }
        Err(e) => {
            tracing::error!("【gRPC】获取实例失败: {e}");
            Err(Status::internal(format!("获取实例失败: {e}")))
        }
    }
}

/// 执行实例动作
pub async fn execute_instance_action(
    state: Arc<WorkflowGrpcService>,
    instance_id: String,
    action: String,
    _user: String,
    _comment: Option<String>,
    variables: Option<String>,
) -> Result<bool, Status> {
    tracing::info!(
        "【gRPC】执行实例动作: instance_id={instance_id}, action={action}"
    );

    match state.state().repository.get_instance(&instance_id).await {
        Ok(Some(mut instance)) => {
            match action.as_str() {
                "complete" => {
                    instance.status = "completed".to_string();
                    instance.completed_at = Some(chrono::Utc::now());
                }
                "cancel" => {
                    instance.status = "cancelled".to_string();
                    instance.completed_at = Some(chrono::Utc::now());
                }
                "reject" => {
                    instance.status = "rejected".to_string();
                    instance.completed_at = Some(chrono::Utc::now());
                }
                _ => {
                    return Err(Status::invalid_argument(format!("未知的动作: {action}")));
                }
            }

            if let Some(vars) = variables
                && let Ok(parsed) = serde_json::from_str(&vars) {
                    instance.variables = parsed;
                }

            match state.state().repository.update_instance(&instance).await {
                Ok(()) => {
                    tracing::info!("【gRPC】实例动作执行成功: {instance_id}");
                    Ok(true)
                }
                Err(e) => {
                    tracing::error!("【gRPC】执行实例动作失败: {e}");
                    Err(Status::internal(format!("执行实例动作失败: {e}")))
                }
            }
        }
        Ok(None) => {
            tracing::warn!("【gRPC】实例不存在: {instance_id}");
            Err(Status::not_found(format!("实例不存在: {instance_id}")))
        }
        Err(e) => {
            tracing::error!("【gRPC】获取实例失败: {e}");
            Err(Status::internal(format!("获取实例失败: {e}")))
        }
    }
}

/// 取消实例
pub async fn cancel_instance(
    state: Arc<WorkflowGrpcService>,
    id: String,
    _user: String,
    _reason: Option<String>,
) -> Result<bool, Status> {
    tracing::info!("【gRPC】取消实例: id={id}");

    match state.state().repository.get_instance(&id).await {
        Ok(Some(mut instance)) => {
            if instance.status == "completed" || instance.status == "cancelled" {
                return Err(Status::failed_precondition("实例已结束，无法取消"));
            }

            instance.status = "cancelled".to_string();
            instance.completed_at = Some(chrono::Utc::now());

            match state.state().repository.update_instance(&instance).await {
                Ok(()) => {
                    tracing::info!("【gRPC】实例取消成功: {id}");
                    Ok(true)
                }
                Err(e) => {
                    tracing::error!("【gRPC】取消实例失败: {e}");
                    Err(Status::internal(format!("取消实例失败: {e}")))
                }
            }
        }
        Ok(None) => {
            tracing::warn!("【gRPC】实例不存在: {id}");
            Err(Status::not_found(format!("实例不存在: {id}")))
        }
        Err(e) => {
            tracing::error!("【gRPC】获取实例失败: {e}");
            Err(Status::internal(format!("获取实例失败: {e}")))
        }
    }
}

// ============== 任务管理接口 ==============

/// 获取任务列表
pub async fn list_tasks(
    state: Arc<WorkflowGrpcService>,
    instance_id: Option<String>,
    assignee: Option<String>,
    status: Option<String>,
    page: i32,
    page_size: i32,
) -> Result<PaginatedTasksInfo, Status> {
    tracing::info!(
        "【gRPC】获取任务列表: instance_id={instance_id:?}, assignee={assignee:?}"
    );

    let page = page.max(1);
    let page_size = page_size.clamp(1, 100);

    match state
        .state()
        .repository
        .list_tasks(
            instance_id.as_deref(),
            assignee.as_deref(),
            status.as_deref(),
            i64::from(page),
            i64::from(page_size),
        )
        .await
    {
        Ok((tasks, total)) => {
            let task_infos: Vec<TaskRecordInfo> = tasks.into_iter().map(std::convert::Into::into).collect();
            tracing::info!("【gRPC】获取到 {} 个任务", task_infos.len());
            Ok(PaginatedTasksInfo {
                tasks: task_infos,
                total,
                page: i64::from(page),
                page_size: i64::from(page_size),
            })
        }
        Err(e) => {
            tracing::error!("【gRPC】获取任务列表失败: {e}");
            Err(Status::internal(format!("获取任务列表失败: {e}")))
        }
    }
}

/// 获取待办任务
pub async fn list_todo_tasks(
    state: Arc<WorkflowGrpcService>,
    user_id: String,
    page: i32,
    page_size: i32,
) -> Result<PaginatedTasksInfo, Status> {
    tracing::info!("【gRPC】获取待办任务: user_id={user_id}");

    let page = page.max(1);
    let page_size = page_size.clamp(1, 100);

    match state
        .state()
        .repository
        .list_tasks_by_assignee(&user_id, i64::from(page), i64::from(page_size))
        .await
    {
        Ok((tasks, total)) => {
            let task_infos: Vec<TaskRecordInfo> = tasks.into_iter().map(std::convert::Into::into).collect();
            tracing::info!("【gRPC】获取到 {} 个待办任务", task_infos.len());
            Ok(PaginatedTasksInfo {
                tasks: task_infos,
                total,
                page: i64::from(page),
                page_size: i64::from(page_size),
            })
        }
        Err(e) => {
            tracing::error!("【gRPC】获取待办任务失败: {e}");
            Err(Status::internal(format!("获取待办任务失败: {e}")))
        }
    }
}

/// 完成任务
pub async fn complete_task(
    state: Arc<WorkflowGrpcService>,
    task_id: String,
    user: String,
    comment: Option<String>,
    _variables: Option<String>,
) -> Result<bool, Status> {
    tracing::info!("【gRPC】完成任务: task_id={task_id}, user={user}");

    match state.state().repository.get_task(&task_id).await {
        Ok(Some(mut task)) => {
            if task.assignee.as_ref() != Some(&user) {
                return Err(Status::permission_denied("任务未分配给该用户"));
            }

            if task.status == "completed" {
                return Err(Status::failed_precondition("任务已完成"));
            }

            task.status = "completed".to_string();
            task.completed_at = Some(chrono::Utc::now());
            if let Some(c) = comment {
                task.comment = Some(c);
            }

            match state.state().repository.update_task(&task).await {
                Ok(()) => {
                    tracing::info!("【gRPC】任务完成成功: {task_id}");
                    Ok(true)
                }
                Err(e) => {
                    tracing::error!("【gRPC】完成任务失败: {e}");
                    Err(Status::internal(format!("完成任务失败: {e}")))
                }
            }
        }
        Ok(None) => {
            tracing::warn!("【gRPC】任务不存在: {task_id}");
            Err(Status::not_found(format!("任务不存在: {task_id}")))
        }
        Err(e) => {
            tracing::error!("【gRPC】获取任务失败: {e}");
            Err(Status::internal(format!("获取任务失败: {e}")))
        }
    }
}

/// 转派任务
pub async fn reassign_task(
    state: Arc<WorkflowGrpcService>,
    task_id: String,
    from_user: String,
    to_user: String,
    reason: Option<String>,
) -> Result<bool, Status> {
    tracing::info!(
        "【gRPC】转派任务: task_id={task_id}, from={from_user}, to={to_user}"
    );

    match state.state().repository.get_task(&task_id).await {
        Ok(Some(mut task)) => {
            if task.assignee.as_ref() != Some(&from_user) {
                return Err(Status::permission_denied("任务未分配给指定用户"));
            }

            task.assignee = Some(to_user);
            if let Some(r) = reason {
                task.comment = Some(format!("转派原因: {r}"));
            }

            match state.state().repository.update_task(&task).await {
                Ok(()) => {
                    tracing::info!("【gRPC】任务转派成功: {task_id}");
                    Ok(true)
                }
                Err(e) => {
                    tracing::error!("【gRPC】转派任务失败: {e}");
                    Err(Status::internal(format!("转派任务失败: {e}")))
                }
            }
        }
        Ok(None) => {
            tracing::warn!("【gRPC】任务不存在: {task_id}");
            Err(Status::not_found(format!("任务不存在: {task_id}")))
        }
        Err(e) => {
            tracing::error!("【gRPC】获取任务失败: {e}");
            Err(Status::internal(format!("获取任务失败: {e}")))
        }
    }
}

/// 领取任务
pub async fn claim_task(
    state: Arc<WorkflowGrpcService>,
    task_id: String,
    user: String,
) -> Result<bool, Status> {
    tracing::info!("【gRPC】领取任务: task_id={task_id}, user={user}");

    match state.state().repository.get_task(&task_id).await {
        Ok(Some(mut task)) => {
            if task.assignee.is_some() {
                return Err(Status::failed_precondition("任务已被领取"));
            }

            task.assignee = Some(user);

            match state.state().repository.update_task(&task).await {
                Ok(()) => {
                    tracing::info!("【gRPC】任务领取成功: {task_id}");
                    Ok(true)
                }
                Err(e) => {
                    tracing::error!("【gRPC】领取任务失败: {e}");
                    Err(Status::internal(format!("领取任务失败: {e}")))
                }
            }
        }
        Ok(None) => {
            tracing::warn!("【gRPC】任务不存在: {task_id}");
            Err(Status::not_found(format!("任务不存在: {task_id}")))
        }
        Err(e) => {
            tracing::error!("【gRPC】获取任务失败: {e}");
            Err(Status::internal(format!("获取任务失败: {e}")))
        }
    }
}

/// 归还任务
pub async fn unclaim_task(state: Arc<WorkflowGrpcService>, task_id: String) -> Result<bool, Status> {
    tracing::info!("【gRPC】归还任务: task_id={task_id}");

    match state.state().repository.get_task(&task_id).await {
        Ok(Some(mut task)) => {
            if task.status == "completed" {
                return Err(Status::failed_precondition("任务已完成，无法归还"));
            }

            task.assignee = None;

            match state.state().repository.update_task(&task).await {
                Ok(()) => {
                    tracing::info!("【gRPC】任务归还成功: {task_id}");
                    Ok(true)
                }
                Err(e) => {
                    tracing::error!("【gRPC】归还任务失败: {e}");
                    Err(Status::internal(format!("归还任务失败: {e}")))
                }
            }
        }
        Ok(None) => {
            tracing::warn!("【gRPC】任务不存在: {task_id}");
            Err(Status::not_found(format!("任务不存在: {task_id}")))
        }
        Err(e) => {
            tracing::error!("【gRPC】获取任务失败: {e}");
            Err(Status::internal(format!("获取任务失败: {e}")))
        }
    }
}

/// 驳回任务
pub async fn reject_task(
    state: Arc<WorkflowGrpcService>,
    task_id: String,
    user: String,
    reason: Option<String>,
) -> Result<bool, Status> {
    tracing::info!("【gRPC】驳回任务: task_id={task_id}, user={user}");

    match state.state().repository.get_task(&task_id).await {
        Ok(Some(mut task)) => {
            if task.assignee.as_ref() != Some(&user) {
                return Err(Status::permission_denied("任务未分配给该用户"));
            }

            task.status = "rejected".to_string();
            task.completed_at = Some(chrono::Utc::now());
            if let Some(r) = reason {
                task.comment = Some(r);
            }

            match state.state().repository.update_task(&task).await {
                Ok(()) => {
                    tracing::info!("【gRPC】任务驳回成功: {task_id}");
                    Ok(true)
                }
                Err(e) => {
                    tracing::error!("【gRPC】驳回任务失败: {e}");
                    Err(Status::internal(format!("驳回任务失败: {e}")))
                }
            }
        }
        Ok(None) => {
            tracing::warn!("【gRPC】任务不存在: {task_id}");
            Err(Status::not_found(format!("任务不存在: {task_id}")))
        }
        Err(e) => {
            tracing::error!("【gRPC】获取任务失败: {e}");
            Err(Status::internal(format!("获取任务失败: {e}")))
        }
    }
}

/// 获取任务历史
pub async fn get_task_history(
    state: Arc<WorkflowGrpcService>,
    instance_id: String,
) -> Result<Vec<TaskRecordInfo>, Status> {
    tracing::info!("【gRPC】获取任务历史: instance_id={instance_id}");

    match state.state().repository.get_task_history(&instance_id).await {
        Ok(tasks) => {
            let task_infos: Vec<TaskRecordInfo> = tasks.into_iter().map(std::convert::Into::into).collect();
            tracing::info!("【gRPC】获取到 {} 条历史记录", task_infos.len());
            Ok(task_infos)
        }
        Err(e) => {
            tracing::error!("【gRPC】获取任务历史失败: {e}");
            Err(Status::internal(format!("获取任务历史失败: {e}")))
        }
    }
}
