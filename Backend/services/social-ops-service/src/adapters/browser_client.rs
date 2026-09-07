//! Browser Service gRPC 客户端
//!
//! 通过 gRPC 调用 browser-service 执行浏览器自动化操作。
//! 遵循内部微服务间通过 gRPC 通信的架构原则。

use grpc_proto::browser::browser_service_client::BrowserServiceClient;
use grpc_proto::browser::{
    OpenPageRequest, NavigateRequest, GetTextRequest, GetHtmlRequest,
    GetTitleRequest, ClickRequest, InputRequest, SetCookiesRequest,
    ScreenshotRequest, GetElementsRequest, ClosePageRequest,
    CookieItem as GrpcCookieItem,
};
use serde_json::Value;

/// browser-service gRPC 地址（默认 <http://localhost:9120>）
fn grpc_endpoint() -> String {
    std::env::var("BROWSER_SERVICE_GRPC").unwrap_or_else(|_| "http://localhost:9120".to_string())
}

/// Cookie 条目
#[derive(Debug, Clone)]
pub struct CookieItem {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: Option<String>,
}

/// `BrowserClient` — 对 browser-service 的 gRPC 客户端封装
#[derive(Debug, Clone)]
pub struct BrowserClient {
    endpoint: String,
}

impl Default for BrowserClient {
    fn default() -> Self {
        Self::new()
    }
}

impl BrowserClient {
    #[must_use]
    pub fn new() -> Self {
        Self {
            endpoint: grpc_endpoint(),
        }
    }

    async fn client(&self) -> Result<BrowserServiceClient<tonic::transport::Channel>, anyhow::Error> {
        let channel = tonic::transport::Channel::from_shared(self.endpoint.clone())?
            .connect()
            .await?;
        Ok(BrowserServiceClient::new(channel))
    }

    /// 创建浏览器标签页，返回 `session_id`
    pub async fn create_session(&self) -> anyhow::Result<String> {
        let mut client = self.client().await?;
        let resp = client.open_page(OpenPageRequest {}).await?;
        Ok(resp.into_inner().session_id)
    }

    /// 导航到 URL
    pub async fn navigate(&self, session_id: &str, url: &str) -> anyhow::Result<String> {
        let mut client = self.client().await?;
        let resp = client
            .navigate(NavigateRequest {
                session_id: session_id.to_string(),
                url: url.to_string(),
            })
            .await?;
        let inner = resp.into_inner();
        if inner.success {
            Ok(inner.title)
        } else {
            anyhow::bail!("导航超时: {url}")
        }
    }

    /// 获取页面文本内容
    pub async fn get_text(&self, session_id: &str) -> anyhow::Result<String> {
        let mut client = self.client().await?;
        let resp = client
            .get_text(GetTextRequest {
                session_id: session_id.to_string(),
            })
            .await?;
        Ok(resp.into_inner().text)
    }

    /// 获取页面 HTML
    pub async fn get_html(&self, session_id: &str) -> anyhow::Result<String> {
        let mut client = self.client().await?;
        let resp = client
            .get_html(GetHtmlRequest {
                session_id: session_id.to_string(),
            })
            .await?;
        Ok(resp.into_inner().html)
    }

    /// 获取页面标题
    pub async fn get_title(&self, session_id: &str) -> anyhow::Result<String> {
        let mut client = self.client().await?;
        let resp = client
            .get_title(GetTitleRequest {
                session_id: session_id.to_string(),
            })
            .await?;
        Ok(resp.into_inner().title)
    }

    /// 点击元素
    pub async fn click(&self, session_id: &str, selector: &str) -> anyhow::Result<()> {
        let mut client = self.client().await?;
        let resp = client
            .click(ClickRequest {
                session_id: session_id.to_string(),
                selector: selector.to_string(),
            })
            .await?;
        if resp.into_inner().success {
            Ok(())
        } else {
            anyhow::bail!("点击失败: {selector}")
        }
    }

    /// 在元素中输入文本
    pub async fn input(&self, session_id: &str, selector: &str, text: &str) -> anyhow::Result<()> {
        let mut client = self.client().await?;
        let resp = client
            .input(InputRequest {
                session_id: session_id.to_string(),
                selector: selector.to_string(),
                text: text.to_string(),
            })
            .await?;
        if resp.into_inner().success {
            Ok(())
        } else {
            anyhow::bail!("输入失败: selector={selector}")
        }
    }

    /// 注入 Cookie
    pub async fn inject_cookies(&self, session_id: &str, cookies: Vec<CookieItem>) -> anyhow::Result<()> {
        let mut client = self.client().await?;
        let grpc_cookies: Vec<GrpcCookieItem> = cookies
            .into_iter()
            .map(|c| GrpcCookieItem {
                name: c.name,
                value: c.value,
                domain: c.domain,
                path: c.path,
            })
            .collect();
        let resp = client
            .set_cookies(SetCookiesRequest {
                session_id: session_id.to_string(),
                cookies: grpc_cookies,
            })
            .await?;
        if resp.into_inner().success {
            Ok(())
        } else {
            anyhow::bail!("Cookie 注入失败")
        }
    }

    /// 截图（返回 base64 字符串）
    pub async fn screenshot(&self, session_id: &str, full_page: bool) -> anyhow::Result<String> {
        let mut client = self.client().await?;
        let resp = client
            .screenshot(ScreenshotRequest {
                session_id: session_id.to_string(),
                full_page,
            })
            .await?;
        Ok(resp.into_inner().image_base64)
    }

    /// 获取元素信息（返回 JSON Value 以兼容现有解析代码）
    pub async fn get_elements(&self, session_id: &str, selector: &str) -> anyhow::Result<Vec<Value>> {
        let mut client = self.client().await?;
        let resp = client
            .get_elements(GetElementsRequest {
                session_id: session_id.to_string(),
                selector: selector.to_string(),
            })
            .await?;
        let elements = resp.into_inner().elements;
        let mut result = Vec::new();
        for el in elements {
            result.push(serde_json::json!({
                "tag": el.tag,
                "text": el.text,
                "attributes": el.attributes,
            }));
        }
        Ok(result)
    }

    /// 关闭浏览器标签页
    pub async fn close_session(&self, session_id: &str) -> anyhow::Result<()> {
        let mut client = self.client().await?;
        let resp = client
            .close_page(ClosePageRequest {
                session_id: session_id.to_string(),
            })
            .await?;
        if resp.into_inner().success {
            Ok(())
        } else {
            anyhow::bail!("关闭会话失败")
        }
    }

    /// 等待指定秒数
    pub async fn wait(&self, _session_id: &str, seconds: u64) -> anyhow::Result<()> {
        tokio::time::sleep(std::time::Duration::from_secs(seconds)).await;
        Ok(())
    }
}
