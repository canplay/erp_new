// ============== Browser 服务 gRPC 调用封装 ==============

impl BrowserGrpcClient {
    pub async fn open_page(&mut self) -> Result<grpc_proto::browser::OpenPageResponse, tonic::Status> {
        Ok(self.inner().clone().open_page(tonic::Request::new(grpc_proto::browser::OpenPageRequest {})).await?.into_inner())
    }
    pub async fn navigate(&mut self, session_id: String, url: String) -> Result<grpc_proto::browser::NavigateResponse, tonic::Status> {
        Ok(self.inner().clone().navigate(tonic::Request::new(grpc_proto::browser::NavigateRequest { session_id, url })).await?.into_inner())
    }
    pub async fn get_text(&mut self, session_id: String) -> Result<grpc_proto::browser::GetTextResponse, tonic::Status> {
        Ok(self.inner().clone().get_text(tonic::Request::new(grpc_proto::browser::GetTextRequest { session_id })).await?.into_inner())
    }
    pub async fn get_html(&mut self, session_id: String) -> Result<grpc_proto::browser::GetHtmlResponse, tonic::Status> {
        Ok(self.inner().clone().get_html(tonic::Request::new(grpc_proto::browser::GetHtmlRequest { session_id })).await?.into_inner())
    }
    pub async fn get_title(&mut self, session_id: String) -> Result<grpc_proto::browser::GetTitleResponse, tonic::Status> {
        Ok(self.inner().clone().get_title(tonic::Request::new(grpc_proto::browser::GetTitleRequest { session_id })).await?.into_inner())
    }
    pub async fn click(&mut self, session_id: String, selector: String) -> Result<grpc_proto::browser::ClickResponse, tonic::Status> {
        Ok(self.inner().clone().click(tonic::Request::new(grpc_proto::browser::ClickRequest { session_id, selector })).await?.into_inner())
    }
    pub async fn input(&mut self, session_id: String, selector: String, text: String) -> Result<grpc_proto::browser::InputResponse, tonic::Status> {
        Ok(self.inner().clone().input(tonic::Request::new(grpc_proto::browser::InputRequest { session_id, selector, text })).await?.into_inner())
    }
    pub async fn screenshot(&mut self, session_id: String, full_page: bool) -> Result<grpc_proto::browser::ScreenshotResponse, tonic::Status> {
        Ok(self.inner().clone().screenshot(tonic::Request::new(grpc_proto::browser::ScreenshotRequest { session_id, full_page })).await?.into_inner())
    }
    pub async fn close_page(&mut self, session_id: String) -> Result<grpc_proto::browser::ClosePageResponse, tonic::Status> {
        Ok(self.inner().clone().close_page(tonic::Request::new(grpc_proto::browser::ClosePageRequest { session_id })).await?.into_inner())
    }
}

