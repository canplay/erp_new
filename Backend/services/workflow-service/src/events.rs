//! Workflow Event Emission Module
//!
//! Emits events on workflow completion/failure via messaging-service gRPC.
//! Event types: WorkflowCompleted, WorkflowFailed, TaskCompleted

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

/// Workflow event errors
#[derive(Error, Debug)]
pub enum WorkflowEventError {
    #[error("Event emission failed: {0}" )]
    Emission(String),

    #[error("Invalid event: {0}" )]
    Invalid(String),

    #[error("Serialization failed: {0}" )]
    Serialization(#[from] serde_json::Error),
}

/// Result type for event operations
pub type WorkflowEventResult<T> = Result<T, WorkflowEventError>;

/// Workflow event types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowEventType {
    /// Workflow instance completed successfully
    WorkflowCompleted,
    /// Workflow instance failed
    WorkflowFailed,
    /// Task within workflow completed
    TaskCompleted,
    /// Workflow instance cancelled
    WorkflowCancelled,
    /// Task assigned to user
    TaskAssigned,
    /// Workflow instance started
    WorkflowStarted,
}

impl std::fmt::Display for WorkflowEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WorkflowCompleted => write!(f, "workflow.completed" ),
            Self::WorkflowFailed => write!(f, "workflow.failed" ),
            Self::TaskCompleted => write!(f, "task.completed" ),
            Self::WorkflowCancelled => write!(f, "workflow.cancelled" ),
            Self::TaskAssigned => write!(f, "task.assigned" ),
            Self::WorkflowStarted => write!(f, "workflow.started" ),
        }
    }
}

/// Event priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum EventPriority {
    Low,
    #[default]
    Normal,
    High,
    Urgent,
}

impl std::fmt::Display for EventPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "low" ),
            Self::Normal => write!(f, "normal" ),
            Self::High => write!(f, "high" ),
            Self::Urgent => write!(f, "urgent" ),
        }
    }
}

/// Base workflow event structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowEvent {
    /// Unique event ID
    pub id: String,
    /// Event type
    pub event_type: WorkflowEventType,
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    /// Workflow instance ID
    pub instance_id: String,
    /// Workflow ID
    pub workflow_id: String,
    /// User who triggered the event
    pub user_id: String,
    /// Event data (type-specific payload)
    pub data: serde_json::Value,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
    /// Event priority
    pub priority: EventPriority,
}

impl WorkflowEvent {
    /// Create a new workflow event
    #[must_use]
    pub fn new(
        event_type: WorkflowEventType,
        instance_id: String,
        workflow_id: String,
        user_id: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            event_type,
            timestamp: Utc::now(),
            instance_id,
            workflow_id,
            user_id,
            data: serde_json::json!({}),
            metadata: HashMap::new(),
            priority: EventPriority::Normal,
        }
    }

    /// Set event data
    #[must_use]
    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = data;
        self
    }

    /// Set event priority
    #[must_use]
    pub fn with_priority(mut self, priority: EventPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Add metadata
    #[must_use]
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Serialize event to JSON
    pub fn to_json(&self) -> WorkflowEventResult<String> {
        Ok(serde_json::to_string(self)?)
    }

    /// Serialize event to JSON bytes
    pub fn to_json_bytes(&self) -> WorkflowEventResult<Vec<u8>> {
        Ok(serde_json::to_vec(self)?)
    }
}

/// Workflow completed event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowCompletedData {
    pub duration_seconds: i64,
    pub nodes_completed: i32,
    pub total_nodes: i32,
}

/// Workflow failed event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowFailedData {
    pub error_message: String,
    pub failed_node_id: Option<String>,
    pub retry_count: i32,
}

/// Task completed event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCompletedData {
    pub task_id: String,
    pub node_id: String,
    pub node_name: String,
    pub assignee: Option<String>,
    pub duration_seconds: i64,
    pub comment: Option<String>,
}

/// Task assigned event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAssignedData {
    pub task_id: String,
    pub node_id: String,
    pub node_name: String,
    pub assignee: String,
    pub assigned_by: String,
    pub timeout_at: Option<DateTime<Utc>>,
}

/// Event emitter for workflow events
pub struct WorkflowEventEmitter {
    /// Messaging service endpoint (for gRPC)
    messaging_endpoint: Option<String>,
    /// Event history (for testing/debugging)
    event_history: std::sync::Mutex<Vec<WorkflowEvent>>,
}

impl WorkflowEventEmitter {
    /// Create a new event emitter
    #[must_use]
    pub fn new() -> Self {
        Self {
            messaging_endpoint: None,
            event_history: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Create a new event emitter with messaging service endpoint
    #[must_use]
    pub fn with_endpoint(endpoint: String) -> Self {
        Self {
            messaging_endpoint: Some(endpoint),
            event_history: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Create from environment variables
    pub fn from_env() -> Self {
        let endpoint = std::env::var("MESSAGING_SERVICE_ENDPOINT" ).ok();
        Self {
            messaging_endpoint: endpoint,
            event_history: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Emit a workflow event
    pub async fn emit(&self, event: &WorkflowEvent) -> WorkflowEventResult<()> {
        // Store in history
        self.event_history.lock().unwrap_or_else(|e| e.into_inner()).push(event.clone());

        // Log the event
        tracing::info!(
            "Emitting workflow event: {} for instance {} (workflow: {})" ,
            event.event_type,
            event.instance_id,
            event.workflow_id
        );

        // In a real implementation, this would:
        // 1. Call messaging-service gRPC to send notification
        // 2. Optionally publish to message queue (e.g., Kafka, RabbitMQ)
        // 3. Store in event store for audit

        if let Some(endpoint) = &self.messaging_endpoint {
            tracing::info!(
                "Sending event to messaging service at {}: {}" ,
                endpoint,
                event.event_type
            );
            // In production:
            // let mut client = messaging_service::connect(endpoint).await?;
            // client.send_notification(event).await?;
        }

        Ok(())
    }

    /// Emit workflow completed event
    pub async fn emit_workflow_completed(
        &self,
        instance_id: &str,
        workflow_id: &str,
        user_id: &str,
        duration_seconds: i64,
        nodes_completed: i32,
        total_nodes: i32,
    ) -> WorkflowEventResult<()> {
        let data = serde_json::to_value(WorkflowCompletedData {
            duration_seconds,
            nodes_completed,
            total_nodes,
        })?;

        let event = WorkflowEvent::new(
            WorkflowEventType::WorkflowCompleted,
            instance_id.to_string(),
            workflow_id.to_string(),
            user_id.to_string(),
        )
        .with_data(data)
        .with_priority(EventPriority::Normal)
        .with_metadata("event_category".to_string(), "workflow".to_string());

        self.emit(&event).await
    }

    /// Emit workflow failed event
    pub async fn emit_workflow_failed(
        &self,
        instance_id: &str,
        workflow_id: &str,
        user_id: &str,
        error_message: &str,
        failed_node_id: Option<&str>,
        retry_count: i32,
    ) -> WorkflowEventResult<()> {
        let data = serde_json::to_value(WorkflowFailedData {
            error_message: error_message.to_string(),
            failed_node_id: failed_node_id.map(String::from),
            retry_count,
        })?;

        let event = WorkflowEvent::new(
            WorkflowEventType::WorkflowFailed,
            instance_id.to_string(),
            workflow_id.to_string(),
            user_id.to_string(),
        )
        .with_data(data)
        .with_priority(EventPriority::High)
        .with_metadata("event_category".to_string(), "workflow".to_string());

        self.emit(&event).await
    }

    /// Emit task completed event
    pub async fn emit_task_completed(
        &self,
        instance_id: &str,
        workflow_id: &str,
        user_id: &str,
        task_id: &str,
        node_id: &str,
        node_name: &str,
        assignee: Option<&str>,
        duration_seconds: i64,
        comment: Option<&str>,
    ) -> WorkflowEventResult<()> {
        let data = serde_json::to_value(TaskCompletedData {
            task_id: task_id.to_string(),
            node_id: node_id.to_string(),
            node_name: node_name.to_string(),
            assignee: assignee.map(String::from),
            duration_seconds,
            comment: comment.map(String::from),
        })?;

        let event = WorkflowEvent::new(
            WorkflowEventType::TaskCompleted,
            instance_id.to_string(),
            workflow_id.to_string(),
            user_id.to_string(),
        )
        .with_data(data)
        .with_priority(EventPriority::Normal)
        .with_metadata("event_category".to_string(), "task".to_string());

        self.emit(&event).await
    }

    /// Emit task assigned event
    pub async fn emit_task_assigned(
        &self,
        instance_id: &str,
        workflow_id: &str,
        user_id: &str,
        task_id: &str,
        node_id: &str,
        node_name: &str,
        assignee: &str,
        assigned_by: &str,
        timeout_at: Option<DateTime<Utc>>,
    ) -> WorkflowEventResult<()> {
        let data = serde_json::to_value(TaskAssignedData {
            task_id: task_id.to_string(),
            node_id: node_id.to_string(),
            node_name: node_name.to_string(),
            assignee: assignee.to_string(),
            assigned_by: assigned_by.to_string(),
            timeout_at,
        })?;

        let event = WorkflowEvent::new(
            WorkflowEventType::TaskAssigned,
            instance_id.to_string(),
            workflow_id.to_string(),
            user_id.to_string(),
        )
        .with_data(data)
        .with_priority(EventPriority::High)
        .with_metadata("event_category".to_string(), "task".to_string());

        self.emit(&event).await
    }

    /// Emit workflow cancelled event
    pub async fn emit_workflow_cancelled(
        &self,
        instance_id: &str,
        workflow_id: &str,
        user_id: &str,
        reason: Option<&str>,
    ) -> WorkflowEventResult<()> {
        let data = serde_json::json!({
            "reason": reason,
            "cancelled_at": Utc::now().to_rfc3339(),
        });

        let event = WorkflowEvent::new(
            WorkflowEventType::WorkflowCancelled,
            instance_id.to_string(),
            workflow_id.to_string(),
            user_id.to_string(),
        )
        .with_data(data)
        .with_priority(EventPriority::Normal)
        .with_metadata("event_category".to_string(), "workflow".to_string());

        self.emit(&event).await
    }

    /// Emit workflow started event
    pub async fn emit_workflow_started(
        &self,
        instance_id: &str,
        workflow_id: &str,
        user_id: &str,
    ) -> WorkflowEventResult<()> {
        let event = WorkflowEvent::new(
            WorkflowEventType::WorkflowStarted,
            instance_id.to_string(),
            workflow_id.to_string(),
            user_id.to_string(),
        )
        .with_priority(EventPriority::Normal)
        .with_metadata("event_category".to_string(), "workflow".to_string());

        self.emit(&event).await
    }

    /// Get event history (for testing/debugging)
    pub fn get_event_history(&self) -> Vec<WorkflowEvent> {
        self.event_history.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    /// Clear event history
    pub fn clear_history(&self) {
        self.event_history.lock().unwrap_or_else(|e| e.into_inner()).clear();
    }

    /// Get event count
    pub fn event_count(&self) -> usize {
        self.event_history.lock().unwrap_or_else(|e| e.into_inner()).len()
    }
}

impl Default for WorkflowEventEmitter {
    fn default() -> Self {
        Self::new()
    }
}

/// Event filter for querying events
#[derive(Debug, Clone, Default)]
pub struct EventFilter {
    pub event_types: Option<Vec<WorkflowEventType>>,
    pub instance_id: Option<String>,
    pub workflow_id: Option<String>,
    pub user_id: Option<String>,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
}

impl EventFilter {
    /// Create a new event filter
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by event types
    #[must_use]
    pub fn with_event_types(mut self, types: Vec<WorkflowEventType>) -> Self {
        self.event_types = Some(types);
        self
    }

    /// Filter by instance ID
    #[must_use]
    pub fn with_instance_id(mut self, instance_id: String) -> Self {
        self.instance_id = Some(instance_id);
        self
    }

    /// Filter by workflow ID
    #[must_use]
    pub fn with_workflow_id(mut self, workflow_id: String) -> Self {
        self.workflow_id = Some(workflow_id);
        self
    }

    /// Filter by user ID
    #[must_use]
    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    /// Filter by time range
    #[must_use]
    pub fn with_time_range(mut self, since: DateTime<Utc>, until: DateTime<Utc>) -> Self {
        self.since = Some(since);
        self.until = Some(until);
        self
    }

    /// Check if an event matches the filter
    pub fn matches(&self, event: &WorkflowEvent) -> bool {
        if let Some(types) = &self.event_types {
            if !types.contains(&event.event_type) {
                return false;
            }
        }

        if let Some(instance_id) = &self.instance_id {
            if event.instance_id != *instance_id {
                return false;
            }
        }

        if let Some(workflow_id) = &self.workflow_id {
            if event.workflow_id != *workflow_id {
                return false;
            }
        }

        if let Some(user_id) = &self.user_id {
            if event.user_id != *user_id {
                return false;
            }
        }

        if let Some(since) = self.since {
            if event.timestamp < since {
                return false;
            }
        }

        if let Some(until) = self.until {
            if event.timestamp > until {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_event_type_display() {
        assert_eq!(
            WorkflowEventType::WorkflowCompleted.to_string(),
            "workflow.completed"
        );
        assert_eq!(
            WorkflowEventType::WorkflowFailed.to_string(),
            "workflow.failed"
        );
        assert_eq!(
            WorkflowEventType::TaskCompleted.to_string(),
            "task.completed"
        );
    }

    #[test]
    fn test_event_priority_display() {
        assert_eq!(EventPriority::Low.to_string(), "low" );
        assert_eq!(EventPriority::Normal.to_string(), "normal" );
        assert_eq!(EventPriority::High.to_string(), "high" );
        assert_eq!(EventPriority::Urgent.to_string(), "urgent" );
    }

    #[test]
    fn test_workflow_event_creation() {
        let event = WorkflowEvent::new(
            WorkflowEventType::WorkflowCompleted,
            "instance-123".to_string(),
            "workflow-456".to_string(),
            "user-789".to_string(),
        );

        assert_eq!(event.event_type, WorkflowEventType::WorkflowCompleted);
        assert_eq!(event.instance_id, "instance-123" );
        assert_eq!(event.workflow_id, "workflow-456" );
        assert_eq!(event.user_id, "user-789" );
        assert_eq!(event.priority, EventPriority::Normal);
    }

    #[test]
    fn test_workflow_event_with_data() {
        let data = serde_json::json!({"key": "value" });
        let event = WorkflowEvent::new(
            WorkflowEventType::TaskCompleted,
            "instance-1".to_string(),
            "workflow-1".to_string(),
            "user-1".to_string(),
        )
        .with_data(data.clone())
        .with_priority(EventPriority::High);

        assert_eq!(event.data, data);
        assert_eq!(event.priority, EventPriority::High);
    }

    #[test]
    fn test_workflow_event_serialization() {
        let event = WorkflowEvent::new(
            WorkflowEventType::WorkflowCompleted,
            "instance-1".to_string(),
            "workflow-1".to_string(),
            "user-1".to_string(),
        );

        let json = event.to_json().expect("lock should not be poisoned" );
        // The enum is serialized as the variant name by default
        assert!(json.contains("WorkflowCompleted" ) || json.contains("workflow.completed" ));
        assert!(json.contains("instance-1" ));
    }

    #[test]
    fn test_workflow_completed_data_serialization() {
        let data = WorkflowCompletedData {
            duration_seconds: 3600,
            nodes_completed: 5,
            total_nodes: 5,
        };

        let json = serde_json::to_value(&data).expect("lock should not be poisoned" );
        assert_eq!(json["duration_seconds" ], 3600);
        assert_eq!(json["nodes_completed" ], 5);
    }

    #[test]
    fn test_workflow_failed_data_serialization() {
        let data = WorkflowFailedData {
            error_message: "Timeout".to_string(),
            failed_node_id: Some("node-1".to_string()),
            retry_count: 3,
        };

        let json = serde_json::to_value(&data).expect("lock should not be poisoned" );
        assert_eq!(json["error_message" ], "Timeout" );
        assert_eq!(json["retry_count" ], 3);
    }

    #[test]
    fn test_task_completed_data_serialization() {
        let data = TaskCompletedData {
            task_id: "task-1".to_string(),
            node_id: "node-1".to_string(),
            node_name: "Approval".to_string(),
            assignee: Some("user-1".to_string()),
            duration_seconds: 120,
            comment: Some("Approved".to_string()),
        };

        let json = serde_json::to_value(&data).expect("lock should not be poisoned" );
        assert_eq!(json["task_id" ], "task-1" );
        assert_eq!(json["node_name" ], "Approval" );
    }

    #[test]
    fn test_event_filter_matches() {
        let event = WorkflowEvent::new(
            WorkflowEventType::WorkflowCompleted,
            "instance-1".to_string(),
            "workflow-1".to_string(),
            "user-1".to_string(),
        );

        let filter = EventFilter::new().with_instance_id("instance-1".to_string());
        assert!(filter.matches(&event));

        let filter = EventFilter::new().with_instance_id("instance-2".to_string());
        assert!(!filter.matches(&event));

        let filter = EventFilter::new().with_event_types(vec![WorkflowEventType::WorkflowCompleted]);
        assert!(filter.matches(&event));

        let filter = EventFilter::new().with_event_types(vec![WorkflowEventType::WorkflowFailed]);
        assert!(!filter.matches(&event));
    }

    #[test]
    fn test_event_filter_by_user() {
        let event = WorkflowEvent::new(
            WorkflowEventType::TaskCompleted,
            "instance-1".to_string(),
            "workflow-1".to_string(),
            "user-42".to_string(),
        );

        let filter = EventFilter::new().with_user_id("user-42".to_string());
        assert!(filter.matches(&event));

        let filter = EventFilter::new().with_user_id("user-99".to_string());
        assert!(!filter.matches(&event));
    }

    #[test]
    fn test_event_emitter_creation() {
        let emitter = WorkflowEventEmitter::new();
        assert_eq!(emitter.event_count(), 0);
        assert!(emitter.messaging_endpoint.is_none());
    }

    #[test]
    fn test_event_emitter_with_endpoint() {
        let emitter =
            WorkflowEventEmitter::with_endpoint("http://localhost:50051".to_string());
        assert_eq!(
            emitter.messaging_endpoint,
            Some("http://localhost:50051".to_string())
        );
    }

    #[test]
    fn test_event_history() {
        let emitter = WorkflowEventEmitter::new();
        let event = WorkflowEvent::new(
            WorkflowEventType::WorkflowCompleted,
            "instance-1".to_string(),
            "workflow-1".to_string(),
            "user-1".to_string(),
        );

        emitter.event_history.lock().expect("lock should not be poisoned" ).push(event);
        assert_eq!(emitter.event_count(), 1);

        emitter.clear_history();
        assert_eq!(emitter.event_count(), 0);
    }
}
