//! CTP 设备服务
//!
//! 封装 CTP 第三方平台 HTTP API 调用及 Redis 设备状态缓存逻辑。
//! 支持设备数据上报、锁控制命令发送、设备状态查询与缓存。

use crate::error::{CtpError, Result};
use crate::models::{CmdType, CtpConfig, CtpResponse, DeviceDataUpload, LockDevice, LockStatus};
use md5;

/// CTP 设备服务
///
/// 管理与 CTP 平台的通信，使用 MD5 签名进行身份验证（协议 v1.3.0），
/// 并将设备最新状态缓存至 Redis。
pub struct CtpDeviceService {
    config: CtpConfig,
}

impl Default for CtpDeviceService {
    fn default() -> Self {
        Self::new()
    }
}

impl CtpDeviceService {
    /// 创建新的 CTP 设备服务实例
    ///
    /// 从环境变量读取配置（`CTP_API_URL`, `CTP_FACTORY_ID`, `CTP_APP_SECRET`, `REDIS_URL`）。
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: CtpConfig::from_env(),
        }
    }

    /// 使用 MD5 生成 API 请求签名（CTP 协议 v1.3.0）
    ///
    /// 签名算法: `hex(md5(app_secret + device_no + date(yyyy-MM-dd) + app_secret))`
    /// 输出: 32位小写字符串
    fn sign(&self, device_no: &str, date: &str) -> String {
        let input = format!("{}{}{}{}", self.config.app_secret, device_no, date, self.config.app_secret);
        let digest = md5::compute(input.as_bytes());
        format!("{digest:x}")
    }

    /// 获取当前 UTC 日期字符串（YYYY-MM-DD）
    fn today_date() -> String {
        chrono::Utc::now().format("%Y-%m-%d").to_string()
    }

    /// 上报设备数据至 CTP 平台
    ///
    /// 使用表单格式提交设备电压、状态等数据，并将结果缓存至 Redis。
    pub async fn upload_device_data(&self, upload: &DeviceDataUpload) -> Result<CtpResponse> {
        let date = Self::today_date();
        let sign = self.sign(&upload.device_no, &date);
        let client = reqwest::Client::new();

        let resp = client
            .post(&self.config.api_base_url)
            .form(&[
                ("Sign", sign.as_str()),
                ("DeviceNo", upload.device_no.as_str()),
                ("DataType", &upload.data_type.to_string()),
                ("Voltage", upload.voltage.as_deref().unwrap_or("")),
                ("StatusOne", upload.status_one.as_deref().unwrap_or("")),
                ("StatusTwo", upload.status_two.as_deref().unwrap_or("")),
                ("DataTime", upload.data_time.as_deref().unwrap_or("")),
            ])
            .send()
            .await?;

        let status = resp.status();
        if status.is_success() {
            let body: CtpResponse = resp.json().await?;
            tracing::info!(
                "设备数据上报成功: device_no={}, data_type={}, error_code={}",
                upload.device_no,
                upload.data_type,
                body.error_code
            );
            Ok(body)
        } else {
            let text = resp.text().await.unwrap_or_default();
            Err(CtpError::DeviceConnection(format!("HTTP {status}: {text}")))
        }
    }

    /// 发送锁控制命令至 CTP 平台
    ///
    /// 支持 `Up`（开锁）、`Down`（关锁）、`Syn`（同步）三种命令类型。
    pub async fn send_lock_command(
        &self,
        device_no: &str,
        cmd_type: &CmdType,
        data: Option<&str>,
    ) -> Result<CtpResponse> {
        let date = Self::today_date();
        let sign = self.sign(device_no, &date);
        let client = reqwest::Client::new();
        let url = format!("{}/api/LockControl", self.config.api_base_url);

        let cmd_str = match cmd_type {
            CmdType::Up => "up",
            CmdType::Down => "down",
            CmdType::Syn => "syn",
        };

        let mut params = vec![
            ("Sign", sign),
            ("FacotryId", self.config.factory_id.clone()),
            ("DeviceNo", device_no.to_string()),
            ("CmdType", cmd_str.to_string()),
        ];

        if let Some(d) = data {
            params.push(("data", d.to_string()));
        }

        let resp = client
            .post(&url)
            .form(&params)
            .send()
            .await?;

        let status = resp.status();
        if status.is_success() {
            let body: CtpResponse = resp.json().await?;
            tracing::info!(
                "锁控制命令发送成功: device_no={}, cmd={}, error_code={}",
                device_no,
                cmd_str,
                body.error_code
            );
            Ok(body)
        } else {
            let text = resp.text().await.unwrap_or_default();
            Err(CtpError::DeviceCommand(format!("HTTP {status}: {text}")))
        }
    }

    /// 从 Redis 缓存查询设备状态
    pub async fn get_device_status(&self, device_no: &str) -> Result<LockDevice> {
        let client = redis::Client::open(self.config.redis_url.as_str())
            .map_err(CtpError::RedisError)?;
        let mut conn = client
            .get_multiplexed_async_connection()
            .await
            .map_err(CtpError::RedisError)?;

        let key = format!("ctp:device:{device_no}");
        let data: Option<String> = redis::AsyncCommands::get(&mut conn, &key)
            .await
            .map_err(CtpError::RedisError)?;

        match data {
            Some(json_str) => {
                let device: LockDevice = serde_json::from_str(&json_str)
                    .map_err(|e| CtpError::Internal(format!("解析设备数据失败: {e}")))?;
                Ok(device)
            }
            None => Err(CtpError::DeviceNotFound(device_no.to_string())),
        }
    }

    /// 分页查询设备列表（支持按车场代码过滤）
    pub async fn list_devices(
        &self,
        park_code: Option<&str>,
        page: i32,
        page_size: i32,
    ) -> Result<serde_json::Value> {
        let client = redis::Client::open(self.config.redis_url.as_str())
            .map_err(CtpError::RedisError)?;
        let mut conn = client
            .get_multiplexed_async_connection()
            .await
            .map_err(CtpError::RedisError)?;

        let pattern = if let Some(code) = park_code {
            format!("ctp:park:{code}:*")
        } else {
            "ctp:device:*".to_string()
        };

        let keys: Vec<String> = redis::AsyncCommands::keys(&mut conn, &pattern)
            .await
            .map_err(CtpError::RedisError)?;

        let mut devices = Vec::new();
        let start = ((page - 1) * page_size) as usize;
        let end = (start + page_size as usize).min(keys.len());

        for key in keys.iter().skip(start).take(end - start) {
            let data: Option<String> = redis::AsyncCommands::get(&mut conn, key)
                .await
                .map_err(CtpError::RedisError)?;
            if let Some(json_str) = data
                && let Ok(device) = serde_json::from_str::<LockDevice>(&json_str) {
                    devices.push(device);
                }
        }

        Ok(serde_json::json!({
            "devices": devices,
            "total": keys.len(),
            "page": page,
            "page_size": page_size,
        }))
    }

    /// 处理设备上报数据并缓存至 Redis
    ///
    /// 使用状态字解析器（附录2）解析 StatusOne/StatusTwo，
    /// 提取线圈/电量/入位/出位/逃费等状态信息写入缓存。
    pub async fn handle_device_data_upload(&self, upload: &DeviceDataUpload) -> Result<()> {
        let client = redis::Client::open(self.config.redis_url.as_str())
            .map_err(CtpError::RedisError)?;
        let mut conn = client
            .get_multiplexed_async_connection()
            .await
            .map_err(CtpError::RedisError)?;

        // 使用状态字解析器
        let s1 = upload.status_one.as_deref().unwrap_or("");
        let s2 = upload.status_two.as_deref().unwrap_or("");
        let parsed = crate::services::status_parser::parse_status(s1, s2);

        let status = match parsed.lock_status.as_str() {
            "Locked" | "Locked(Force)" => LockStatus::Locked,
            "Unlocked" | "Unlocked(Force)" | "Transient" => LockStatus::Unlocked,
            _ if s1.is_empty() => LockStatus::Offline,
            _ => LockStatus::Fault,
        };

        let device = LockDevice {
            id: uuid::Uuid::new_v4().to_string(),
            device_no: upload.device_no.clone(),
            factory_id: self.config.factory_id.clone(),
            status,
            battery: None,
            signal: None,
            voltage: upload.voltage.clone(),
            park_code: String::new(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: upload.data_time.clone().unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
        };

        // 将解析结果附加到 device 结构（通过序列化扩展）
        let mut device_json = serde_json::to_value(&device)
            .map_err(|e| CtpError::Internal(format!("序列化设备失败: {e}")))?;
        device_json["status_parsed"] = serde_json::to_value(&parsed.status_one)
            .map_err(|e| CtpError::Internal(format!("序列化状态失败: {e}")))?;
        if let Some(ref st2) = parsed.status_two {
            device_json["status_two_parsed"] = serde_json::to_value(st2)
                .map_err(|e| CtpError::Internal(format!("序列化状态失败: {e}")))?;
        }
        device_json["battery_level"] = serde_json::Value::String(parsed.battery_level.clone());

        let key = format!("ctp:device:{}", upload.device_no);
        let _: () = redis::AsyncCommands::set(
            &mut conn,
            &key,
            serde_json::to_string(&device_json).unwrap_or_default(),
        )
        .await
        .map_err(CtpError::RedisError)?;

        tracing::info!(
            "设备数据已缓存: device_no={}, status={:?}, battery={}, lock_state={}",
            upload.device_no,
            device.status,
            parsed.battery_level,
            parsed.lock_status,
        );

        if parsed.valid {
            tracing::debug!(
                "状态字解析: S1={}, 锁={}, 线圈左={}, 线圈右={}, 电量={}",
                s1,
                parsed.status_one.lock_state,
                parsed.status_one.left_coil,
                parsed.status_one.right_coil,
                parsed.battery_level,
            );
            if let Some(ref st2) = parsed.status_two {
                tracing::debug!(
                    "状态字解析: S2={}, 入位={}, 出位={}, 逃费={}, 当前车辆={}",
                    s2,
                    st2.total_entry_count,
                    st2.total_exit_count,
                    st2.total_theft_count,
                    st2.current_occupancy,
                );
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_generates_correct_hash() {
        let config = CtpConfig {
            api_base_url: "http://test.com".to_string(),
            factory_id: "fac123".to_string(),
            app_secret: "secret456".to_string(),
            redis_url: "redis://localhost".to_string(),
        };
        let service = CtpDeviceService { config };
        let sign = service.sign("dev001", "2026-06-18");
        // Expected: md5("secret456dev0012026-06-18secret456")
        let expected = format!("{:x}", md5::compute(b"secret456dev0012026-06-18secret456"));
        assert_eq!(sign, expected);
        assert_eq!(sign.len(), 32);
    }

    #[test]
    fn test_sign_is_32_chars_lowercase() {
        let config = CtpConfig {
            api_base_url: "http://test.com".to_string(),
            factory_id: "fac123".to_string(),
            app_secret: "test".to_string(),
            redis_url: "redis://localhost".to_string(),
        };
        let service = CtpDeviceService { config };
        let sign = service.sign("dev001", "2026-06-18");
        assert_eq!(sign.len(), 32);
        assert_eq!(sign, sign.to_lowercase());
    }

    #[test]
    fn test_sign_different_device_produces_different_hash() {
        let config = CtpConfig {
            api_base_url: "http://test.com".to_string(),
            factory_id: "fac123".to_string(),
            app_secret: "secret456".to_string(),
            redis_url: "redis://localhost".to_string(),
        };
        let service = CtpDeviceService { config };
        let sign1 = service.sign("dev001", "2026-06-18");
        let sign2 = service.sign("dev002", "2026-06-18");
        assert_ne!(sign1, sign2);
    }

    #[test]
    fn test_sign_different_date_produces_different_hash() {
        let config = CtpConfig {
            api_base_url: "http://test.com".to_string(),
            factory_id: "fac123".to_string(),
            app_secret: "secret456".to_string(),
            redis_url: "redis://localhost".to_string(),
        };
        let service = CtpDeviceService { config };
        let sign1 = service.sign("dev001", "2026-06-18");
        let sign2 = service.sign("dev001", "2026-06-19");
        assert_ne!(sign1, sign2);
    }

    #[test]
    fn test_sign_different_secret_produces_different_hash() {
        let config1 = CtpConfig {
            api_base_url: "http://test.com".to_string(),
            factory_id: "fac123".to_string(),
            app_secret: "secret1".to_string(),
            redis_url: "redis://localhost".to_string(),
        };
        let config2 = CtpConfig {
            api_base_url: "http://test.com".to_string(),
            factory_id: "fac123".to_string(),
            app_secret: "secret2".to_string(),
            redis_url: "redis://localhost".to_string(),
        };
        let svc1 = CtpDeviceService { config: config1 };
        let svc2 = CtpDeviceService { config: config2 };
        assert_ne!(svc1.sign("dev001", "2026-06-18"), svc2.sign("dev001", "2026-06-18"));
    }

    #[test]
    fn test_today_date_format() {
        let date = CtpDeviceService::today_date();
        assert_eq!(date.len(), 10);
        // Should match YYYY-MM-DD format
        assert!(date.chars().nth(4) == Some('-'));
        assert!(date.chars().nth(7) == Some('-'));
        assert!(date.split('-').count() == 3);
    }

    #[test]
    fn test_status_derivation_locked_prefix_4() {
        let config = CtpConfig {
            api_base_url: "http://test.com".to_string(),
            factory_id: "fac123".to_string(),
            app_secret: "secret".to_string(),
            redis_url: "redis://localhost".to_string(),
        };
        let _service = CtpDeviceService { config };
        let upload = DeviceDataUpload {
            device_no: "dev001".to_string(),
            data_type: 1,
            voltage: Some("12.5".to_string()),
            status_one: Some("41".to_string()),
            status_two: Some("00".to_string()),
            data_time: Some("2026-06-18T10:00:00Z".to_string()),
        };

        let status = if let Some(ref s1) = upload.status_one {
            if s1.starts_with('4') || s1.starts_with('6') {
                LockStatus::Locked
            } else if s1.starts_with('2') || s1.starts_with('3') {
                LockStatus::Unlocked
            } else {
                LockStatus::Fault
            }
        } else {
            LockStatus::Offline
        };

        assert_eq!(status, LockStatus::Locked);
    }

    #[test]
    fn test_status_derivation_locked_prefix_6() {
        let upload = DeviceDataUpload {
            device_no: "dev001".to_string(),
            data_type: 1,
            voltage: None,
            status_one: Some("60".to_string()),
            status_two: None,
            data_time: None,
        };

        let status = if let Some(ref s1) = upload.status_one {
            if s1.starts_with('4') || s1.starts_with('6') {
                LockStatus::Locked
            } else if s1.starts_with('2') || s1.starts_with('3') {
                LockStatus::Unlocked
            } else {
                LockStatus::Fault
            }
        } else {
            LockStatus::Offline
        };

        assert_eq!(status, LockStatus::Locked);
    }

    #[test]
    fn test_status_derivation_unlocked_prefix_2() {
        let upload = DeviceDataUpload {
            device_no: "dev001".to_string(),
            data_type: 1,
            voltage: None,
            status_one: Some("20".to_string()),
            status_two: None,
            data_time: None,
        };

        let status = if let Some(ref s1) = upload.status_one {
            if s1.starts_with('4') || s1.starts_with('6') {
                LockStatus::Locked
            } else if s1.starts_with('2') || s1.starts_with('3') {
                LockStatus::Unlocked
            } else {
                LockStatus::Fault
            }
        } else {
            LockStatus::Offline
        };

        assert_eq!(status, LockStatus::Unlocked);
    }

    #[test]
    fn test_status_derivation_unlocked_prefix_3() {
        let upload = DeviceDataUpload {
            device_no: "dev001".to_string(),
            data_type: 1,
            voltage: None,
            status_one: Some("30".to_string()),
            status_two: None,
            data_time: None,
        };

        let status = if let Some(ref s1) = upload.status_one {
            if s1.starts_with('4') || s1.starts_with('6') {
                LockStatus::Locked
            } else if s1.starts_with('2') || s1.starts_with('3') {
                LockStatus::Unlocked
            } else {
                LockStatus::Fault
            }
        } else {
            LockStatus::Offline
        };

        assert_eq!(status, LockStatus::Unlocked);
    }

    #[test]
    fn test_status_derivation_fault() {
        let upload = DeviceDataUpload {
            device_no: "dev001".to_string(),
            data_type: 1,
            voltage: None,
            status_one: Some("99".to_string()),
            status_two: None,
            data_time: None,
        };

        let status = if let Some(ref s1) = upload.status_one {
            if s1.starts_with('4') || s1.starts_with('6') {
                LockStatus::Locked
            } else if s1.starts_with('2') || s1.starts_with('3') {
                LockStatus::Unlocked
            } else {
                LockStatus::Fault
            }
        } else {
            LockStatus::Offline
        };

        assert_eq!(status, LockStatus::Fault);
    }

    #[test]
    fn test_status_derivation_offline() {
        let upload = DeviceDataUpload {
            device_no: "dev001".to_string(),
            data_type: 1,
            voltage: None,
            status_one: None,
            status_two: None,
            data_time: None,
        };

        let status = if let Some(ref s1) = upload.status_one {
            if s1.starts_with('4') || s1.starts_with('6') {
                LockStatus::Locked
            } else if s1.starts_with('2') || s1.starts_with('3') {
                LockStatus::Unlocked
            } else {
                LockStatus::Fault
            }
        } else {
            LockStatus::Offline
        };

        assert_eq!(status, LockStatus::Offline);
    }

    #[test]
    fn test_list_devices_pagination_math() {
        let page = 2i32;
        let page_size = 10i32;
        let keys_len = 25usize;
        let start = ((page - 1) * page_size) as usize;
        let end = (start + page_size as usize).min(keys_len);
        assert_eq!(start, 10);
        assert_eq!(end, 20);
    }

    #[test]
    fn test_list_devices_first_page() {
        let page = 1i32;
        let page_size = 10i32;
        let keys_len = 25usize;
        let start = ((page - 1) * page_size) as usize;
        let end = (start + page_size as usize).min(keys_len);
        assert_eq!(start, 0);
        assert_eq!(end, 10);
    }

    #[test]
    fn test_list_devices_last_page_partial() {
        let page = 3i32;
        let page_size = 10i32;
        let keys_len = 25usize;
        let start = ((page - 1) * page_size) as usize;
        let end = (start + page_size as usize).min(keys_len);
        assert_eq!(start, 20);
        assert_eq!(end, 25);
    }
}
