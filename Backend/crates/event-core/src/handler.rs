//! Event handler traits for processing events

use async_trait::async_trait;

use crate::event::Event;
use crate::event::EventType;

/// Result type for event handlers
pub type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

/// Trait for handling events asynchronously
///
/// Implementors can process events and return a result indicating
/// success or failure. Handlers should be idempotent as they may
/// be called multiple times for the same event.
#[async_trait]
pub trait EventHandler: Send + Sync {
    /// Handle an event
    ///
    /// Returns Ok(()) if the event was handled successfully,
    /// or an error if processing failed.
    async fn handle(&self, event: &Event) -> HandlerResult;

    /// The event types this handler is interested in
    ///
    /// Return an empty slice to receive all events.
    fn event_types(&self) -> &[EventType] {
        &[]
    }

    /// Handler name for logging and debugging
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}

/// A boxed event handler for type erasure
pub type BoxedHandler = Box<dyn EventHandler>;

/// Helper function to check if a handler is interested in an event type
pub fn handler_interested_in(handler: &dyn EventHandler, event_type: &EventType) -> bool {
    let types = handler.event_types();
    types.is_empty() || types.contains(event_type)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{EventMetadata, EventType};

    struct TestHandler {
        pub called: std::sync::Arc<tokio::sync::Mutex<bool>>,
    }

    #[async_trait]
    impl EventHandler for TestHandler {
        async fn handle(&self, _event: &Event) -> HandlerResult {
            *self.called.lock().await = true;
            Ok(())
        }

        fn event_types(&self) -> &[EventType] {
            &[EventType::UserCreated]
        }

        fn name(&self) -> &str {
            "TestHandler"
        }
    }

    #[tokio::test]
    async fn test_handler_called() {
        let called = std::sync::Arc::new(tokio::sync::Mutex::new(false));
        let handler = TestHandler { called: called.clone() };

        let metadata = EventMetadata::new(EventType::UserCreated, "test");
        let event = Event::new(metadata, serde_json::json!({}));

        handler.handle(&event).await.unwrap();
        assert!(*called.lock().await);
    }

    #[test]
    fn test_handler_interested_in() {
        struct AllEventsHandler;
        #[async_trait]
        impl EventHandler for AllEventsHandler {
            async fn handle(&self, _event: &Event) -> HandlerResult {
                Ok(())
            }
        }

        let handler = AllEventsHandler;
        assert!(handler_interested_in(&handler, &EventType::UserCreated));
        assert!(handler_interested_in(&handler, &EventType::PaymentFailed));
    }
}
