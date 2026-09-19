//! Workflow Service Library
//!
//! 合并工作流、报表和调度功能

pub mod engine; // 工作流执行引擎
pub mod events; // Workflow event emission module
pub mod extension;
pub mod grpc; // gRPC 服务模块
pub mod grpc_server; // gRPC 服务实现
pub mod http; // HTTP 服务模块
pub mod models; // 导出数据模型
pub mod repository; // 仓储层（数据库 CRUD）

// 导出 HTTP 状态类型
pub use http::{AppState, WorkflowState};

// 导出模型类型
pub use models::{
    InstanceStatus, NodeStatus, NodeType, Report, ReportStatus, ReportTask, ScheduledTask,
    ScheduledTaskStatus, TaskRecord, Workflow, WorkflowInstance, WorkflowStatus,
};

// 导出仓储类型
pub use repository::{
    event_repository::{
        InMemoryReportRepository, InMemoryScheduledTaskRepository, ReportRepository,
        ScheduledTaskRepository,
    },
    instance_repository::{InstanceRepository, PostgresInstanceRepository},
    task_repository::{PostgresTaskRepository, TaskRepository},
    workflow_repository::{
        InMemoryWorkflowRepository, PostgresWorkflowRepository, WorkflowRepository,
    },
    Workflow as WorkflowModel, WorkflowEdge, WorkflowInstance as WorkflowInstanceModel,
    WorkflowNode, ScheduledTask as ScheduledTaskModel, TaskRecord as TaskRecordModel,
};

// 导出 gRPC 类型
pub use grpc::{WorkflowAppState, WorkflowGrpcService, WorkflowInfo};

// 导出引擎类型
pub use common::AppError;
pub use common::AppResult;
pub use engine::{EngineError, EngineResult, TaskScheduler, WorkflowEngine};

// 导出事件类型
pub use events::{
    EventFilter, EventPriority, TaskAssignedData, TaskCompletedData, WorkflowCompletedData,
    WorkflowEvent, WorkflowEventEmitter, WorkflowEventError, WorkflowEventResult,
    WorkflowEventType, WorkflowFailedData,
};

// 导出节点扩展类型
pub use extension::{
    ConfigParam, NodeConnection, NodeExecutionContext, NodeExecutor, NodeExtensionConfig,
    NodeManager, NodeState, NodeStats, Port, PortDefinition, ValidationError, ValidationRule,
    WorkflowNode as NodeGraphNode,
};
