// ============== API Key 服务 gRPC 调用封装 ==============

impl ApiKeyGrpcClient {
    pub async fn create_api_key(
        &mut self,
        name: String,
        description: String,
        permission_level: i32,
        allowed_ips: Vec<String>,
        rate_limit: i64,
        tenant_id: i64,
        user_id: i64,
        expires_at: i64,
    ) -> Result<grpc_proto::api_key::CreateApiKeyResponse, tonic::Status> {
        let request = grpc_proto::api_key::CreateApiKeyRequest {
            name, description, permission_level, allowed_ips,
            rate_limit, tenant_id, user_id, expires_at,
        };
        Ok(self.inner.create_api_key(request).await?.into_inner())
    }

    pub async fn validate_api_key(
        &mut self,
        key_id: String,
        secret_key: String,
        ip_address: String,
    ) -> Result<grpc_proto::api_key::ValidateApiKeyResponse, tonic::Status> {
        let request = grpc_proto::api_key::ValidateApiKeyRequest {
            key_id, secret_key, ip_address,
        };
        Ok(self.inner.validate_api_key(request).await?.into_inner())
    }

    pub async fn list_api_keys(
        &mut self,
        page: i32,
        page_size: i32,
        user_id: i64,
        tenant_id: i64,
        status: String,
    ) -> Result<grpc_proto::api_key::ListApiKeysResponse, tonic::Status> {
        let request = grpc_proto::api_key::ListApiKeysRequest {
            page, page_size, user_id, tenant_id, status,
        };
        Ok(self.inner.list_api_keys(request).await?.into_inner())
    }

    pub async fn delete_api_key(
        &mut self,
        id: i64,
    ) -> Result<grpc_proto::api_key::DeleteApiKeyResponse, tonic::Status> {
        let request = grpc_proto::api_key::DeleteApiKeyRequest { id };
        Ok(self.inner.delete_api_key(request).await?.into_inner())
    }

    pub async fn rotate_api_key(
        &mut self,
        id: i64,
    ) -> Result<grpc_proto::api_key::RotateApiKeyResponse, tonic::Status> {
        let request = grpc_proto::api_key::RotateApiKeyRequest { id };
        Ok(self.inner.rotate_api_key(request).await?.into_inner())
    }
}

