//! Integration tests for event bus

use event_core::{Event, EventBus, EventMetadata, EventType};

#[tokio::test]
async fn test_event_bus_publish_subscribe() {
    let bus = EventBus::new();
    let mut rx = bus.subscribe();

    let metadata = EventMetadata::new(EventType::UserCreated, "test-service")
        .with_tenant_id("tenant-001")
        .with_user_id("user-001");

    let event = Event::new(
        metadata,
        serde_json::json!({"username": "testuser", "email": "test@example.com"}),
    );

    let subscriber_count = bus.publish(event).unwrap();
    assert_eq!(subscriber_count, 1);

    let received = rx.recv().await.unwrap();
    assert_eq!(received.event_type(), EventType::UserCreated);
    assert_eq!(received.metadata.tenant_id, Some("tenant-001".to_string()));
    assert_eq!(received.payload["username"], "testuser");
}

#[test]
fn test_event_metadata_builder() {
    let metadata = EventMetadata::new(EventType::WorkflowCompleted, "workflow-service")
        .with_tenant_id("tenant-123")
        .with_user_id("user-456")
        .with_correlation_id("corr-abc");

    assert_eq!(metadata.event_type, EventType::WorkflowCompleted);
    assert_eq!(metadata.source, "workflow-service");
    assert_eq!(metadata.tenant_id, Some("tenant-123".to_string()));
    assert_eq!(metadata.user_id, Some("user-456".to_string()));
    assert_eq!(metadata.correlation_id, Some("corr-abc".to_string()));
    assert!(!metadata.event_id.is_nil());
}

#[tokio::test]
async fn test_event_bus_multiple_subscribers() {
    let bus = EventBus::new();
    let mut rx1 = bus.subscribe();
    let mut rx2 = bus.subscribe();
    let mut rx3 = bus.subscribe();

    assert_eq!(bus.subscriber_count(), 3);

    let metadata = EventMetadata::new(EventType::PaymentCompleted, "pay-service");
    let event = Event::new(
        metadata,
        serde_json::json!({"order_id": "ord-001", "amount": 199.99}),
    );

    let count = bus.publish(event).unwrap();
    assert_eq!(count, 3);

    // All subscribers should receive the event
    let event1 = rx1.recv().await.unwrap();
    let event2 = rx2.recv().await.unwrap();
    let event3 = rx3.recv().await.unwrap();

    assert_eq!(event1.event_type(), EventType::PaymentCompleted);
    assert_eq!(event2.event_type(), EventType::PaymentCompleted);
    assert_eq!(event3.event_type(), EventType::PaymentCompleted);

    // All should have the same event ID
    assert_eq!(event1.metadata.event_id, event2.metadata.event_id);
    assert_eq!(event2.metadata.event_id, event3.metadata.event_id);
}
