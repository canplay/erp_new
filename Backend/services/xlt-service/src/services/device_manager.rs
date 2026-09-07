//! 设备管理器
//!
//! 管理信路通设备注册、心跳更新和 MQTT 命令分发。

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::Serialize;

use crate::models::MqttEnvelope;

type DeviceHandler = Arc<dyn Fn(MqttEnvelope) + Send + Sync>;

/// 设备管理器
pub struct DeviceManager {
    devices: RwLock<HashMap<String, DeviceInfo>>,
    handlers: RwLock<HashMap<String, Vec<DeviceHandler>>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeviceInfo {
    pub sn: String,
    pub client_id: String,
    pub dev_info: String,
    pub version: String,
    pub connected_at: String,
    pub last_heartbeat: String,
}

impl Default for DeviceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DeviceManager {
    #[must_use]
    pub fn new() -> Self {
        Self {
            devices: RwLock::new(HashMap::new()),
            handlers: RwLock::new(HashMap::new()),
        }
    }

    pub async fn register_device(&self, sn: &str, client_id: &str, dev_info: &str, version: &str) {
        let now = chrono::Utc::now().to_rfc3339();
        let info = DeviceInfo {
            sn: sn.to_string(),
            client_id: client_id.to_string(),
            dev_info: dev_info.to_string(),
            version: version.to_string(),
            connected_at: now.clone(),
            last_heartbeat: now,
        };
        self.devices.write().await.insert(sn.to_string(), info);
        tracing::info!("设备注册: sn={sn}, client_id={client_id}");
    }

    pub async fn update_heartbeat(&self, sn: &str) {
        if let Some(info) = self.devices.write().await.get_mut(sn) {
            info.last_heartbeat = chrono::Utc::now().to_rfc3339();
        }
    }

    pub async fn get_device(&self, sn: &str) -> Option<DeviceInfo> {
        self.devices.read().await.get(sn).cloned()
    }

    pub async fn list_devices(&self) -> Vec<DeviceInfo> {
        self.devices.read().await.values().cloned().collect()
    }

    pub async fn register_handler(&self, command: &str, handler: DeviceHandler) {
        self.handlers
            .write()
            .await
            .entry(command.to_string())
            .or_default()
            .push(handler);
    }

    pub async fn dispatch(&self, envelope: MqttEnvelope) {
        let handlers = self.handlers.read().await.get(&envelope.command).cloned();
        if let Some(handlers) = handlers {
            for handler in &handlers {
                handler(envelope.clone());
            }
        } else {
            tracing::warn!("未知命令: {}, sn={}", envelope.command, envelope.sn);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_and_get_device() {
        let manager = DeviceManager::new();
        manager.register_device("SN001", "client-1", "info1", "1.0").await;

        let device = manager.get_device("SN001").await;
        assert!(device.is_some());
        let device = device.unwrap();
        assert_eq!(device.sn, "SN001");
        assert_eq!(device.client_id, "client-1");
        assert_eq!(device.dev_info, "info1");
        assert_eq!(device.version, "1.0");
    }

    #[tokio::test]
    async fn test_get_nonexistent_device() {
        let manager = DeviceManager::new();
        let device = manager.get_device("NONEXISTENT").await;
        assert!(device.is_none());
    }

    #[tokio::test]
    async fn test_list_devices_multiple() {
        let manager = DeviceManager::new();
        manager.register_device("SN001", "client-1", "info1", "1.0").await;
        manager.register_device("SN002", "client-2", "info2", "2.0").await;

        let devices = manager.list_devices().await;
        assert_eq!(devices.len(), 2);
    }

    #[tokio::test]
    async fn test_list_devices_empty() {
        let manager = DeviceManager::new();
        let devices = manager.list_devices().await;
        assert!(devices.is_empty());
    }

    #[tokio::test]
    async fn test_update_heartbeat_updates_timestamp() {
        let manager = DeviceManager::new();
        manager.register_device("SN001", "client-1", "info1", "1.0").await;

        let device_before = manager.get_device("SN001").await.unwrap();
        let heartbeat_before = device_before.last_heartbeat.clone();

        // Small delay to ensure timestamp changes
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        manager.update_heartbeat("SN001").await;

        let device_after = manager.get_device("SN001").await.unwrap();
        assert!(device_after.last_heartbeat >= heartbeat_before);
    }

    #[tokio::test]
    async fn test_update_heartbeat_nonexistent_device() {
        let manager = DeviceManager::new();
        // Should not panic
        manager.update_heartbeat("NONEXISTENT").await;
    }

    #[tokio::test]
    async fn test_register_device_connected_at_set() {
        let manager = DeviceManager::new();
        manager.register_device("SN001", "client-1", "info1", "1.0").await;

        let device = manager.get_device("SN001").await.unwrap();
        assert!(!device.connected_at.is_empty());
        assert_eq!(device.connected_at, device.last_heartbeat);
    }

    #[tokio::test]
    async fn test_register_handler() {
        let manager = DeviceManager::new();
        let called = Arc::new(tokio::sync::Mutex::new(false));
        let called_clone = called.clone();

        let handler: DeviceHandler = Arc::new(move |_envelope| {
            let c = called_clone.clone();
            tokio::spawn(async move {
                let mut val = c.lock().await;
                *val = true;
            });
        });

        manager.register_handler("TestCommand", handler).await;

        let envelope = MqttEnvelope {
            command: "TestCommand".to_string(),
            request_id: "req-1".to_string(),
            time: "2026-06-18T10:00:00Z".to_string(),
            version: "1.0.1".to_string(),
            sn: "SN001".to_string(),
            data: "{}".to_string(),
        };

        manager.dispatch(envelope).await;

        // Give the handler time to execute
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        assert!(*called.lock().await);
    }

    #[tokio::test]
    async fn test_dispatch_unknown_command() {
        let manager = DeviceManager::new();
        let envelope = MqttEnvelope {
            command: "UnknownCommand".to_string(),
            request_id: "req-1".to_string(),
            time: "2026-06-18T10:00:00Z".to_string(),
            version: "1.0.1".to_string(),
            sn: "SN001".to_string(),
            data: "{}".to_string(),
        };

        // Should not panic
        manager.dispatch(envelope).await;
    }
}
