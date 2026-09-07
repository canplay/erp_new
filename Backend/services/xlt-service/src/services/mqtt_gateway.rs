//! MQTT 网关
//!
//! 通过 EMQX HTTP API 桥接，实现与信路通停车设备的 MQTT 通信。
//! 支持开闸、关闸、长开、快照、配置下发等设备命令。

use crate::error::{Result, XltError};
use crate::models::{
    MqttConfig, MqttEnvelope, ReplyData,
};
use serde::Serialize;

/// MQTT 网关
pub struct MqttGateway {
    config: MqttConfig,
    http_client: reqwest::Client,
}

impl MqttGateway {
    #[must_use]
    pub fn new(config: MqttConfig) -> Self {
        Self {
            config,
            http_client: reqwest::Client::new(),
        }
    }

    #[must_use]
    pub fn config(&self) -> &MqttConfig {
        &self.config
    }

    /// 通过 EMQX HTTP API 发布 MQTT 消息到设备
    pub async fn publish(&self, topic: &str, payload: &str, qos: i32) -> Result<()> {
        let url = format!("http://{}:{}/api/v5/publish", self.config.host, self.config.http_port);

        let body = serde_json::json!({
            "topic": topic,
            "payload": payload,
            "qos": qos,
            "retain": false,
            "clientid": self.config.client_id,
        });

        let resp = self
            .http_client
            .post(&url)
            .basic_auth(&self.config.username, Some(&self.config.password))
            .json(&body)
            .send()
            .await?;

        if resp.status().is_success() {
            tracing::info!("MQTT发布成功: topic={topic}, qos={qos}");
        } else {
            tracing::warn!("MQTT发布失败: HTTP {}, topic={}", resp.status(), topic);
        }

        Ok(())
    }

    async fn send_device_command(&self, envelope: &MqttEnvelope) -> Result<()> {
        let topic = format!("download/{}", envelope.sn);
        let payload = serde_json::to_string(envelope)
            .map_err(|e| XltError::Internal(format!("序列化失败: {e}")))?;
        self.publish(&topic, &payload, 1).await
    }

    pub async fn send_reply(&self, sn: &str, request_id: &str, command: &str, err_code: i32, err_info: &str) -> Result<()> {
        let data = ReplyData {
            err_code,
            err_info: err_info.to_string(),
        };
        let envelope = MqttEnvelope {
            command: command.to_string(),
            request_id: request_id.to_string(),
            time: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            version: "1.0.1".to_string(),
            sn: sn.to_string(),
            data: serde_json::to_string(&data).unwrap_or_default(),
        };
        self.send_device_command(&envelope).await
    }

    pub async fn send_open(&self, sn: &str, request_id: &str) -> Result<()> {
        let envelope = MqttEnvelope {
            command: "Open".to_string(),
            request_id: request_id.to_string(),
            time: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            version: "1.0.1".to_string(),
            sn: sn.to_string(),
            data: String::new(),
        };
        self.send_device_command(&envelope).await
    }

    pub async fn send_close(&self, sn: &str, request_id: &str) -> Result<()> {
        let envelope = MqttEnvelope {
            command: "Close".to_string(),
            request_id: request_id.to_string(),
            time: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            version: "1.0.1".to_string(),
            sn: sn.to_string(),
            data: String::new(),
        };
        self.send_device_command(&envelope).await
    }

    pub async fn send_long_open(&self, sn: &str, request_id: &str, switch_type: &str) -> Result<()> {
        let data = serde_json::json!({"switchType": switch_type});
        let envelope = MqttEnvelope {
            command: "LongOpen".to_string(),
            request_id: request_id.to_string(),
            time: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            version: "1.0.1".to_string(),
            sn: sn.to_string(),
            data: data.to_string(),
        };
        self.send_device_command(&envelope).await
    }

    pub async fn send_enable_update(&self, sn: &str, request_id: &str, passruler: i32, carinfo: i32) -> Result<()> {
        let data = serde_json::json!({"passruler": passruler, "carinfo": carinfo});
        let envelope = MqttEnvelope {
            command: "EnableUpdate".to_string(),
            request_id: request_id.to_string(),
            time: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            version: "1.0.1".to_string(),
            sn: sn.to_string(),
            data: data.to_string(),
        };
        self.send_device_command(&envelope).await
    }

    pub async fn send_snapshot(&self, sn: &str, request_id: &str, snapshot_id: &str) -> Result<()> {
        let data = serde_json::json!({"snapshotId": snapshot_id});
        let envelope = MqttEnvelope {
            command: "SnapshotPic".to_string(),
            request_id: request_id.to_string(),
            time: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            version: "1.0.1".to_string(),
            sn: sn.to_string(),
            data: data.to_string(),
        };
        self.send_device_command(&envelope).await
    }

    pub async fn send_reset(&self, sn: &str, request_id: &str) -> Result<()> {
        let envelope = MqttEnvelope {
            command: "ResetDevice".to_string(),
            request_id: request_id.to_string(),
            time: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            version: "1.0.1".to_string(),
            sn: sn.to_string(),
            data: String::new(),
        };
        self.send_device_command(&envelope).await
    }

    pub async fn send_image_config(&self, sn: &str, request_id: &str, config: &crate::models::ImageConfig) -> Result<()> {
        let data = serde_json::to_string(config)
            .map_err(|e| XltError::Internal(format!("序列化失败: {e}")))?;
        let envelope = MqttEnvelope {
            command: "Image".to_string(),
            request_id: request_id.to_string(),
            time: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            version: "1.0.1".to_string(),
            sn: sn.to_string(),
            data,
        };
        self.send_device_command(&envelope).await
    }

    pub async fn send_dl_pass_rule<T: Serialize>(&self, sn: &str, request_id: &str, rule: &T) -> Result<()> {
        let data = serde_json::to_string(rule)
            .map_err(|e| XltError::Internal(format!("序列化失败: {e}")))?;
        let envelope = MqttEnvelope {
            command: "DlPassRule".to_string(),
            request_id: request_id.to_string(),
            time: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            version: "1.0.1".to_string(),
            sn: sn.to_string(),
            data,
        };
        self.send_device_command(&envelope).await
    }

    pub async fn send_dl_car_info(&self, sn: &str, request_id: &str, url: &str) -> Result<()> {
        let data = serde_json::json!({"url": url});
        let envelope = MqttEnvelope {
            command: "DlCarInfo".to_string(),
            request_id: request_id.to_string(),
            time: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            version: "1.0.1".to_string(),
            sn: sn.to_string(),
            data: data.to_string(),
        };
        self.send_device_command(&envelope).await
    }

    pub async fn send_encrypt_device(&self, sn: &str, request_id: &str, key: &str, enable: i32) -> Result<()> {
        let data = serde_json::json!({"key": key, "enable": enable});
        let envelope = MqttEnvelope {
            command: "EncryptDevice".to_string(),
            request_id: request_id.to_string(),
            time: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            version: "1.0.1".to_string(),
            sn: sn.to_string(),
            data: data.to_string(),
        };
        self.send_device_command(&envelope).await
    }

    pub async fn send_config(&self, sn: &str, request_id: &str, config: &crate::models::ConfigData) -> Result<()> {
        let data = serde_json::to_string(config)
            .map_err(|e| XltError::Internal(format!("序列化失败: {e}")))?;
        let envelope = MqttEnvelope {
            command: "Config".to_string(),
            request_id: request_id.to_string(),
            time: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            version: "1.0.1".to_string(),
            sn: sn.to_string(),
            data,
        };
        self.send_device_command(&envelope).await
    }

    pub async fn send_clear_data(&self, sn: &str, request_id: &str, clear: &crate::models::ClearDataMsg) -> Result<()> {
        let data = serde_json::to_string(clear)
            .map_err(|e| XltError::Internal(format!("序列化失败: {e}")))?;
        let envelope = MqttEnvelope {
            command: "ClearData".to_string(),
            request_id: request_id.to_string(),
            time: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            version: "1.0.1".to_string(),
            sn: sn.to_string(),
            data,
        };
        self.send_device_command(&envelope).await
    }

    pub async fn send_serial_config(&self, sn: &str, request_id: &str, sc: &crate::models::SerialConfigData) -> Result<()> {
        let data = serde_json::to_string(sc)
            .map_err(|e| XltError::Internal(format!("序列化失败: {e}")))?;
        let envelope = MqttEnvelope {
            command: "SerialConfig".to_string(),
            request_id: request_id.to_string(),
            time: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            version: "1.0.1".to_string(),
            sn: sn.to_string(),
            data,
        };
        self.send_device_command(&envelope).await
    }

    pub async fn send_serial_data(&self, sn: &str, request_id: &str, sd: &crate::models::SerialDataMsg) -> Result<()> {
        let data = serde_json::to_string(sd)
            .map_err(|e| XltError::Internal(format!("序列化失败: {e}")))?;
        let envelope = MqttEnvelope {
            command: "SerialData".to_string(),
            request_id: request_id.to_string(),
            time: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            version: "1.0.1".to_string(),
            sn: sn.to_string(),
            data,
        };
        self.send_device_command(&envelope).await
    }

    pub async fn send_enable_reply(&self, sn: &str, request_id: &str, enable: i32) -> Result<()> {
        let data = serde_json::json!({"enable": enable});
        let envelope = MqttEnvelope {
            command: "EnableReply".to_string(),
            request_id: request_id.to_string(),
            time: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            version: "1.0.1".to_string(),
            sn: sn.to_string(),
            data: data.to_string(),
        };
        self.send_device_command(&envelope).await
    }

    /// 下发显示屏配置 (`SetLCDItems`)
    ///
    /// 远程配置设备显示屏的显示内容、语音播报和开闸动作。
    /// 对应信路通协议 4.4.3 `SetLCDItems`。
    pub async fn send_set_lcd_items(&self, sn: &str, request_id: &str, template: i32, items: &str, voice: &str, action: i32) -> Result<()> {
        let data = serde_json::json!({
            "cmdName": "SetLCDItems",
            "template": template,
            "adID": 0,
            "items": items,
            "voice": voice,
            "action": action,
        });
        let envelope = MqttEnvelope {
            command: "SetLCDItems".to_string(),
            request_id: request_id.to_string(),
            time: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            version: "1.0.1".to_string(),
            sn: sn.to_string(),
            data: data.to_string(),
        };
        self.send_device_command(&envelope).await
    }

    /// 添加固定车辆信息 (`AddCarInfo`)
    pub async fn send_add_car_info(&self, sn: &str, request_id: &str, plate: &str, car_type: i32, valid_time: &str) -> Result<()> {
        let data = serde_json::json!({
            "carInfoList": [{"plate": plate, "type": car_type, "time": valid_time}]
        });
        let envelope = MqttEnvelope {
            command: "AddCarInfo".to_string(),
            request_id: request_id.to_string(),
            time: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            version: "1.0.1".to_string(),
            sn: sn.to_string(),
            data: data.to_string(),
        };
        self.send_device_command(&envelope).await
    }

    /// 删除固定车辆信息 (`DeleteCarInfo`)
    pub async fn send_delete_car_info(&self, sn: &str, request_id: &str, plates: &[String]) -> Result<()> {
        let data = serde_json::json!({"plateList": plates});
        let envelope = MqttEnvelope {
            command: "DeleteCarInfo".to_string(),
            request_id: request_id.to_string(),
            time: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            version: "1.0.1".to_string(),
            sn: sn.to_string(),
            data: data.to_string(),
        };
        self.send_device_command(&envelope).await
    }

    /// 查询固定车辆信息 (`QueryCarInfo`)
    pub async fn send_query_car_info(&self, sn: &str, request_id: &str, query_id: &str, plate: &str) -> Result<()> {
        let data = serde_json::json!({"queryId": query_id, "plate": plate});
        let envelope = MqttEnvelope {
            command: "QueryCarInfo".to_string(),
            request_id: request_id.to_string(),
            time: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            version: "1.0.1".to_string(),
            sn: sn.to_string(),
            data: data.to_string(),
        };
        self.send_device_command(&envelope).await
    }
}
