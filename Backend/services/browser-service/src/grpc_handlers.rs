//! gRPC 处理函数 — `BrowserService` 实现

use std::sync::Arc;
use tonic::{Request, Response, Status};

use grpc_proto::browser::browser_service_server::BrowserService;
use grpc_proto::browser::{
    OpenPageRequest, OpenPageResponse,
    NavigateRequest, NavigateResponse,
    GetTextRequest, GetTextResponse,
    GetHtmlRequest, GetHtmlResponse,
    GetTitleRequest, GetTitleResponse,
    ClickRequest, ClickResponse,
    InputRequest, InputResponse,
    SetCookiesRequest, SetCookiesResponse,
    ScreenshotRequest, ScreenshotResponse,
    GetElementsRequest, GetElementsResponse,
    ClosePageRequest, ClosePageResponse,
    ElementInfo,
};
use uuid::Uuid;

use crate::browser_pool::BrowserPool;

/// gRPC 应用状态
#[derive(Clone)]
pub struct GrpcAppState {
    pub pool: Arc<BrowserPool>,
    pub sessions: Arc<tokio::sync::Mutex<std::collections::HashMap<String, crate::browser_pool::BrowserSession>>>,
}

impl GrpcAppState {
    pub fn new(pool: Arc<BrowserPool>) -> Self {
        Self {
            pool,
            sessions: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        }
    }
}

/// `BrowserService` gRPC 实现
#[tonic::async_trait]
impl BrowserService for GrpcAppState {
    async fn open_page(&self, _request: Request<OpenPageRequest>) -> Result<Response<OpenPageResponse>, Status> {
        let page_arc = self.pool.get_page().ok_or_else(|| Status::unavailable("浏览器池为空"))?;
        let session_id = Uuid::new_v4().to_string();
        let session = crate::browser_pool::BrowserSession::new(session_id.clone());

        let page = page_arc.lock().await;
        match page.new_tab(None).await {
            Ok(tab) => {
                *session.tab.lock().await = Some(tab);
            }
            Err(e) => return Err(Status::internal(format!("无法创建标签页: {e}"))),
        }

        self.sessions.lock().await.insert(session_id.clone(), session);
        Ok(Response::new(OpenPageResponse { session_id }))
    }

    async fn navigate(&self, request: Request<NavigateRequest>) -> Result<Response<NavigateResponse>, Status> {
        let req = request.into_inner();
        let sessions = self.sessions.lock().await;
        let session = sessions.get(&req.session_id).ok_or_else(|| Status::not_found("会话不存在"))?;
        let mut tab_guard = session.tab.lock().await;
        let tab = tab_guard.as_mut().ok_or_else(|| Status::failed_precondition("标签页未初始化"))?;

        match tab.get(&req.url).await {
            Ok(success) => {
                let title = tab.title().await.unwrap_or_default();
                Ok(Response::new(NavigateResponse { success, title }))
            }
            Err(e) => Err(Status::internal(format!("导航失败: {e}"))),
        }
    }

    async fn get_text(&self, request: Request<GetTextRequest>) -> Result<Response<GetTextResponse>, Status> {
        let req = request.into_inner();
        let sessions = self.sessions.lock().await;
        let session = sessions.get(&req.session_id).ok_or_else(|| Status::not_found("会话不存在"))?;
        let tab_guard = session.tab.lock().await;
        let tab = tab_guard.as_ref().ok_or_else(|| Status::failed_precondition("标签页未初始化"))?;

        let text = tab.ele_text("body").await.unwrap_or(None).unwrap_or_default();
        Ok(Response::new(GetTextResponse { text }))
    }

    async fn get_html(&self, request: Request<GetHtmlRequest>) -> Result<Response<GetHtmlResponse>, Status> {
        let req = request.into_inner();
        let sessions = self.sessions.lock().await;
        let session = sessions.get(&req.session_id).ok_or_else(|| Status::not_found("会话不存在"))?;
        let tab_guard = session.tab.lock().await;
        let tab = tab_guard.as_ref().ok_or_else(|| Status::failed_precondition("标签页未初始化"))?;

        let html = tab.html().await.unwrap_or_default();
        Ok(Response::new(GetHtmlResponse { html }))
    }

    async fn get_title(&self, request: Request<GetTitleRequest>) -> Result<Response<GetTitleResponse>, Status> {
        let req = request.into_inner();
        let sessions = self.sessions.lock().await;
        let session = sessions.get(&req.session_id).ok_or_else(|| Status::not_found("会话不存在"))?;
        let tab_guard = session.tab.lock().await;
        let tab = tab_guard.as_ref().ok_or_else(|| Status::failed_precondition("标签页未初始化"))?;

        let title = tab.title().await.unwrap_or_default();
        Ok(Response::new(GetTitleResponse { title }))
    }

    async fn click(&self, request: Request<ClickRequest>) -> Result<Response<ClickResponse>, Status> {
        let req = request.into_inner();
        let sessions = self.sessions.lock().await;
        let session = sessions.get(&req.session_id).ok_or_else(|| Status::not_found("会话不存在"))?;
        let tab_guard = session.tab.lock().await;
        let tab = tab_guard.as_ref().ok_or_else(|| Status::failed_precondition("标签页未初始化"))?;

        match tab.click(&req.selector).await {
            Ok(()) => Ok(Response::new(ClickResponse { success: true })),
            Err(e) => Err(Status::internal(format!("点击失败: {e}"))),
        }
    }

    async fn input(&self, request: Request<InputRequest>) -> Result<Response<InputResponse>, Status> {
        let req = request.into_inner();
        let sessions = self.sessions.lock().await;
        let session = sessions.get(&req.session_id).ok_or_else(|| Status::not_found("会话不存在"))?;
        let tab_guard = session.tab.lock().await;
        let tab = tab_guard.as_ref().ok_or_else(|| Status::failed_precondition("标签页未初始化"))?;

        match tab.input(&req.selector, &req.text).await {
            Ok(()) => Ok(Response::new(InputResponse { success: true })),
            Err(e) => Err(Status::internal(format!("输入失败: {e}"))),
        }
    }

    async fn set_cookies(&self, request: Request<SetCookiesRequest>) -> Result<Response<SetCookiesResponse>, Status> {
        let req = request.into_inner();
        let sessions = self.sessions.lock().await;
        let session = sessions.get(&req.session_id).ok_or_else(|| Status::not_found("会话不存在"))?;
        let tab_guard = session.tab.lock().await;
        let tab = tab_guard.as_ref().ok_or_else(|| Status::failed_precondition("标签页未初始化"))?;

        let cookie_params: Vec<drission::cdp::CookieParam> = req.cookies.iter().map(|c| {
            drission::cdp::CookieParam {
                name: c.name.clone(),
                value: c.value.clone(),
                url: None,
                domain: Some(c.domain.clone()),
                path: c.path.clone(),
                secure: None,
                http_only: None,
                expires: None,
            }
        }).collect();
        let count = cookie_params.len() as i32;

        match tab.set_cookies(cookie_params).await {
            Ok(()) => Ok(Response::new(SetCookiesResponse { success: true, count })),
            Err(e) => Err(Status::internal(format!("Cookie 注入失败: {e}"))),
        }
    }

    async fn screenshot(&self, request: Request<ScreenshotRequest>) -> Result<Response<ScreenshotResponse>, Status> {
        let req = request.into_inner();
        let sessions = self.sessions.lock().await;
        let session = sessions.get(&req.session_id).ok_or_else(|| Status::not_found("会话不存在"))?;
        let tab_guard = session.tab.lock().await;
        let tab = tab_guard.as_ref().ok_or_else(|| Status::failed_precondition("标签页未初始化"))?;

        match tab.screenshot_base64(req.full_page).await {
            Ok(image_base64) => Ok(Response::new(ScreenshotResponse { image_base64 })),
            Err(e) => Err(Status::internal(format!("截图失败: {e}"))),
        }
    }

    async fn get_elements(&self, request: Request<GetElementsRequest>) -> Result<Response<GetElementsResponse>, Status> {
        let req = request.into_inner();
        let sessions = self.sessions.lock().await;
        let session = sessions.get(&req.session_id).ok_or_else(|| Status::not_found("会话不存在"))?;
        let tab_guard = session.tab.lock().await;
        let tab = tab_guard.as_ref().ok_or_else(|| Status::failed_precondition("标签页未初始化"))?;

        match tab.eles(&req.selector).await {
            Ok(elements) => {
                let mut infos = Vec::new();
                for el in &elements {
                    let tag = el.tag().await.unwrap_or_default();
                    let text = el.text().await.unwrap_or_default();
                    let mut attributes = std::collections::HashMap::new();
                    if let Ok(Some(src)) = el.attr("src").await {
                        attributes.insert("src".to_string(), src);
                    }
                    if let Ok(Some(href)) = el.attr("href").await {
                        attributes.insert("href".to_string(), href);
                    }
                    if let Ok(Some(cls)) = el.attr("class").await {
                        attributes.insert("class".to_string(), cls);
                    }
                    infos.push(ElementInfo { tag, text, attributes });
                }
                let count = infos.len() as i32;
                Ok(Response::new(GetElementsResponse { elements: infos, count }))
            }
            Err(e) => Err(Status::internal(format!("查找元素失败: {e}"))),
        }
    }

    async fn close_page(&self, request: Request<ClosePageRequest>) -> Result<Response<ClosePageResponse>, Status> {
        let req = request.into_inner();
        let mut sessions = self.sessions.lock().await;
        if let Some(session) = sessions.remove(&req.session_id) {
            let mut tab_guard = session.tab.lock().await;
            if let Some(tab) = tab_guard.take() {
                let _ = tab.close().await;
            }
            Ok(Response::new(ClosePageResponse { success: true }))
        } else {
            Err(Status::not_found("会话不存在"))
        }
    }
}
