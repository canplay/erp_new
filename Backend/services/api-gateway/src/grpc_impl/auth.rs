// ============== 认证服务 gRPC 调用封装 ==============

impl AuthGrpcClient {
    pub async fn login(
        &mut self,
        username: String,
        password: String,
    ) -> Result<grpc_proto::auth::LoginResponse, tonic::Status> {
        let request = grpc_proto::auth::LoginRequest { username, password };
        Ok(self.inner.login(request).await?.into_inner())
    }

    pub async fn register(
        &mut self,
        username: String,
        password: String,
        email: String,
        phone: String,
        nickname: String,
    ) -> Result<grpc_proto::auth::RegisterResponse, tonic::Status> {
        let request = grpc_proto::auth::RegisterRequest {
            username, password, email, phone, nickname,
        };
        Ok(self.inner.register(request).await?.into_inner())
    }

    pub async fn validate_token(
        &mut self,
        token: String,
    ) -> Result<grpc_proto::auth::TokenResponse, tonic::Status> {
        let request = grpc_proto::auth::TokenRequest { token };
        Ok(self.inner.validate_token(request).await?.into_inner())
    }

    pub async fn refresh_token(
        &mut self,
        refresh_token: String,
    ) -> Result<grpc_proto::auth::RefreshResponse, tonic::Status> {
        let request = grpc_proto::auth::RefreshRequest { refresh_token };
        Ok(self.inner.refresh_token(request).await?.into_inner())
    }
}

