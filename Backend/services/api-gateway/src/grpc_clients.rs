#![allow(clippy::too_many_arguments)]

use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;

use grpc_core::{GrpcClient, ServiceDiscovery, ServiceInstance, connect_grpc, GrpcTokenInterceptor};
use grpc_proto::api_key::api_key_service_client::ApiKeyServiceClient;
use grpc_proto::audit::audit_service_client::AuditServiceClient;
use grpc_proto::ctp::ctp_service_client::CtpServiceClient;
use grpc_proto::auth::auth_service_client::AuthServiceClient;
use grpc_proto::cms::cms_service_client::CmsServiceClient;
use grpc_proto::feedback::feedback_service_client::FeedbackServiceClient;
use grpc_proto::file::file_service_client::FileServiceClient;
use grpc_proto::lpr::lpr_service_client::LprServiceClient;
use grpc_proto::message::message_service_client::MessageServiceClient;
use grpc_proto::tow::tow_service_client::TowServiceClient;
use grpc_proto::tenant::tenant_service_client::TenantServiceClient;
use grpc_proto::user::user_service_client::UserServiceClient;
use grpc_proto::workflow::workflow_service_client::WorkflowServiceClient;
use grpc_proto::socialops::account_service_client::AccountServiceClient;
use grpc_proto::hik::hik_service_client::HikServiceClient;
use grpc_proto::xlt::xlt_service_client::XltServiceClient;
use grpc_proto::ebike::ebike_service_client::EbikeServiceClient;
use grpc_proto::pay::pay_service_client::PayServiceClient;

use common::AppError;

pub use grpc_core::GrpcClientConfig;

/// 服务定义元信息
#[derive(Debug, Clone, Copy)]
pub struct ServiceDef {
    pub key: &'static str,
    pub name: &'static str,
    pub env_var: &'static str,
    pub default_url: &'static str,
}

/// 文章创建参数
#[derive(Debug, Clone)]
pub struct CreateArticleParams {
    pub title: String,
    pub content: String,
    pub summary: String,
    pub cover_image: String,
    pub category_id: i64,
    pub tags: Vec<String>,
    pub author: String,
    pub status: String,
}

/// 文章更新参数
#[derive(Debug, Clone)]
pub struct UpdateArticleParams {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub summary: String,
    pub cover_image: String,
    pub category_id: i64,
    pub tags: Vec<String>,
    pub status: String,
}

/// 所有 gRPC 上游服务的定义
pub const SERVICE_DEFS: &[ServiceDef] = &[
    ServiceDef { key: "auth", name: "auth-service", env_var: "AUTH_SERVICE_GRPC_URL", default_url: "http://auth-service:9091" },
    ServiceDef { key: "user", name: "user-service", env_var: "USER_SERVICE_GRPC_URL", default_url: "http://user-service:9090" },
    ServiceDef { key: "cms", name: "cms-service", env_var: "CMS_SERVICE_GRPC_URL", default_url: "http://cms-service:9082" },
    ServiceDef { key: "workflow", name: "workflow-service", env_var: "WORKFLOW_SERVICE_GRPC_URL", default_url: "http://workflow-service:9088" },
    ServiceDef { key: "audit", name: "audit-service", env_var: "AUDIT_SERVICE_GRPC_URL", default_url: "http://audit-service:9010" },
    ServiceDef { key: "tenant", name: "tenant-service", env_var: "TENANT_SERVICE_GRPC_URL", default_url: "http://tenant-service:9095" },
    ServiceDef { key: "feedback", name: "feedback-service", env_var: "FEEDBACK_SERVICE_GRPC_URL", default_url: "http://feedback-service:9085" },
    ServiceDef { key: "message", name: "message-service", env_var: "MESSAGE_SERVICE_GRPC_URL", default_url: "http://messaging-service:9083" },
    ServiceDef { key: "file", name: "file-service", env_var: "FILE_SERVICE_GRPC_URL", default_url: "http://file-service:9084" },
    ServiceDef { key: "api-key", name: "api-key-service", env_var: "API_KEY_SERVICE_GRPC_URL", default_url: "http://api-key-service:9094" },
    ServiceDef { key: "ctp", name: "ctp-service", env_var: "CTP_SERVICE_GRPC_URL", default_url: "http://ctp-service:9097" },
    ServiceDef { key: "lpr", name: "lpr-service", env_var: "LPR_SERVICE_GRPC_URL", default_url: "http://lpr-service:9099" },
    ServiceDef { key: "tow", name: "tow-service", env_var: "TOW_SERVICE_GRPC_URL", default_url: "http://tow-service:9086" },
    ServiceDef { key: "social-ops", name: "social-ops-service", env_var: "SOCIAL_OPS_SERVICE_GRPC_URL", default_url: "http://social-ops-service:9110" },
    ServiceDef { key: "hik", name: "hik-service", env_var: "HIK_SERVICE_GRPC_URL", default_url: "http://hik-service:9092" },
    ServiceDef { key: "xlt", name: "xlt-service", env_var: "XLT_SERVICE_GRPC_URL", default_url: "http://xlt-service:9096" },
    ServiceDef { key: "ebike", name: "ebike-service", env_var: "EBIKE_SERVICE_GRPC_URL", default_url: "http://ebike-service:9100" },
    ServiceDef { key: "pay", name: "pay-service", env_var: "PAY_SERVICE_GRPC_URL", default_url: "http://pay-service:9093" },
    ServiceDef { key: "clean", name: "clean-service", env_var: "CLEAN_SERVICE_GRPC_URL", default_url: "http://clean-service:9087" },
    ServiceDef { key: "browser", name: "browser-service", env_var: "BROWSER_SERVICE_GRPC_URL", default_url: "http://browser-service:8120" },
];

/// 从 gRPC URL 字符串中解析出主机和端口
#[must_use]
pub fn parse_grpc_url(url: &str) -> (&str, u16) {
    let without_scheme = url
        .trim_start_matches("http://")
        .trim_start_matches("https://");
    if let Some((host, port_str)) = without_scheme.split_once(':') {
        let port: u16 = port_str.parse().unwrap_or(9091);
        (host, port)
    } else {
        (without_scheme, 9091)
    }
}

/// 从环境变量注册所有服务到 `ServiceDiscovery`
pub fn register_services_to_discovery(discovery: &ServiceDiscovery) {
    for def in SERVICE_DEFS {
        let url = std::env::var(def.env_var).unwrap_or_else(|_| def.default_url.to_string());
        let (host, port) = parse_grpc_url(&url);
        let instance = ServiceInstance::new(def.name, host, port);
        discovery.register(instance);
    }
}

macro_rules! connect_env {
    ($clients:ident, $field:ident, $client:ty, $env_var:expr, $default_url:expr) => {
        $clients.$field = {
            let url = std::env::var($env_var).unwrap_or_else(|_| $default_url.to_string());
            connect_grpc(url, &$clients.config, |ch| <$client>::new(ch)).await.unwrap_or_else(|e| {
                tracing::warn!("连接 {} 失败: {}", stringify!($field), e);
                GrpcClient::new($default_url.to_string(), $clients.config.clone())
            })
        }
    };
}

macro_rules! connect_disc {
    ($clients:ident, $field:ident, $client:ty, $service_name:expr, $discovery:expr) => {
        if let Some(instance) = $discovery.get_instance($service_name) {
            let addr = instance.grpc_addr();
            $clients.$field = match connect_grpc(addr.clone(), &$clients.config, |ch| <$client>::new(ch)).await {
                Ok(c) => c,
                Err(e) => {
                    tracing::warn!("[服务发现] {} 连接失败: {}", $service_name, e);
                    GrpcClient::new(addr, $clients.config.clone())
                }
            };
        } else {
            tracing::warn!("[服务发现] {} 未注册", $service_name);
        }
    };
}

macro_rules! reconnect_svc {
    ($clients:ident, $field:ident, $client:ty, $service_name:expr, $addr:expr) => {
        if let Ok(c) = connect_grpc($addr.clone(), &$clients.config, |ch| <$client>::new(ch)).await {
            if let Some(inner) = c.get_inner().await {
                $clients.$field.set_inner(inner).await;
                tracing::info!("[服务发现] {} 重连成功 ({})", $service_name, $addr);
            }
        } else {
            tracing::warn!("[服务发现] {} 重连失败 ({})", $service_name, $addr);
        }
    };
}

macro_rules! is_available {
    ($clients:ident, $field:ident) => {
        $clients.$field.is_connected().await
    };
}

macro_rules! push_if_connected {
    ($services:ident, $clients:ident, $field:ident, $name:expr) => {
        if $clients.$field.is_connected().await {
            $services.push($name);
        }
    };
}

/// gRPC 客户端容器
#[derive(Clone)]
pub struct GrpcClients {
    pub config: GrpcClientConfig,
    pub auth_service: GrpcClient<AuthServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>>,
    pub user_service: GrpcClient<UserServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>>,
    pub cms_service: GrpcClient<CmsServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>>,
    pub workflow_service: GrpcClient<WorkflowServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>>,
    pub audit_service: GrpcClient<AuditServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>>,
    pub tenant_service: GrpcClient<TenantServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>>,
    pub feedback_service: GrpcClient<FeedbackServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>>,
    pub message_service: GrpcClient<MessageServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>>,
    pub file_service: GrpcClient<FileServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>>,
    pub api_key_service: GrpcClient<ApiKeyServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>>,
    pub ctp_service: GrpcClient<CtpServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>>,
    pub lpr_service: GrpcClient<LprServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>>,
    pub tow_service: GrpcClient<TowServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>>,
    pub social_ops_service: GrpcClient<AccountServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>>,
    pub hik_service: GrpcClient<HikServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>>,
    pub xlt_service: GrpcClient<XltServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>>,
    pub ebike_service: GrpcClient<EbikeServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>>,
    pub pay_service: GrpcClient<PayServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>>,
    pub clean_service: GrpcClient<grpc_proto::clean::clean_service_client::CleanServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>>,
    pub browser_service: GrpcClient<grpc_proto::browser::browser_service_client::BrowserServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>>,
}

macro_rules! disconnected_client {
    ($config:expr, $default_url:expr) => {
        GrpcClient::new($default_url.to_string(), $config.clone())
    };
}

impl GrpcClients {
    #[must_use]
    pub fn new(config: GrpcClientConfig) -> Self {
        let cfg = config.clone();
        Self {
            config,
            auth_service: disconnected_client!(cfg, "http://localhost:9091"),
            user_service: disconnected_client!(cfg, "http://localhost:9090"),
            cms_service: disconnected_client!(cfg, "http://localhost:9082"),
            workflow_service: disconnected_client!(cfg, "http://localhost:9088"),
            audit_service: disconnected_client!(cfg, "http://localhost:9010"),
            tenant_service: disconnected_client!(cfg, "http://localhost:9095"),
            feedback_service: disconnected_client!(cfg, "http://localhost:9085"),
            message_service: disconnected_client!(cfg, "http://localhost:9083"),
            file_service: disconnected_client!(cfg, "http://localhost:9084"),
            api_key_service: disconnected_client!(cfg, "http://localhost:9094"),
            ctp_service: disconnected_client!(cfg, "http://localhost:9097"),
            lpr_service: disconnected_client!(cfg, "http://localhost:9099"),
            tow_service: disconnected_client!(cfg, "http://localhost:9086"),
            social_ops_service: disconnected_client!(cfg, "http://localhost:9110"),
            hik_service: disconnected_client!(cfg, "http://localhost:9092"),
            xlt_service: disconnected_client!(cfg, "http://localhost:9096"),
            ebike_service: disconnected_client!(cfg, "http://localhost:9100"),
            pay_service: disconnected_client!(cfg, "http://localhost:9093"),
            clean_service: disconnected_client!(cfg, "http://localhost:9087"),
            browser_service: disconnected_client!(cfg, "http://localhost:8120"),
        }
    }

    pub async fn from_env(config: GrpcClientConfig) -> Self {
        let mut clients = Self::new(config);

        for def in SERVICE_DEFS {
            let url = std::env::var(def.env_var).unwrap_or_else(|_| def.default_url.to_string());
            match def.key {
                "auth" => connect_env!(clients, auth_service, AuthServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, "AUTH_SERVICE_GRPC_URL", url),
                "user" => connect_env!(clients, user_service, UserServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, "USER_SERVICE_GRPC_URL", url),
                "cms" => connect_env!(clients, cms_service, CmsServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, "CMS_SERVICE_GRPC_URL", url),
                "workflow" => connect_env!(clients, workflow_service, WorkflowServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, "WORKFLOW_SERVICE_GRPC_URL", url),
                "audit" => connect_env!(clients, audit_service, AuditServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, "AUDIT_SERVICE_GRPC_URL", url),
                "tenant" => connect_env!(clients, tenant_service, TenantServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, "TENANT_SERVICE_GRPC_URL", url),
                "feedback" => connect_env!(clients, feedback_service, FeedbackServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, "FEEDBACK_SERVICE_GRPC_URL", url),
                "message" => connect_env!(clients, message_service, MessageServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, "MESSAGE_SERVICE_GRPC_URL", url),
                "file" => connect_env!(clients, file_service, FileServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, "FILE_SERVICE_GRPC_URL", url),
                "api-key" => connect_env!(clients, api_key_service, ApiKeyServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, "API_KEY_SERVICE_GRPC_URL", url),
                "ctp" => connect_env!(clients, ctp_service, CtpServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, "CTP_SERVICE_GRPC_URL", url),
                "lpr" => connect_env!(clients, lpr_service, LprServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, "LPR_SERVICE_GRPC_URL", url),
                "tow" => connect_env!(clients, tow_service, TowServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, "TOW_SERVICE_GRPC_URL", url),
                "social-ops" => connect_env!(clients, social_ops_service, AccountServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, "SOCIAL_OPS_SERVICE_GRPC_URL", url),
                "hik" => connect_env!(clients, hik_service, HikServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, "HIK_SERVICE_GRPC_URL", url),
                "xlt" => connect_env!(clients, xlt_service, XltServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, "XLT_SERVICE_GRPC_URL", url),
                "ebike" => connect_env!(clients, ebike_service, EbikeServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, "EBIKE_SERVICE_GRPC_URL", url),
                "pay" => connect_env!(clients, pay_service, PayServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, "PAY_SERVICE_GRPC_URL", url),
                "clean" => connect_env!(clients, clean_service, grpc_proto::clean::clean_service_client::CleanServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, "CLEAN_SERVICE_GRPC_URL", url),
                "browser" => connect_env!(clients, browser_service, grpc_proto::browser::browser_service_client::BrowserServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, "BROWSER_SERVICE_GRPC_URL", url),
                _ => {}
            }
        }

        tracing::info!(
            "gRPC 客户端初始化完成: 可用服务={}",
            clients.available_services().await.len()
        );

        clients
    }

    pub async fn from_discovery(config: GrpcClientConfig, discovery: &ServiceDiscovery) -> Self {
        let mut clients = Self::new(config);

        connect_disc!(clients, auth_service, AuthServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, "auth-service", discovery);
        connect_disc!(clients, user_service, UserServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, "user-service", discovery);
        connect_disc!(clients, cms_service, CmsServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, "cms-service", discovery);
        connect_disc!(clients, workflow_service, WorkflowServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, "workflow-service", discovery);
        connect_disc!(clients, audit_service, AuditServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, "audit-service", discovery);
        connect_disc!(clients, tenant_service, TenantServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, "tenant-service", discovery);
        connect_disc!(clients, feedback_service, FeedbackServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, "feedback-service", discovery);
        connect_disc!(clients, message_service, MessageServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, "message-service", discovery);
        connect_disc!(clients, file_service, FileServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, "file-service", discovery);
        connect_disc!(clients, api_key_service, ApiKeyServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, "api-key-service", discovery);
        connect_disc!(clients, ctp_service, CtpServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, "ctp-service", discovery);
        connect_disc!(clients, lpr_service, LprServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, "lpr-service", discovery);
        connect_disc!(clients, tow_service, TowServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, "tow-service", discovery);
        connect_disc!(clients, hik_service, HikServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, "hik-service", discovery);
        connect_disc!(clients, xlt_service, XltServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, "xlt-service", discovery);
        connect_disc!(clients, ebike_service, EbikeServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, "ebike-service", discovery);
        connect_disc!(clients, pay_service, PayServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, "pay-service", discovery);
        connect_disc!(clients, clean_service, grpc_proto::clean::clean_service_client::CleanServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, "clean-service", discovery);

        tracing::info!(
            "[服务发现] gRPC 客户端初始化完成: 可用服务={}",
            clients.available_services().await.len()
        );

        clients
    }

    pub async fn auth_client(&self) -> Result<AuthGrpcClient, AppError> {
        let inner = self.auth_service.get_inner().await
            .ok_or_else(|| AppError::ServiceUnavailable("auth-service 不可用".into()))?;
        Ok(AuthGrpcClient::new(inner))
    }

    pub async fn user_client(&self) -> Result<UserGrpcClient, AppError> {
        let inner = self.user_service.get_inner().await
            .ok_or_else(|| AppError::ServiceUnavailable("user-service 不可用".into()))?;
        Ok(UserGrpcClient::new(inner))
    }

    pub async fn cms_client(&self) -> Result<CmsGrpcClient, AppError> {
        let inner = self.cms_service.get_inner().await
            .ok_or_else(|| AppError::ServiceUnavailable("cms-service 不可用".into()))?;
        Ok(CmsGrpcClient::new(inner))
    }

    pub async fn workflow_client(&self) -> Result<WorkflowGrpcClient, AppError> {
        let inner = self.workflow_service.get_inner().await
            .ok_or_else(|| AppError::ServiceUnavailable("workflow-service 不可用".into()))?;
        Ok(WorkflowGrpcClient::new(inner))
    }

    pub async fn audit_client(&self) -> Result<AuditGrpcClient, AppError> {
        let inner = self.audit_service.get_inner().await
            .ok_or_else(|| AppError::ServiceUnavailable("audit-service 不可用".into()))?;
        Ok(AuditGrpcClient::new(inner))
    }

    pub async fn tenant_client(&self) -> Result<TenantGrpcClient, AppError> {
        let inner = self.tenant_service.get_inner().await
            .ok_or_else(|| AppError::ServiceUnavailable("tenant-service 不可用".into()))?;
        Ok(TenantGrpcClient::new(inner))
    }

    pub async fn feedback_client(&self) -> Result<FeedbackGrpcClient, AppError> {
        let inner = self.feedback_service.get_inner().await
            .ok_or_else(|| AppError::ServiceUnavailable("feedback-service 不可用".into()))?;
        Ok(FeedbackGrpcClient::new(inner))
    }

    pub async fn message_client(&self) -> Result<MessageGrpcClient, AppError> {
        let inner = self.message_service.get_inner().await
            .ok_or_else(|| AppError::ServiceUnavailable("message-service 不可用".into()))?;
        Ok(MessageGrpcClient::new(inner))
    }

    pub async fn file_client(&self) -> Result<FileGrpcClient, AppError> {
        let inner = self.file_service.get_inner().await
            .ok_or_else(|| AppError::ServiceUnavailable("file-service 不可用".into()))?;
        Ok(FileGrpcClient::new(inner))
    }

    pub async fn api_key_client(&self) -> Result<ApiKeyGrpcClient, AppError> {
        let inner = self.api_key_service.get_inner().await
            .ok_or_else(|| AppError::ServiceUnavailable("api-key-service 不可用".into()))?;
        Ok(ApiKeyGrpcClient::new(inner))
    }

    pub async fn ctp_client(&self) -> Result<CtpGrpcClient, AppError> {
        let inner = self.ctp_service.get_inner().await
            .ok_or_else(|| AppError::ServiceUnavailable("ctp-service 不可用".into()))?;
        Ok(CtpGrpcClient::new(inner))
    }

    pub async fn lpr_client(&self) -> Result<LprGrpcClient, AppError> {
        let inner = self.lpr_service.get_inner().await
            .ok_or_else(|| AppError::ServiceUnavailable("lpr-service 不可用".into()))?;
        Ok(LprGrpcClient::new(inner))
    }

    pub async fn tow_client(&self) -> Result<TowGrpcClient, AppError> {
        let inner = self.tow_service.get_inner().await
            .ok_or_else(|| AppError::ServiceUnavailable("tow-service 不可用".into()))?;
        Ok(TowGrpcClient::new(inner))
    }

    pub async fn social_ops_client(&self) -> Result<SocialOpsGrpcClient, AppError> {
        let inner = self.social_ops_service.get_inner().await
            .ok_or_else(|| AppError::ServiceUnavailable("social-ops-service 不可用".into()))?;
        Ok(SocialOpsGrpcClient::new(inner))
    }

    pub async fn hik_client(&self) -> Result<HikGrpcClient, AppError> {
        let inner = self.hik_service.get_inner().await
            .ok_or_else(|| AppError::ServiceUnavailable("hik-service 不可用".into()))?;
        Ok(HikGrpcClient::new(inner))
    }

    pub async fn xlt_client(&self) -> Result<XltGrpcClient, AppError> {
        let inner = self.xlt_service.get_inner().await
            .ok_or_else(|| AppError::ServiceUnavailable("xlt-service 不可用".into()))?;
        Ok(XltGrpcClient::new(inner))
    }

    pub async fn ebike_client(&self) -> Result<EbikeGrpcClient, AppError> {
        let inner = self.ebike_service.get_inner().await
            .ok_or_else(|| AppError::ServiceUnavailable("ebike-service 不可用".into()))?;
        Ok(EbikeGrpcClient::new(inner))
    }

    pub async fn pay_client(&self) -> Result<PayGrpcClient, AppError> {
        let inner = self.pay_service.get_inner().await
            .ok_or_else(|| AppError::ServiceUnavailable("pay-service 不可用".into()))?;
        Ok(PayGrpcClient::new(inner))
    }

    pub async fn clean_client(&self) -> Result<CleanGrpcClient, AppError> {
        let inner = self.clean_service.get_inner().await
            .ok_or_else(|| AppError::ServiceUnavailable("clean-service 不可用".into()))?;
        Ok(CleanGrpcClient::new(inner))
    }

    pub async fn browser_client(&self) -> Result<BrowserGrpcClient, AppError> {
        let inner = self.browser_service.get_inner().await
            .ok_or_else(|| AppError::ServiceUnavailable("browser-service 不可用".into()))?;
        Ok(BrowserGrpcClient::new(inner))
    }

    pub async fn is_service_available(&self, service: &str) -> bool {
        match service {
            "auth" => is_available!(self, auth_service),
            "user" => is_available!(self, user_service),
            "cms" => is_available!(self, cms_service),
            "workflow" => is_available!(self, workflow_service),
            "audit" => is_available!(self, audit_service),
            "tenant" => is_available!(self, tenant_service),
            "feedback" => is_available!(self, feedback_service),
            "message" => is_available!(self, message_service),
            "file" => is_available!(self, file_service),
            "api-key" => is_available!(self, api_key_service),
            "ctp" => is_available!(self, ctp_service),
            "lpr" => is_available!(self, lpr_service),
            "tow" => is_available!(self, tow_service),
            "social-ops" => is_available!(self, social_ops_service),
            "hik" => is_available!(self, hik_service),
            "xlt" => is_available!(self, xlt_service),
            "ebike" => is_available!(self, ebike_service),
            "pay" => is_available!(self, pay_service),
            "clean" => is_available!(self, clean_service),
            _ => false,
        }
    }

    pub async fn available_services(&self) -> Vec<&'static str> {
        let mut services = Vec::new();
        push_if_connected!(services, self, auth_service, "auth");
        push_if_connected!(services, self, user_service, "user");
        push_if_connected!(services, self, cms_service, "cms");
        push_if_connected!(services, self, workflow_service, "workflow");
        push_if_connected!(services, self, audit_service, "audit");
        push_if_connected!(services, self, tenant_service, "tenant");
        push_if_connected!(services, self, feedback_service, "feedback");
        push_if_connected!(services, self, message_service, "message");
        push_if_connected!(services, self, file_service, "file");
        push_if_connected!(services, self, api_key_service, "api-key");
        push_if_connected!(services, self, ctp_service, "ctp");
        push_if_connected!(services, self, lpr_service, "lpr");
        push_if_connected!(services, self, tow_service, "tow");
        push_if_connected!(services, self, social_ops_service, "social-ops");
        push_if_connected!(services, self, hik_service, "hik");
        push_if_connected!(services, self, xlt_service, "xlt");
        push_if_connected!(services, self, ebike_service, "ebike");
        push_if_connected!(services, self, pay_service, "pay");
        push_if_connected!(services, self, clean_service, "clean");
        services
    }
}

macro_rules! reconnect_all {
    ($clients:ident, $discovery:ident) => {
        for def in crate::grpc_clients::SERVICE_DEFS {
            if !$clients.is_service_available(def.key).await {
                if let Some(instance) = $discovery.get_instance(def.name) {
                    let addr = instance.grpc_addr();
                    match def.key {
                        "auth" => reconnect_svc!($clients, auth_service, AuthServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, def.name, addr),
                        "user" => reconnect_svc!($clients, user_service, UserServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, def.name, addr),
                        "cms" => reconnect_svc!($clients, cms_service, CmsServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, def.name, addr),
                        "workflow" => reconnect_svc!($clients, workflow_service, WorkflowServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, def.name, addr),
                        "audit" => reconnect_svc!($clients, audit_service, AuditServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, def.name, addr),
                        "tenant" => reconnect_svc!($clients, tenant_service, TenantServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, def.name, addr),
                        "feedback" => reconnect_svc!($clients, feedback_service, FeedbackServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, def.name, addr),
                        "message" => reconnect_svc!($clients, message_service, MessageServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, def.name, addr),
                        "file" => reconnect_svc!($clients, file_service, FileServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, def.name, addr),
                        "api-key" => reconnect_svc!($clients, api_key_service, ApiKeyServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, def.name, addr),
                        "ctp" => reconnect_svc!($clients, ctp_service, CtpServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, def.name, addr),
                        "lpr" => reconnect_svc!($clients, lpr_service, LprServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, def.name, addr),
                        "tow" => reconnect_svc!($clients, tow_service, TowServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, def.name, addr),
                        "social-ops" => reconnect_svc!($clients, social_ops_service, AccountServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, def.name, addr),
                        "hik" => reconnect_svc!($clients, hik_service, HikServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, def.name, addr),
                        "xlt" => reconnect_svc!($clients, xlt_service, XltServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, def.name, addr),
                        "ebike" => reconnect_svc!($clients, ebike_service, EbikeServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, def.name, addr),
                        "pay" => reconnect_svc!($clients, pay_service, PayServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, def.name, addr),
                        "clean" => reconnect_svc!($clients, clean_service, grpc_proto::clean::clean_service_client::CleanServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, def.name, addr),
                        _ => {}
                    }
                }
            }
        }
    };
}

impl GrpcClients {
    pub async fn reconnect_all(&self, discovery: &ServiceDiscovery) {
        reconnect_all!(self, discovery);
    }

    pub async fn reconnect_service(
        &self,
        service_name: &str,
        addr: String,
    ) {
        match service_name {
            "auth-service" => reconnect_svc!(self, auth_service, AuthServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, service_name, addr),
            "user-service" => reconnect_svc!(self, user_service, UserServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, service_name, addr),
            "cms-service" => reconnect_svc!(self, cms_service, CmsServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, service_name, addr),
            "workflow-service" => reconnect_svc!(self, workflow_service, WorkflowServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, service_name, addr),
            "audit-service" => reconnect_svc!(self, audit_service, AuditServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, service_name, addr),
            "tenant-service" => reconnect_svc!(self, tenant_service, TenantServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, service_name, addr),
            "feedback-service" => reconnect_svc!(self, feedback_service, FeedbackServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, service_name, addr),
            "message-service" => reconnect_svc!(self, message_service, MessageServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, service_name, addr),
            "file-service" => reconnect_svc!(self, file_service, FileServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, service_name, addr),
            "api-key-service" => reconnect_svc!(self, api_key_service, ApiKeyServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, service_name, addr),
            "ctp-service" => reconnect_svc!(self, ctp_service, CtpServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, service_name, addr),
            "lpr-service" => reconnect_svc!(self, lpr_service, LprServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, service_name, addr),
            "tow-service" => reconnect_svc!(self, tow_service, TowServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, service_name, addr),
            "social-ops-service" => reconnect_svc!(self, social_ops_service, AccountServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, service_name, addr),
            "hik-service" => reconnect_svc!(self, hik_service, HikServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, service_name, addr),
            "xlt-service" => reconnect_svc!(self, xlt_service, XltServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, service_name, addr),
            "ebike-service" => reconnect_svc!(self, ebike_service, EbikeServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, service_name, addr),
            "pay-service" => reconnect_svc!(self, pay_service, PayServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>, service_name, addr),
            _ => tracing::warn!("未知服务: {service_name}"),
        }
    }
}

// ============== 通用 gRPC 客户端包装器 ==============

pub struct GrpcClientWrapper<T> {
    inner: T,
}

impl<T> GrpcClientWrapper<T> {
    pub const fn new(inner: T) -> Self {
        Self { inner }
    }

    pub const fn inner(&self) -> &T {
        &self.inner
    }

    pub const fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

// ============== 各服务类型别名 ==============

pub type AuthGrpcClient = GrpcClientWrapper<AuthServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>>;
pub type UserGrpcClient = GrpcClientWrapper<UserServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>>;
pub type CmsGrpcClient = GrpcClientWrapper<CmsServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>>;
pub type WorkflowGrpcClient = GrpcClientWrapper<WorkflowServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>>;
pub type AuditGrpcClient = GrpcClientWrapper<AuditServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>>;
pub type MessageGrpcClient = GrpcClientWrapper<MessageServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>>;
pub type ApiKeyGrpcClient = GrpcClientWrapper<ApiKeyServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>>;
pub type FileGrpcClient = GrpcClientWrapper<FileServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>>;
pub type TenantGrpcClient = GrpcClientWrapper<TenantServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>>;
pub type FeedbackGrpcClient = GrpcClientWrapper<FeedbackServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>>;
pub type CtpGrpcClient = GrpcClientWrapper<CtpServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>>;
pub type LprGrpcClient = GrpcClientWrapper<LprServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>>;
pub type HikGrpcClient = GrpcClientWrapper<HikServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>>;
pub type XltGrpcClient = GrpcClientWrapper<XltServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>>;
pub type EbikeGrpcClient = GrpcClientWrapper<EbikeServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>>;
pub type PayGrpcClient = GrpcClientWrapper<PayServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>>;
pub type CleanGrpcClient = GrpcClientWrapper<grpc_proto::clean::clean_service_client::CleanServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>>;
pub type BrowserGrpcClient = GrpcClientWrapper<grpc_proto::browser::browser_service_client::BrowserServiceClient<InterceptedService<Channel, GrpcTokenInterceptor>>>;

include!("grpc_impl/auth.rs");
include!("grpc_impl/user.rs");
include!("grpc_impl/cms.rs");
include!("grpc_impl/workflow.rs");
include!("grpc_impl/audit.rs");
include!("grpc_impl/message.rs");
include!("grpc_impl/feedback.rs");
include!("grpc_impl/tenant.rs");
include!("grpc_impl/file.rs");
include!("grpc_impl/api_key.rs");
include!("grpc_impl/lpr.rs");
include!("grpc_impl/tow.rs");
include!("grpc_impl/social_ops.rs");
include!("grpc_impl/hik.rs");
include!("grpc_impl/xlt.rs");
include!("grpc_impl/browser.rs");
include!("grpc_impl/ctp.rs");
include!("grpc_impl/ebike.rs");
include!("grpc_impl/pay.rs");
include!("grpc_impl/clean.rs");