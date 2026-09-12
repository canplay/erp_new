//! Event types and metadata definitions

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// Event types for domain events across services
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    // Tenant events
    TenantCreated,
    TenantUpdated,

    // User events
    UserCreated,
    UserUpdated,
    UserLoggedIn,

    // Workflow events
    WorkflowCompleted,
    WorkflowFailed,

    // Payment events
    PaymentCompleted,
    PaymentFailed,

    // Notification events
    NotificationSent,
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            EventType::TenantCreated => "tenant.created",
            EventType::TenantUpdated => "tenant.updated",
            EventType::UserCreated => "user.created",
            EventType::UserUpdated => "user.updated",
            EventType::UserLoggedIn => "user.logged_in",
            EventType::WorkflowCompleted => "workflow.completed",
            EventType::WorkflowFailed => "workflow.failed",
            EventType::PaymentCompleted => "payment.completed",
            EventType::PaymentFailed => "payment.failed",
            EventType::NotificationSent => "notification.sent",
        };
        write!(f, "{}", s)
    }
}

/// Metadata associated with an event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    /// Unique event identifier
    pub event_id: Uuid,
    /// Type of the event
    pub event_type: EventType,
    /// Tenant identifier (for multi-tenant isolation)
    pub tenant_id: Option<String>,
    /// User identifier (who triggered the event)
    pub user_id: Option<String>,
    /// Timestamp when the event was created
    pub timestamp: DateTime<Utc>,
    /// Source service that emitted the event
    pub source: String,
    /// Correlation ID for tracing across services
    pub correlation_id: Option<String>,
}

impl EventMetadata {
    /// Create new metadata with required fields
    pub fn new(event_type: EventType, source: impl Into<String>) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            event_type,
            tenant_id: None,
            user_id: None,
            timestamp: Utc::now(),
            source: source.into(),
            correlation_id: None,
        }
    }

    /// Set tenant ID
    pub fn with_tenant_id(mut self, tenant_id: impl Into<String>) -> Self {
        self.tenant_id = Some(tenant_id.into());
        self
    }

    /// Set user ID
    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// Set correlation ID
    pub fn with_correlation_id(mut self, correlation_id: impl Into<String>) -> Self {
        self.correlation_id = Some(correlation_id.into());
        self
    }
}

/// A domain event with metadata and payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Event metadata
    pub metadata: EventMetadata,
    /// Event payload (JSON value)
    pub payload: Value,
}

impl Event {
    /// Create a new event with metadata and payload
    pub fn new(metadata: EventMetadata, payload: Value) -> Self {
        Self { metadata, payload }
    }

    /// Create a new event from metadata builder with a serializable payload
    pub fn with_payload(
        metadata: EventMetadata,
        payload: impl Serialize,
    ) -> Result<Self, serde_json::Error> {
        let value = serde_json::to_value(payload)?;
        Ok(Self::new(metadata, value))
    }

    /// Get the event type
    pub fn event_type(&self) -> EventType {
        self.metadata.event_type
    }

    /// Get the event ID
    pub fn event_id(&self) -> Uuid {
        self.metadata.event_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_display() {
        assert_eq!(EventType::TenantCreated.to_string(), "tenant.created");
        assert_eq!(EventType::UserLoggedIn.to_string(), "user.logged_in");
        assert_eq!(EventType::WorkflowFailed.to_string(), "workflow.failed");
    }

    #[test]
    fn test_event_metadata_builder() {
        let metadata = EventMetadata::new(EventType::UserCreated, "test-service")
            .with_tenant_id("tenant-123")
            .with_user_id("user-456")
            .with_correlation_id("corr-789");

        assert_eq!(metadata.event_type, EventType::UserCreated);
        assert_eq!(metadata.source, "test-service");
        assert_eq!(metadata.tenant_id, Some("tenant-123".to_string()));
        assert_eq!(metadata.user_id, Some("user-456".to_string()));
        assert_eq!(metadata.correlation_id, Some("corr-789".to_string()));
    }

    #[test]
    fn test_event_with_payload() {
        let metadata = EventMetadata::new(EventType::PaymentCompleted, "pay-service");
        let payload = serde_json::json!({"order_id": "ord-001", "amount": 99.99});

        let event = Event::with_payload(metadata, &payload).unwrap();
        assert_eq!(event.event_type(), EventType::PaymentCompleted);
        assert_eq!(event.payload["order_id"], "ord-001");
    }
}
