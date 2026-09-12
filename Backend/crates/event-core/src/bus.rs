//! Event bus implementation using tokio broadcast channel

use std::sync::Arc;

use tokio::sync::broadcast;
use tracing::{debug, error};

use crate::event::Event;

/// Default channel capacity for the event bus
pub const DEFAULT_CHANNEL_CAPACITY: usize = 1024;

/// Errors that can occur when using the event bus
#[derive(Debug, thiserror::Error)]
pub enum EventBusError {
    #[error("Failed to publish event: {0}")]
    PublishError(String),

    #[error("Event bus channel is full (capacity: {capacity})")]
    ChannelFull { capacity: usize },

    #[error("No active subscribers for event")]
    NoSubscribers,
}

/// Result type for event bus operations
pub type EventBusResult<T> = Result<T, EventBusError>;

/// Event bus for publishing and subscribing to events
///
/// Uses tokio's broadcast channel for multi-producer, multi-consumer
/// event distribution. Events are delivered to all active subscribers.
#[derive(Debug)]
pub struct EventBus {
    sender: broadcast::Sender<Event>,
}

impl EventBus {
    /// Create a new event bus with the default channel capacity (1024)
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_CHANNEL_CAPACITY)
    }

    /// Create a new event bus with a custom channel capacity
    pub fn with_capacity(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Publish an event to all subscribers
    ///
    /// Returns the number of subscribers that will receive the event,
    /// or an error if the publish fails.
    pub fn publish(&self, event: Event) -> EventBusResult<usize> {
        let event_id = event.metadata.event_id;
        let event_type = event.metadata.event_type;

        match self.sender.send(event) {
            Ok(count) => {
                debug!(
                    event_id = %event_id,
                    event_type = %event_type,
                    subscribers = count,
                    "Event published successfully"
                );
                Ok(count)
            }
            Err(e) => {
                error!(
                    event_id = %event_id,
                    event_type = %event_type,
                    error = %e,
                    "Failed to publish event"
                );
                Err(EventBusError::PublishError(e.to_string()))
            }
        }
    }

    /// Subscribe to the event bus and receive a receiver for events
    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.sender.subscribe()
    }

    /// Get the number of active subscribers
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }

    /// Get the number of messages in the channel
    pub fn len(&self) -> usize {
        self.sender.len()
    }

    /// Check if the channel is empty
    pub fn is_empty(&self) -> bool {
        self.sender.len() == 0
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe shared event bus
pub type SharedEventBus = Arc<EventBus>;

/// Create a new shared event bus wrapped in an Arc
pub fn shared_event_bus() -> SharedEventBus {
    Arc::new(EventBus::new())
}

/// Create a new shared event bus with custom capacity
pub fn shared_event_bus_with_capacity(capacity: usize) -> SharedEventBus {
    Arc::new(EventBus::with_capacity(capacity))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{EventMetadata, EventType};

    #[tokio::test]
    async fn test_publish_and_subscribe() {
        let bus = EventBus::new();
        let mut rx = bus.subscribe();

        let metadata = EventMetadata::new(EventType::UserCreated, "test");
        let event = Event::new(metadata, serde_json::json!({"user_id": "123"}));

        let count = bus.publish(event).unwrap();
        assert_eq!(count, 1);

        let received = rx.recv().await.unwrap();
        assert_eq!(received.event_type(), EventType::UserCreated);
    }

    #[test]
    fn test_subscriber_count() {
        let bus = EventBus::new();
        assert_eq!(bus.subscriber_count(), 0);

        let _rx1 = bus.subscribe();
        let _rx2 = bus.subscribe();
        assert_eq!(bus.subscriber_count(), 2);
    }
}
