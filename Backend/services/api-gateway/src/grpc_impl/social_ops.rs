// ============== Social Ops 服务 gRPC 调用封装 ==============

pub type SocialOpsGrpcClient = GrpcClientWrapper<grpc_proto::socialops::account_service_client::AccountServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>>;

impl SocialOpsGrpcClient {
    pub async fn list_accounts(&mut self) -> Result<grpc_proto::socialops::ListAccountsResp, tonic::Status> {
        Ok(self.inner.list_accounts(grpc_proto::socialops::ListAccountsReq { user_id: String::new() }).await?.into_inner())
    }

    pub async fn get_account(&mut self, id: &str) -> Result<grpc_proto::socialops::Account, tonic::Status> {
        Ok(self.inner.get_account(grpc_proto::socialops::GetAccountReq { id: id.to_string() }).await?.into_inner())
    }
}

