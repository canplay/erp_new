//! gRPC Proto Library
//!
//! 所有微服务的 protobuf 定义

pub mod auth {
    tonic::include_proto!("auth");
}

pub mod user {
    tonic::include_proto!("user");
}

pub mod cms {
    tonic::include_proto!("cms");
}

pub mod message {
    tonic::include_proto!("message");
}

pub mod file {
    tonic::include_proto!("file");
}

pub mod feedback {
    tonic::include_proto!("feedback");
}

pub mod tenant {
    tonic::include_proto!("tenant");
}

pub mod workflow {
    tonic::include_proto!("workflow");
}

pub mod audit {
    tonic::include_proto!("audit");
}

pub mod api_key {
    tonic::include_proto!("api_key");
}

pub mod clean {
    tonic::include_proto!("grpc_proto.clean");
}

pub mod browser {
    tonic::include_proto!("browser");
}

pub mod ctp {
    tonic::include_proto!("ctp");
}

pub mod lpr {
    tonic::include_proto!("lpr");
}

pub mod tow {
    tonic::include_proto!("tow");
}

pub mod socialops {
    tonic::include_proto!("socialops");
}

pub mod hik {
    tonic::include_proto!("hik");
}

pub mod xlt {
    tonic::include_proto!("xlt");
}

pub mod ebike {
    tonic::include_proto!("ebike");
}

pub mod pay {
    tonic::include_proto!("pay");
}

pub mod billing {
    tonic::include_proto!("erp.billing.v1");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_auth_modules_exist() {
        let _req = crate::auth::LoginRequest::default();
        let _resp = crate::auth::LoginResponse::default();
    }

    #[test]
    fn test_user_modules_exist() {
        let _req = crate::user::GetUserRequest::default();
        let _resp = crate::user::CreateUserResponse::default();
    }

    #[test]
    fn test_cms_modules_exist() {
        let _article = crate::cms::Article::default();
    }

    #[test]
    fn test_file_modules_exist() {
        let _upload = crate::file::UploadRequest::default();
        let _file_info = crate::file::FileInfo::default();
    }

    #[test]
    fn test_feedback_modules_exist() {
        let _feedback = crate::feedback::Feedback::default();
        let _reply = crate::feedback::Reply::default();
    }

    #[test]
    fn test_tenant_modules_exist() {
        let _tenant = crate::tenant::Tenant::default();
    }

    #[test]
    fn test_workflow_modules_exist() {
        let _def = crate::workflow::WorkflowDefinition::default();
        let _instance = crate::workflow::WorkflowInstance::default();
    }

    #[test]
    fn test_audit_modules_exist() {
        let _log = crate::audit::AuditLog::default();
    }

    #[test]
    fn test_api_key_modules_exist() {
        let _key = crate::api_key::ApiKeyInfo::default();
    }

    #[test]
    fn test_message_modules_exist() {
        let _msg = crate::message::Message::default();
    }
}
