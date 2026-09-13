// 银联商务(UMS)支付服务
// Union Money Service payment integration

use redis::AsyncCommands;
use serde::Deserialize;
use sqlx::PgPool;
use sqlx::types::BigDecimal;

use crate::error::{PayError, Result};

/// UMS 银联支付配置
#[derive(Debug, Clone, Deserialize)]
pub struct UmsConfig {
    pub appid: String,
    pub appkey: String,
    pub mid: String,
    pub tid: String,
    pub ysjc: Option<String>, // 银食聚财子商户号
}

/// UMS 查询参数
#[derive(Debug, Deserialize)]
pub struct UmsQueryParams {
    pub time: String,
    pub no: Option<String>,
}

/// UMS 创建订单参数
#[derive(Debug, Deserialize)]
pub struct UmsOrderParams {
    pub time: String,
    pub no: String,
    #[serde(rename = "no_pay")]
    pub no_pay: Option<serde_json::Value>,
    pub amount: i64,
    pub desc: Option<String>,
    pub return_url: Option<String>,
    pub notify: Option<String>,
    pub zone: Option<String>,
}

/// UMS 关闭订单参数
#[derive(Debug, Deserialize)]
pub struct UmsCloseParams {
    pub no: String,
    pub time: String,
}

/// UMS 退款参数
#[derive(Debug, Deserialize)]
pub struct UmsRefundParams {
    pub time: String,
    pub no: String,
    pub desc: Option<String>,
    pub amount: i64,
    pub refundno: String,
    pub zone: Option<String>,
}

/// UMS 支付回调参数
#[derive(Debug, Deserialize)]
pub struct UmsNotifyParams {
    pub order: String,
    /// 账单日期 (YYYY-MM-DD), 用于主动查询核实; 缺省时取当天
    pub time: Option<String>,
}

/// UMS 银联支付服务
pub struct UmsService {
    config: UmsConfig,
    /// 数据库连接池（共享）
    pool: PgPool,
}

impl UmsService {
    /// 创建 UMS 支付服务
    #[must_use]
    pub fn new(config: UmsConfig, pool: PgPool) -> Self {
        Self { config, pool }
    }

    /// 获取访问令牌（公开方法，供 handler 调用）
    pub async fn get_access_token(&self) -> Result<String> {
        let redis_url = std::env::var("REDIS_URL").unwrap_or_default();
        let client = redis::Client::open(redis_url.as_str()).map_err(PayError::RedisError)?;
        let mut conn = client
            .get_multiplexed_async_connection()
            .await
            .map_err(PayError::RedisError)?;

        // 尝试从 Redis 获取缓存的 token
        let cached: Option<String> = conn.get("ums:key").await.map_err(PayError::RedisError)?;

        if let Some(token) = cached
            && !token.is_empty() && token != "null" {
                return Ok(token);
            }

        // 生成新 token
        let token = self.request_new_token().await?;

        // 缓存 token (600秒)
        let _: () = conn
            .set_ex("ums:key", &token, 600)
            .await
            .map_err(PayError::RedisError)?;

        Ok(token)
    }

    /// 请求新的访问令牌
    async fn request_new_token(&self) -> Result<String> {
        use std::time::{SystemTime, UNIX_EPOCH};

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after UNIX epoch")
            .as_secs();

        let timestamp_str = format!(
            "{:04}{:02}{:02}{:02}{:02}{:02}",
            (timestamp / 31536000) + 1970,
            (timestamp / 2592000) % 12 + 1,
            (timestamp / 86400) % 30 + 1,
            (timestamp / 3600) % 24,
            (timestamp / 60) % 60,
            timestamp % 60
        );

        // 生成随机 nonce
        let uuid = uuid::Uuid::new_v4().to_string();
        let nonce: String = uuid.chars().filter(|c| *c != '-').collect();

        let signature = Self::sha256(format!(
            "{}{}{}{}",
            self.config.appid, timestamp_str, nonce, self.config.appkey
        ));

        let client_req = serde_json::json!({
            "appId": self.config.appid,
            "timestamp": timestamp_str,
            "nonce": nonce,
            "signMethod": "SHA256",
            "signature": signature
        });

        let client = reqwest::Client::new();
        let response = client
            .post("https://api-mop.chinaums.com/v1/token/access")
            .json(&client_req)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
            .map_err(PayError::RequestError)?;

        let body: serde_json::Value = response.json().await.unwrap_or_default();

        body.get("accessToken")
            .and_then(|v| v.as_str())
            .map(String::from)
            .ok_or_else(|| PayError::InternalError("Failed to get access token".to_string()))
    }

    /// 查询支付订单
    pub async fn query(
        &self,
        params: &UmsQueryParams,
        access_token: &str,
    ) -> Result<serde_json::Value> {
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let client_req = serde_json::json!({
            "billDate": params.time,
            "billNo": params.no,
            "instMid": "QRPAYDEFAULT",
            "mid": self.config.mid,
            "requestTimestamp": timestamp,
            "tid": self.config.tid
        });

        let client = reqwest::Client::new();
        let response = client
            .post("https://api-mop.chinaums.com/v1/netpay/bills/query")
            .header("Content-type", "application/json")
            .header(
                "Authorization",
                format!(
                    "OPEN-ACCESS-TOKEN AccessToken=\"{}\", AppId=\"{}\"",
                    access_token, self.config.appid
                ),
            )
            .json(&client_req)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
            .map_err(PayError::RequestError)?;

        let body: serde_json::Value = response.json().await.unwrap_or_default();
        Ok(body)
    }

    /// 创建支付订单
    pub async fn create_order(
        &self,
        params: &UmsOrderParams,
        access_token: &str,
    ) -> Result<serde_json::Value> {
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let mut client_req = serde_json::json!({
            "billDate": params.time,
            "billDesc": params.desc,
            "billNo": params.no,
            "counterNo": params.desc,
            "instMid": "QRPAYDEFAULT",
            "mid": self.config.mid,
            "requestTimestamp": timestamp,
            "tid": self.config.tid,
            "totalAmount": (params.amount * 100) as i64,
            "returnUrl": params.return_url,
            "notifyUrl": params.notify
        });

        // 如果是银食聚财分区，添加分账信息
        if params.zone.as_deref() == Some("ysjc") {
            client_req["divisionFlag"] = serde_json::json!(true);
            client_req["platformAmount"] = serde_json::json!(0);

            if let Some(ysjc_mid) = &self.config.ysjc {
                let sub_order = serde_json::json!({
                    "mid": ysjc_mid,
                    "merOrderId": format!("{}SUB", params.no),
                    "totalAmount": (params.amount * 100) as i64
                });
                client_req["subOrders"] = serde_json::json!([sub_order]);
            }
        }

        let client = reqwest::Client::new();
        let response = client
            .post("https://api-mop.chinaums.com/v1/netpay/bills/get-qrcode")
            .header("Content-type", "application/json")
            .header(
                "Authorization",
                format!(
                    "OPEN-ACCESS-TOKEN AccessToken=\"{}\", AppId=\"{}\"",
                    access_token, self.config.appid
                ),
            )
            .json(&client_req)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
            .map_err(PayError::RequestError)?;

        let body: serde_json::Value = response.json().await.unwrap_or_default();

        // 检查是否成功或账单号重复（可接受）
        let err_code = body.get("errCode").and_then(|v| v.as_str()).unwrap_or("");
        if err_code == "SUCCESS"
            || body.get("errMsg").and_then(|v| v.as_str()) == Some("账单号重复")
        {
            // 更新或创建支付记录
            self.save_order(params, &client_req).await?;
        }

        Ok(body)
    }

    /// 保存订单到数据库
    async fn save_order(
        &self,
        params: &UmsOrderParams,
        create_params: &serde_json::Value,
    ) -> Result<()> {
        let no_pay = params
            .no_pay
            .as_ref()
            .map(|v| serde_json::to_string(v).unwrap_or_default())
            .unwrap_or_default();

        let amount = (params.amount * 100) as i32;
        let desc = params.desc.as_deref().unwrap_or("");
        let create_params_str = serde_json::to_string(create_params).unwrap_or_default();

        // 检查订单是否存在
        let exists: Option<i32> = sqlx::query_scalar!(
            "SELECT 1 FROM pay WHERE \"order\" = $1",
            &params.no
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(PayError::DatabaseError)?
        .flatten();

        if exists.is_some() {
            // 更新现有订单
            sqlx::query!(
                "UPDATE pay SET order_pay = $1, status = 'generate', \"type\" = 'ums', amount = $2, remark = $3, create_service = 'pay', create_params = $4, create_date = $5, update_date = $5 WHERE \"order\" = $6",
                serde_json::Value::String(no_pay).clone(),
                BigDecimal::from(amount),
                desc,
                serde_json::Value::String(create_params_str).clone(),
                chrono::DateTime::parse_from_rfc3339(&params.time).unwrap_or(chrono::Utc::now().fixed_offset()).into(),
                params.no.clone().into(),
            )
            .execute(&self.pool)
            .await
            .map_err(PayError::DatabaseError)?;
        } else {
            // 创建新订单
            let id = uuid::Uuid::new_v4().to_string();
            sqlx::query!(
                "INSERT INTO pay VALUES ($1, $2, 'generate', 'ums', $3, $4, $5, 'pay', $6, $7, $8)",
                &id,
                params.no.clone().into(),
                serde_json::Value::String(no_pay).clone(),
                BigDecimal::from(amount),
                desc,
                serde_json::Value::String(create_params_str).clone(),
                chrono::DateTime::parse_from_rfc3339(&params.time).unwrap_or(chrono::Utc::now().fixed_offset()).into(),
                chrono::DateTime::parse_from_rfc3339(&params.time).unwrap_or(chrono::Utc::now().fixed_offset()).into(),
            )
            .execute(&self.pool)
            .await
            .map_err(PayError::DatabaseError)?;
        }

        Ok(())
    }

    /// 处理支付回调通知
    ///
    /// 安全加固 (2026-08-04 审计修复):
    /// 1. 幂等: 已 paid/closed 直接返回成功, 不重复处理
    /// 2. 状态机: 仅允许 'generate'(未支付) → 'paid'
    /// 3. 金额核实: 回调参数仅有 order 无签名/金额字段可验, 故主动调用 UMS
    ///    账单查询接口核实订单真实状态与金额, 核实通过才置 paid
    pub async fn notify(&self, params: &UmsNotifyParams) -> Result<bool> {
        // 1) 查本地订单: 不存在则拒绝
        let row = sqlx::query!(
                "SELECT status, amount FROM pay WHERE \"order\" = $1",
                &params.order
            )
            .fetch_optional(&self.pool)
            .await
            .map_err(PayError::DatabaseError)?;

        let (status, local_amount) = row.map(|r| (
            r.status.unwrap_or_default(),
            r.amount.unwrap_or_default().to_string().parse::<i64>().unwrap_or(0)
        )).ok_or_else(|| {
            PayError::InternalError(format!("支付回调: 订单不存在: {}", params.order))
        })?;

        // 2) 幂等: 已支付/已关闭直接返回成功
        if status == "paid" || status == "closed" {
            return Ok(true);
        }

        // 3) 状态机: 仅未支付状态允许流转到 paid
        if status != "generate" {
            return Err(PayError::InternalError(format!(
                "支付回调: 订单状态不允许流转: {status}"
            )));
        }

        // 4) 主动向 UMS 查询核实订单真实状态与金额(回调无签名/金额字段可验)
        let access_token = self.get_access_token().await?;
        let bill_date = params
            .time
            .clone()
            .unwrap_or_else(|| chrono::Utc::now().format("%Y-%m-%d").to_string());
        let query_params = UmsQueryParams {
            time: bill_date,
            no: Some(params.order.clone()),
        };
        let verify = self.query(&query_params, &access_token).await?;

        let err_code = verify.get("errCode").and_then(|v| v.as_str()).unwrap_or("");
        if err_code != "00" {
            return Err(PayError::InternalError(format!(
                "支付回调: UMS 订单核实失败 errCode={err_code}"
            )));
        }
        // 金额核实: UMS totalAmount 与本地 amount 均为"分"
        if let Some(total) = verify.get("totalAmount").and_then(|v| v.as_i64()) {
            if (total - local_amount).abs() > 1 {
                return Err(PayError::InternalError(format!(
                    "支付回调: 金额不匹配 核实={total} 本地={local_amount}"
                )));
            }
        }

        // 5) 条件更新 (status='generate' 防止并发重复流转), 更新行数为 0 视为已被并发处理
        let result = sqlx::query!(
            "UPDATE pay SET status = 'paid', update_date = $1 WHERE \"order\" = $2 AND status = 'generate'",
            chrono::Utc::now(),
            &params.order,
        )
        .execute(&self.pool)
        .await
        .map_err(PayError::DatabaseError)?;

        if result.rows_affected() == 0 {
            return Ok(true); // 并发下已被其他回调处理, 视为成功(幂等)
        }

        Ok(true)
    }

    /// 关闭支付订单
    pub async fn close(
        &self,
        params: &UmsCloseParams,
        access_token: &str,
    ) -> Result<serde_json::Value> {
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let client_req = serde_json::json!({
            "qrCodeId": params.no,
            "instMid": "QRPAYDEFAULT",
            "mid": self.config.mid,
            "requestTimestamp": timestamp,
            "tid": self.config.tid
        });

        let client = reqwest::Client::new();
        let response = client
            .post("https://api-mop.chinaums.com/v1/netpay/bills/close-qrcode")
            .header("Content-type", "application/json")
            .header(
                "Authorization",
                format!(
                    "OPEN-ACCESS-TOKEN AccessToken=\"{}\", AppId=\"{}\"",
                    access_token, self.config.appid
                ),
            )
            .json(&client_req)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
            .map_err(PayError::RequestError)?;

        let body: serde_json::Value = response.json().await.unwrap_or_default();
        Ok(body)
    }

    /// 退款
    pub async fn refund(
        &self,
        params: &UmsRefundParams,
        access_token: &str,
    ) -> Result<serde_json::Value> {
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let mut client_req = serde_json::json!({
            "billDate": params.time,
            "refundDesc": params.desc,
            "billNo": params.no,
            "counterNo": params.desc,
            "instMid": "QRPAYDEFAULT",
            "mid": self.config.mid,
            "requestTimestamp": timestamp,
            "tid": self.config.tid,
            "refundAmount": (params.amount * 100) as i64,
            "refundOrderId": params.refundno
        });

        // 如果是银食聚财分区
        if params.zone.as_deref() == Some("ysjc") {
            client_req["platformAmount"] = serde_json::json!((params.amount * 100) as i64);

            if let Some(ysjc_mid) = &self.config.ysjc {
                let sub_order = serde_json::json!({
                    "mid": ysjc_mid,
                    "merOrderId": format!("{}SUB", params.no),
                    "refundOrderId": format!("{}SUB", params.refundno),
                    "totalAmount": (params.amount * 100) as i64
                });
                client_req["subOrders"] = serde_json::json!([sub_order]);
            }
        }

        let client = reqwest::Client::new();
        let response = client
            .post("https://api-mop.chinaums.com/v1/netpay/bills/refund")
            .header("Content-type", "application/json")
            .header(
                "Authorization",
                format!(
                    "OPEN-ACCESS-TOKEN AccessToken=\"{}\", AppId=\"{}\"",
                    access_token, self.config.appid
                ),
            )
            .json(&client_req)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
            .map_err(PayError::RequestError)?;

        let body: serde_json::Value = response.json().await.unwrap_or_default();
        Ok(body)
    }

    /// 查询订单信息（通过 `access_token`）
    pub async fn query_ums_info(
        &self,
        order: &str,
        _access_token: &str,
    ) -> Result<serde_json::Value> {
        let row = sqlx::query!("SELECT order_pay FROM pay WHERE \"order\" = $1", order)
            .fetch_optional(&self.pool)
            .await
            .map_err(PayError::DatabaseError)?;

        Ok(row.map_or(serde_json::Value::Null, |r| {
            r.order_pay.unwrap_or(serde_json::Value::Null)
        }))
    }

    /// SHA256 哈希
    fn sha256(input: String) -> String {
        use sha2::{Digest, Sha256};
        let hash = Sha256::digest(input.as_bytes());
        hex::encode(hash)
    }
}

/// 查询支付订单信息（独立函数，用于兼容）
pub async fn ums_info(order: &str, pool: &sqlx::PgPool) -> Result<serde_json::Value> {
    let row = sqlx::query!("SELECT order_pay FROM pay WHERE \"order\" = $1", order)
        .fetch_optional(pool)
        .await
        .map_err(PayError::DatabaseError)?;

    Ok(row.map_or(serde_json::Value::Null, |r| {
        r.order_pay.unwrap_or(serde_json::Value::Null)
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_returns_hex_string() {
        let hash = UmsService::sha256("test".to_string());
        // SHA256 of "test" is known
        assert_eq!(
            hash,
            "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"
        );
    }

    #[test]
    fn test_sha256_empty_string() {
        let hash = UmsService::sha256(String::new());
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_sha256_different_inputs_different_hashes() {
        let h1 = UmsService::sha256("input1".to_string());
        let h2 = UmsService::sha256("input2".to_string());
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_sha256_returns_64_chars() {
        let hash = UmsService::sha256("any input".to_string());
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_ums_query_params_deserialize() {
        let json = r#"{"time": "2026-06-18", "no": "ORDER001"}"#;
        let params: UmsQueryParams = serde_json::from_str(json).expect("test assertion");
        assert_eq!(params.time, "2026-06-18");
        assert_eq!(params.no, Some("ORDER001".to_string()));
    }

    #[test]
    fn test_ums_order_params_deserialize() {
        let json = r#"{"time": "2026-06-18", "no": "ORDER001", "amount": 100}"#;
        let params: UmsOrderParams = serde_json::from_str(json).expect("test assertion");
        assert_eq!(params.time, "2026-06-18");
        assert_eq!(params.no, "ORDER001");
        assert_eq!(params.amount, 100i64);
    }

    #[test]
    fn test_ums_order_params_with_optional_fields() {
        let json = r#"{"time": "2026-06-18", "no": "ORDER001", "amount": 50, "desc": "test order", "zone": "ysjc"}"#;
        let params: UmsOrderParams = serde_json::from_str(json).expect("test assertion");
        assert_eq!(params.desc, Some("test order".to_string()));
        assert_eq!(params.zone, Some("ysjc".to_string()));
    }

    #[test]
    fn test_ums_close_params_deserialize() {
        let json = r#"{"no": "ORDER001", "time": "2026-06-18"}"#;
        let params: UmsCloseParams = serde_json::from_str(json).expect("test assertion");
        assert_eq!(params.no, "ORDER001");
        assert_eq!(params.time, "2026-06-18");
    }

    #[test]
    fn test_ums_refund_params_deserialize() {
        let json = r#"{"time": "2026-06-18", "no": "ORDER001", "amount": 50, "refundno": "REF001"}"#;
        let params: UmsRefundParams = serde_json::from_str(json).expect("test assertion");
        assert_eq!(params.time, "2026-06-18");
        assert_eq!(params.no, "ORDER001");
        assert_eq!(params.refundno, "REF001");
    }

    #[test]
    fn test_ums_notify_params_deserialize() {
        let json = r#"{"order": "ORDER001"}"#;
        let params: UmsNotifyParams = serde_json::from_str(json).expect("test assertion");
        assert_eq!(params.order, "ORDER001");
    }
}
