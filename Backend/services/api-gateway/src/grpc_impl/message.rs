// ============== 消息服务 gRPC 调用封装 ==============

impl MessageGrpcClient {
    pub async fn list_messages(
        &mut self,
        user_id: i64,
        page: i32,
        page_size: i32,
        r#type: String,
        unread_only: bool,
    ) -> Result<grpc_proto::message::ListMessagesResponse, tonic::Status> {
        let request = grpc_proto::message::ListMessagesRequest {
            user_id, page, page_size, r#type, unread_only,
        };
        Ok(self.inner.list_messages(request).await?.into_inner())
    }

    pub async fn get_message(
        &mut self,
        id: i64,
        user_id: i64,
    ) -> Result<grpc_proto::message::GetMessageResponse, tonic::Status> {
        let request = grpc_proto::message::GetMessageRequest { id, user_id };
        Ok(self.inner.get_message(request).await?.into_inner())
    }

    pub async fn send_message(
        &mut self,
        user_id: i64,
        sender_id: i64,
        r#type: String,
        title: String,
        content: String,
        data: Option<grpc_proto::message::MessageData>,
        user_ids: Vec<i64>,
    ) -> Result<grpc_proto::message::SendMessageResponse, tonic::Status> {
        let request = grpc_proto::message::SendMessageRequest {
            user_id, sender_id, r#type, title, content, data, user_ids,
        };
        Ok(self.inner.send_message(request).await?.into_inner())
    }

    pub async fn get_unread_count(
        &mut self,
        user_id: i64,
        r#type: String,
    ) -> Result<grpc_proto::message::GetUnreadCountResponse, tonic::Status> {
        let request = grpc_proto::message::GetUnreadCountRequest { user_id, r#type };
        Ok(self.inner.get_unread_count(request).await?.into_inner())
    }

    pub async fn mark_as_read(
        &mut self,
        user_id: i64,
        message_ids: Vec<i64>,
    ) -> Result<grpc_proto::message::MarkAsReadResponse, tonic::Status> {
        let request = grpc_proto::message::MarkAsReadRequest { user_id, message_ids };
        Ok(self.inner.mark_as_read(request).await?.into_inner())
    }

    pub async fn delete_message(
        &mut self,
        user_id: i64,
        message_ids: Vec<i64>,
    ) -> Result<grpc_proto::message::DeleteMessageResponse, tonic::Status> {
        let request = grpc_proto::message::DeleteMessageRequest { user_id, message_ids };
        Ok(self.inner.delete_message(request).await?.into_inner())
    }

    pub async fn list_templates(
        &mut self,
        page: i32,
        page_size: i32,
        r#type: String,
    ) -> Result<grpc_proto::message::ListTemplatesResponse, tonic::Status> {
        let request = grpc_proto::message::ListTemplatesRequest { page, page_size, r#type };
        Ok(self.inner.list_templates(request).await?.into_inner())
    }

    pub async fn create_template(
        &mut self,
        name: String,
        r#type: String,
        title_template: String,
        content_template: String,
        variables: std::collections::HashMap<String, String>,
        is_active: bool,
    ) -> Result<grpc_proto::message::CreateTemplateResponse, tonic::Status> {
        let request = grpc_proto::message::CreateTemplateRequest {
            name, r#type, title_template, content_template, variables, is_active,
        };
        Ok(self.inner.create_template(request).await?.into_inner())
    }

    pub async fn update_template(
        &mut self,
        id: i64,
        name: String,
        r#type: String,
        title_template: String,
        content_template: String,
        variables: std::collections::HashMap<String, String>,
        is_active: bool,
    ) -> Result<grpc_proto::message::UpdateTemplateResponse, tonic::Status> {
        let request = grpc_proto::message::UpdateTemplateRequest {
            id, name, r#type, title_template, content_template, variables, is_active,
        };
        Ok(self.inner.update_template(request).await?.into_inner())
    }

    pub async fn delete_template(
        &mut self,
        id: i64,
    ) -> Result<grpc_proto::message::DeleteTemplateResponse, tonic::Status> {
        let request = grpc_proto::message::DeleteTemplateRequest { id };
        Ok(self.inner.delete_template(request).await?.into_inner())
    }
}

