//! Tow Service Library
//!
//! 提供拖车服务的 gRPC 实现
//! 包含车辆管理、配置选项等功能
//! 使用微服务架构标准模式

pub mod grpc;
pub mod http_handlers;
pub mod repository;
pub mod timeout; // 任务超时处理模块

// 导出 Repository
pub use repository::{
    CarClassRepository, CarColorRepository, CarRepository, CarRepositoryError, CarTypeRepository,
    DcCausesRepository, DcTypeRepository,
};

// 导出数据模型
pub use repository::{
    Car, CarClass, CarClassListItem, CarColor, CarColorListItem, CarListItem, CarQuery, CarType,
    CarTypeListItem, DcCauses, DcCausesListItem, DcType, DcTypeListItem, PaginatedCars,
};

// 导出 HTTP Handler 状态
pub use http_handlers::HttpAppState;

// 导出超时模块类型
pub use timeout::{
    CheckResult, HandleResult, Task, TimeoutConfig, TimeoutEvent, TimeoutEventType, TimeoutManager,
    TimeoutState, TimeoutStats, TimeoutStrategy,
};

// ============ CarQueryParams 实现 ============

/// HTTP 请求参数 -> `CarQuery` 转换
///
/// 从 HTTP 查询参数转换为数据库查询条件
impl From<http_handlers::CarQueryParams> for CarQuery {
    fn from(params: http_handlers::CarQueryParams) -> Self {
        Self {
            status: params.status,
            in_date: params.in_date,
            out_date: params.out_date,
            content: params.content,
            model: params.model,
            name: params.name,
            unit: params.unit,
            key: params.key,
            page: params.page,
            page_size: params.page_size,
            sort_by: params.sort_by,
            descending: params.descending,
        }
    }
}
