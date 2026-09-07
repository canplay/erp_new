// ============== 审计服务 gRPC 调用封装 ==============

impl AuditGrpcClient {
    pub async fn create_login_log(
        &mut self,
        user_id: i64,
        username: String,
        ip_address: String,
        user_agent: String,
        login_location: String,
        login_status: i32,
        fail_reason: String,
        login_type: String,
    ) -> Result<grpc_proto::audit::CreateLoginLogResponse, tonic::Status> {
        let request = grpc_proto::audit::CreateLoginLogRequest {
            user_id,
            username,
            ip_address,
            user_agent,
            login_location,
            login_status,
            fail_reason,
            login_type,
        };
        Ok(self.inner.create_login_log(request).await?.into_inner())
    }

    pub async fn list_logs(
        &mut self,
        page: i32,
        page_size: i32,
        user_id: i64,
        keyword: String,
        resource_type: String,
        action: String,
        start_time: String,
        end_time: String,
    ) -> Result<grpc_proto::audit::ListLogsResponse, tonic::Status> {
        let action_enum = match action.as_str() {
            "create" => grpc_proto::audit::AuditAction::Create as i32,
            "update" => grpc_proto::audit::AuditAction::Update as i32,
            "delete" => grpc_proto::audit::AuditAction::Delete as i32,
            "login" => grpc_proto::audit::AuditAction::Login as i32,
            "logout" => grpc_proto::audit::AuditAction::Logout as i32,
            "export" => grpc_proto::audit::AuditAction::Export as i32,
            "import" => grpc_proto::audit::AuditAction::Import as i32,
            _ => grpc_proto::audit::AuditAction::Other as i32,
        };
        let request = grpc_proto::audit::ListLogsRequest {
            page, page_size, user_id, keyword,
            action: action_enum,
            resource_type,
            resource_id: 0,
            start_time, end_time,
        };
        Ok(self.inner.list_logs(request).await?.into_inner())
    }

    pub async fn get_log(
        &mut self,
        id: i64,
    ) -> Result<grpc_proto::audit::GetLogResponse, tonic::Status> {
        let request = grpc_proto::audit::GetLogRequest { id };
        Ok(self.inner.get_log(request).await?.into_inner())
    }

    pub async fn log_action(
        &mut self,
        user_id: i64,
        username: String,
        action: i32,
        resource_type: String,
        resource_id: i64,
        description: String,
        old_data: std::collections::HashMap<String, String>,
        new_data: std::collections::HashMap<String, String>,
    ) -> Result<grpc_proto::audit::LogActionResponse, tonic::Status> {
        let action_enum = grpc_proto::audit::AuditAction::try_from(action)
            .unwrap_or(grpc_proto::audit::AuditAction::Other)
            .into();
        let request = grpc_proto::audit::LogActionRequest {
            user_id, username, action: action_enum,
            resource_type, resource_id, description, old_data, new_data,
        };
        Ok(self.inner.log_action(request).await?.into_inner())
    }

    pub async fn get_stats(
        &mut self,
        period: String,
    ) -> Result<grpc_proto::audit::GetStatsResponse, tonic::Status> {
        let request = grpc_proto::audit::GetStatsRequest { period };
        Ok(self.inner.get_stats(request).await?.into_inner())
    }

    pub async fn archive_logs(
        &mut self,
        start_time: String,
        end_time: String,
        archive_user_id: i64,
    ) -> Result<grpc_proto::audit::ArchiveLogsResponse, tonic::Status> {
        let request = grpc_proto::audit::ArchiveLogsRequest {
            start_time, end_time, archive_user_id,
        };
        Ok(self.inner.archive_logs(request).await?.into_inner())
    }

    pub async fn list_login_logs(
        &mut self,
        page: i32,
        page_size: i32,
        user_id: i64,
        keyword: String,
        status: i32,
        start_time: String,
        end_time: String,
    ) -> Result<grpc_proto::audit::ListLoginLogsResponse, tonic::Status> {
        let request = grpc_proto::audit::ListLoginLogsRequest {
            page, page_size, user_id, keyword, status, start_time, end_time,
        };
        Ok(self.inner.list_login_logs(request).await?.into_inner())
    }

    pub async fn get_login_log(
        &mut self,
        id: i64,
    ) -> Result<grpc_proto::audit::GetLoginLogResponse, tonic::Status> {
        let request = grpc_proto::audit::GetLoginLogRequest { id };
        Ok(self.inner.get_login_log(request).await?.into_inner())
    }

    pub async fn clear_login_logs(
        &mut self,
        before_date: String,
    ) -> Result<grpc_proto::audit::ClearLoginLogsResponse, tonic::Status> {
        let request = grpc_proto::audit::ClearLoginLogsRequest { before_date };
        Ok(self.inner.clear_login_logs(request).await?.into_inner())
    }

    pub async fn list_api_call_logs(
        &mut self,
        page: i32,
        page_size: i32,
        user_id: i64,
        method: String,
        path: String,
        status_code: i32,
        start_time: String,
        end_time: String,
    ) -> Result<grpc_proto::audit::ListApiCallLogsResponse, tonic::Status> {
        let request = grpc_proto::audit::ListApiCallLogsRequest {
            page, page_size, user_id, method, path, status_code, start_time, end_time,
        };
        Ok(self.inner.list_api_call_logs(request).await?.into_inner())
    }

    pub async fn get_api_call_log(
        &mut self,
        id: i64,
    ) -> Result<grpc_proto::audit::GetApiCallLogResponse, tonic::Status> {
        let request = grpc_proto::audit::GetApiCallLogRequest { id };
        Ok(self.inner.get_api_call_log(request).await?.into_inner())
    }

    pub async fn export_logs(
        &mut self,
        start_time: String,
        end_time: String,
        format: String,
        log_type: String,
    ) -> Result<grpc_proto::audit::ExportLogsResponse, tonic::Status> {
        let request = grpc_proto::audit::ExportLogsRequest { start_time, end_time, format, log_type };
        Ok(self.inner.export_logs(request).await?.into_inner())
    }
}

