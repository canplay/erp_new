// ============== 反馈服务 gRPC 调用封装 ==============

impl FeedbackGrpcClient {
    pub async fn list_feedbacks(
        &mut self,
        page: i32,
        page_size: i32,
        user_id: i64,
        r#type: i32,
        status: i32,
        priority: i32,
        keyword: String,
    ) -> Result<grpc_proto::feedback::ListFeedbacksResponse, tonic::Status> {
        let request = grpc_proto::feedback::ListFeedbacksRequest {
            page, page_size, user_id,
            r#type: grpc_proto::feedback::FeedbackType::try_from(r#type)
                .unwrap_or(grpc_proto::feedback::FeedbackType::Other).into(),
            status: grpc_proto::feedback::FeedbackStatus::try_from(status)
                .unwrap_or(grpc_proto::feedback::FeedbackStatus::Open).into(),
            priority: grpc_proto::feedback::FeedbackPriority::try_from(priority)
                .unwrap_or(grpc_proto::feedback::FeedbackPriority::Medium).into(),
            keyword,
        };
        Ok(self.inner.list_feedbacks(request).await?.into_inner())
    }

    pub async fn get_feedback(
        &mut self,
        id: i64,
    ) -> Result<grpc_proto::feedback::GetFeedbackResponse, tonic::Status> {
        let request = grpc_proto::feedback::GetFeedbackRequest { id };
        Ok(self.inner.get_feedback(request).await?.into_inner())
    }

    pub async fn create_feedback(
        &mut self,
        user_id: i64,
        username: String,
        r#type: i32,
        title: String,
        content: String,
        attachments: Vec<String>,
        device_info: String,
        app_version: String,
        extra: std::collections::HashMap<String, String>,
    ) -> Result<grpc_proto::feedback::CreateFeedbackResponse, tonic::Status> {
        let request = grpc_proto::feedback::CreateFeedbackRequest {
            user_id, username,
            r#type: grpc_proto::feedback::FeedbackType::try_from(r#type)
                .unwrap_or(grpc_proto::feedback::FeedbackType::Other).into(),
            title, content, attachments, device_info, app_version, extra,
        };
        Ok(self.inner.create_feedback(request).await?.into_inner())
    }

    pub async fn update_feedback(
        &mut self,
        id: i64,
        status: i32,
        priority: i32,
        assigned_to: i64,
        reply_content: String,
    ) -> Result<grpc_proto::feedback::UpdateFeedbackResponse, tonic::Status> {
        let request = grpc_proto::feedback::UpdateFeedbackRequest {
            id,
            status: grpc_proto::feedback::FeedbackStatus::try_from(status)
                .unwrap_or(grpc_proto::feedback::FeedbackStatus::Open).into(),
            priority: grpc_proto::feedback::FeedbackPriority::try_from(priority)
                .unwrap_or(grpc_proto::feedback::FeedbackPriority::Medium).into(),
            assigned_to, reply_content,
        };
        Ok(self.inner.update_feedback(request).await?.into_inner())
    }

    pub async fn delete_feedback(
        &mut self,
        id: i64,
    ) -> Result<grpc_proto::feedback::DeleteFeedbackResponse, tonic::Status> {
        let request = grpc_proto::feedback::DeleteFeedbackRequest { id };
        Ok(self.inner.delete_feedback(request).await?.into_inner())
    }

    pub async fn add_reply(
        &mut self,
        feedback_id: i64,
        user_id: i64,
        username: String,
        content: String,
        is_admin: bool,
    ) -> Result<grpc_proto::feedback::AddReplyResponse, tonic::Status> {
        let request = grpc_proto::feedback::AddReplyRequest {
            feedback_id, user_id, username, content, is_admin,
        };
        Ok(self.inner.add_reply(request).await?.into_inner())
    }

    pub async fn list_replies(
        &mut self,
        feedback_id: i64,
    ) -> Result<grpc_proto::feedback::ListRepliesResponse, tonic::Status> {
        let request = grpc_proto::feedback::ListRepliesRequest { feedback_id };
        Ok(self.inner.list_replies(request).await?.into_inner())
    }

    pub async fn get_feedback_stats(
        &mut self,
        period_start: i64,
        period_end: i64,
    ) -> Result<grpc_proto::feedback::GetStatsResponse, tonic::Status> {
        let request = grpc_proto::feedback::GetStatsRequest { period_start, period_end };
        Ok(self.inner.get_stats(request).await?.into_inner())
    }
}

