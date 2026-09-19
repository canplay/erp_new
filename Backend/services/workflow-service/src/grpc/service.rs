use std::sync::Arc;

use crate::grpc::info::*;

/// Workflow gRPC 服务实现
#[derive(Clone)]
pub struct WorkflowGrpcService {
    state: Arc<WorkflowAppState>,
}

impl WorkflowGrpcService {
    pub const fn new(state: Arc<WorkflowAppState>) -> Self {
        Self { state }
    }

    #[must_use]
    pub const fn state(&self) -> &Arc<WorkflowAppState> {
        &self.state
    }
}
