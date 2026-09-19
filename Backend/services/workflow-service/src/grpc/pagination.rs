use crate::grpc::info::*;

#[derive(Debug, Clone)]
pub struct PaginatedWorkflowsInfo {
    pub workflows: Vec<WorkflowInfo>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone)]
pub struct PaginatedInstancesInfo {
    pub instances: Vec<WorkflowInstanceInfo>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone)]
pub struct PaginatedTasksInfo {
    pub tasks: Vec<TaskRecordInfo>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}
