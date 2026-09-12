//! Messaging Service Library
//!
//! 合并消息和通知功能

pub mod email; // Email provider module
pub mod grpc_handlers; // gRPC 服务处理器（辅助函数）
pub mod grpc_server; // gRPC 服务完整实现
pub mod preferences; // Notification preferences module
pub mod receipt;
pub mod repository; // 数据库仓储层 // 消息已读回执模块

// 导出状态类型（定义在 grpc_server.rs）
pub use grpc_server::{AppState, MessagingState};
pub use grpc_server::MessagingGrpcServer;

// 导出仓储类型
pub use repository::{
    Announcement, AnnouncementRepository, Message, MessageRepository, MessageTemplate, MessageUser,
    PostgresAnnouncementRepository, PostgresMessageRepository, TemplateRepository,
};

// 导出已读回执类型
pub use receipt::{
    BatchReceiptRequest, GroupReadReceipt, ReadReceipt, ReceiptConfig, ReceiptManager,
    ReceiptStats, ReceiptStatus,
};

// 导出邮件类型
pub use email::{
    EmailError, EmailMessage, EmailProvider, EmailResult, EmailTemplate, EmailTemplateEngine,
    MockEmailProvider, SmtpConfig, SmtpEmailProvider,
};

// 导出通知偏好类型
pub use preferences::{
    NotificationChannel, NotificationPreferences, NotificationType, PreferenceError,
    PreferenceResult, QuietHours, TypePreference, NotificationPreferencesRepository,
};
