//! Event core library for event-driven architecture
//!
//! Provides event bus, event types, and handler traits for decoupled service communication.

pub mod bus;
pub mod event;
pub mod handler;

pub use bus::{EventBus, EventBusError};
pub use event::{Event, EventMetadata, EventType};
pub use handler::{EventHandler, HandlerResult};
