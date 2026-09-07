// ============== 工作流服务 gRPC 调用封装 ==============

impl WorkflowGrpcClient {
    pub async fn list_workflows(
        &mut self,
        page: i32,
        page_size: i32,
        keyword: String,
        status: i32,
    ) -> Result<grpc_proto::workflow::ListWorkflowsResponse, tonic::Status> {
        let request = grpc_proto::workflow::ListWorkflowsRequest {
            page, page_size, keyword,
            status: grpc_proto::workflow::WorkflowStatus::try_from(status)
                .unwrap_or(grpc_proto::workflow::WorkflowStatus::WorkflowDraft)
                .into(),
        };
        Ok(self.inner.list_workflows(request).await?.into_inner())
    }

    pub async fn get_workflow(
        &mut self,
        id: i64,
    ) -> Result<grpc_proto::workflow::GetWorkflowResponse, tonic::Status> {
        let request = grpc_proto::workflow::GetWorkflowRequest { id };
        Ok(self.inner.get_workflow(request).await?.into_inner())
    }

    pub async fn list_instances(
        &mut self,
        workflow_id: i64,
        page: i32,
        page_size: i32,
        status: i32,
        business_key: String,
    ) -> Result<grpc_proto::workflow::ListInstancesResponse, tonic::Status> {
        let request = grpc_proto::workflow::ListInstancesRequest {
            workflow_id, page, page_size,
            status: grpc_proto::workflow::InstanceStatus::try_from(status)
                .unwrap_or(grpc_proto::workflow::InstanceStatus::InstanceRunning)
                .into(),
            business_key,
        };
        Ok(self.inner.list_instances(request).await?.into_inner())
    }

    pub async fn list_tasks(
        &mut self,
        instance_id: i64,
        assignee: String,
        status: i32,
        page: i32,
        page_size: i32,
    ) -> Result<grpc_proto::workflow::ListTasksResponse, tonic::Status> {
        let request = grpc_proto::workflow::ListTasksRequest {
            instance_id, assignee,
            status: grpc_proto::workflow::TaskStatus::try_from(status)
                .unwrap_or(grpc_proto::workflow::TaskStatus::TaskPending)
                .into(),
            page, page_size,
        };
        Ok(self.inner.list_tasks(request).await?.into_inner())
    }

    pub async fn start_workflow(
        &mut self,
        workflow_id: i64,
        business_key: String,
        variables: std::collections::HashMap<String, String>,
    ) -> Result<grpc_proto::workflow::StartWorkflowResponse, tonic::Status> {
        let request = grpc_proto::workflow::StartWorkflowRequest {
            workflow_id, business_key, variables,
        };
        Ok(self.inner.start_workflow(request).await?.into_inner())
    }

    pub async fn complete_task(
        &mut self,
        task_id: i64,
        action: String,
        variables: std::collections::HashMap<String, String>,
        comment: String,
    ) -> Result<grpc_proto::workflow::CompleteTaskResponse, tonic::Status> {
        let request = grpc_proto::workflow::CompleteTaskRequest {
            task_id, action, variables, comment,
        };
        Ok(self.inner.complete_task(request).await?.into_inner())
    }

    pub async fn create_workflow(
        &mut self,
        name: String,
        description: String,
        nodes: Vec<grpc_proto::workflow::WorkflowNode>,
        variables: std::collections::HashMap<String, String>,
    ) -> Result<grpc_proto::workflow::CreateWorkflowResponse, tonic::Status> {
        let request = grpc_proto::workflow::CreateWorkflowRequest {
            name, description, nodes, variables,
        };
        Ok(self.inner.create_workflow(request).await?.into_inner())
    }

    pub async fn update_workflow(
        &mut self,
        id: i64,
        name: String,
        description: String,
        nodes: Vec<grpc_proto::workflow::WorkflowNode>,
        variables: std::collections::HashMap<String, String>,
        status: i32,
    ) -> Result<grpc_proto::workflow::UpdateWorkflowResponse, tonic::Status> {
        let request = grpc_proto::workflow::UpdateWorkflowRequest {
            id, name, description, nodes, variables,
            status: grpc_proto::workflow::WorkflowStatus::try_from(status)
                .unwrap_or(grpc_proto::workflow::WorkflowStatus::WorkflowDraft)
                .into(),
        };
        Ok(self.inner.update_workflow(request).await?.into_inner())
    }

    pub async fn delete_workflow(
        &mut self,
        id: i64,
    ) -> Result<grpc_proto::workflow::DeleteWorkflowResponse, tonic::Status> {
        let request = grpc_proto::workflow::DeleteWorkflowRequest { id };
        Ok(self.inner.delete_workflow(request).await?.into_inner())
    }

    pub async fn get_instance(
        &mut self,
        id: i64,
    ) -> Result<grpc_proto::workflow::GetInstanceResponse, tonic::Status> {
        let request = grpc_proto::workflow::GetInstanceRequest { id };
        Ok(self.inner.get_instance(request).await?.into_inner())
    }

    pub async fn cancel_instance(
        &mut self,
        instance_id: i64,
        reason: String,
    ) -> Result<grpc_proto::workflow::CancelInstanceResponse, tonic::Status> {
        let request = grpc_proto::workflow::CancelInstanceRequest { instance_id, reason };
        Ok(self.inner.cancel_instance(request).await?.into_inner())
    }
}

