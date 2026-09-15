//! 停车服务
//!
//! 处理车辆入场/出场逻辑，使用 Redis 缓存在场车辆信息。

use common::AppError;
use common::AppResult;
use crate::models::{ParkingVehicle, VehicleEvent, XltConfig};

/// 停车服务
pub struct ParkingService {
    config: XltConfig,
}

impl Default for ParkingService {
    fn default() -> Self {
        Self::new()
    }
}

impl ParkingService {
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: XltConfig::from_env(),
        }
    }

    /// 车辆入场处理
    pub async fn handle_entry(&self, event: &VehicleEvent) -> AppResult<VehicleEvent> {
        let client = redis::Client::open(self.config.redis_url.as_str())
            .map_err(AppError::from)?;
        let mut conn = client
            .get_multiplexed_async_connection()
            .await
            .map_err(AppError::from)?;

        let vehicle_key = format!("xlt:parking:{}:{}", event.park_code, event.plate_no);
        let now = chrono::Utc::now().to_rfc3339();

        let record = serde_json::json!({
            "plateNo": event.plate_no,
            "plateColor": event.plate_color,
            "parkCode": event.park_code,
            "laneCode": event.lane_code,
            "entryTime": event.event_time,
            "vehicleType": event.vehicle_type,
            "imageUrl": event.image_url,
            "status": "parking",
            "createdAt": now,
        });

        let _: () = redis::AsyncCommands::set(
            &mut conn,
            &vehicle_key,
            serde_json::to_string(&record).unwrap_or_default(),
        )
        .await
        .map_err(AppError::from)?;

        tracing::info!(
            "车辆入场: plate_no={}, park_code={}",
            event.plate_no,
            event.park_code
        );

        Ok(event.clone())
    }

    /// 车辆出场处理
    pub async fn handle_exit(&self, event: &VehicleEvent) -> AppResult<VehicleEvent> {
        let client = redis::Client::open(self.config.redis_url.as_str())
            .map_err(AppError::from)?;
        let mut conn = client
            .get_multiplexed_async_connection()
            .await
            .map_err(AppError::from)?;

        let vehicle_key = format!("xlt:parking:{}:{}", event.park_code, event.plate_no);

        let _: () = redis::AsyncCommands::del(&mut conn, &vehicle_key)
            .await
            .map_err(AppError::from)?;

        tracing::info!(
            "车辆出场: plate_no={}, park_code={}, amount={}",
            event.plate_no,
            event.park_code,
            event.amount
        );

        Ok(event.clone())
    }

    /// 查询在场车辆
    pub async fn get_parking_vehicle(
        &self,
        park_code: &str,
        plate_no: &str,
    ) -> AppResult<ParkingVehicle> {
        let client = redis::Client::open(self.config.redis_url.as_str())
            .map_err(AppError::from)?;
        let mut conn = client
            .get_multiplexed_async_connection()
            .await
            .map_err(AppError::from)?;

        let vehicle_key = format!("xlt:parking:{park_code}:{plate_no}");
        let data: Option<String> = redis::AsyncCommands::get(&mut conn, &vehicle_key)
            .await
            .map_err(AppError::from)?;

        match data {
            Some(json_str) => {
                let v: serde_json::Value =
                    serde_json::from_str(&json_str).map_err(|e| AppError::Internal(e.to_string()))?;

                let entry_time = v["entryTime"].as_str().unwrap_or("").to_string();
                let duration = Self::calc_duration_minutes(&entry_time);

                Ok(ParkingVehicle {
                    plate_no: v["plateNo"].as_str().unwrap_or("").to_string(),
                    plate_color: v["plateColor"].as_str().unwrap_or("").to_string(),
                    park_code: v["parkCode"].as_str().unwrap_or("").to_string(),
                    entry_time: entry_time.clone(),
                    lane_code: v["laneCode"].as_str().unwrap_or("").to_string(),
                    vehicle_type: v["vehicleType"].as_str().unwrap_or("").to_string(),
                    duration_minutes: duration,
                    amount: Self::calc_billing_default(&entry_time),
                })
            }
            None => Err(AppError::XltVehicleNotFound),
        }
    }

    fn calc_duration_minutes(entry_time: &str) -> i64 {
        let entry = chrono::DateTime::parse_from_rfc3339(entry_time).map_or_else(|_| chrono::Utc::now(), |dt| dt.with_timezone(&chrono::Utc));
        let now = chrono::Utc::now();
        (now - entry).num_minutes().max(0)
    }

    fn calc_billing_default(entry_time: &str) -> i64 {
        let entry = chrono::DateTime::parse_from_rfc3339(entry_time).map_or_else(|_| chrono::Utc::now(), |dt| dt.with_timezone(&chrono::Utc));
        let exit = chrono::Utc::now();
        let duration = (exit - entry).num_minutes().max(0);

        let free_minutes = 15i64;
        let first_hour_fee = 500i64;
        let hourly_fee = 200i64;
        let daily_cap = 3000i64;

        if duration <= free_minutes {
            return 0;
        }

        let billable = duration - free_minutes;
        let hours = (billable + 59) / 60;

        let total = if hours <= 1 {
            first_hour_fee
        } else {
            first_hour_fee + (hours - 1) * hourly_fee
        };

        total.min(daily_cap)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calc_duration_minutes_returns_positive() {
        let entry = "2026-06-18T08:00:00Z";
        let duration = ParkingService::calc_duration_minutes(entry);
        assert!(duration >= 0);
    }

    #[test]
    fn test_calc_duration_minutes_recent_time() {
        // Use a time just a few seconds ago
        let now = chrono::Utc::now();
        let recent = (now - chrono::Duration::seconds(30)).to_rfc3339();
        let duration = ParkingService::calc_duration_minutes(&recent);
        assert_eq!(duration, 0);
    }

    #[test]
    fn test_calc_duration_minutes_one_hour_ago() {
        let now = chrono::Utc::now();
        let one_hour_ago = (now - chrono::Duration::hours(1)).to_rfc3339();
        let duration = ParkingService::calc_duration_minutes(&one_hour_ago);
        assert_eq!(duration, 60);
    }

    #[test]
    fn test_calc_duration_invalid_time_returns_zero() {
        let duration = ParkingService::calc_duration_minutes("invalid-date");
        // Invalid dates default to Utc::now(), so duration is 0
        assert_eq!(duration, 0);
    }

    #[test]
    fn test_calc_billing_default_free_period() {
        let now = chrono::Utc::now();
        let recent = (now - chrono::Duration::minutes(10)).to_rfc3339();
        assert_eq!(ParkingService::calc_billing_default(&recent), 0);
    }

    #[test]
    fn test_calc_billing_default_first_hour() {
        let now = chrono::Utc::now();
        let entry = (now - chrono::Duration::minutes(30)).to_rfc3339();
        // 30 min: 15 free + 15 billable = ceil(15/60)=1 = first_hour_fee(500)
        assert_eq!(ParkingService::calc_billing_default(&entry), 500);
    }

    #[test]
    fn test_calc_billing_default_multi_hour() {
        let now = chrono::Utc::now();
        let entry = (now - chrono::Duration::minutes(90)).to_rfc3339();
        // 90 min: 15 free + 75 billable = ceil(75/60)=2 hours
        // first hour 500 + 200 = 700
        assert_eq!(ParkingService::calc_billing_default(&entry), 700);
    }

    #[test]
    fn test_calc_billing_default_daily_cap() {
        let now = chrono::Utc::now();
        let entry = (now - chrono::Duration::hours(24)).to_rfc3339();
        // 1440 min should be capped at daily_cap(3000)
        assert_eq!(ParkingService::calc_billing_default(&entry), 3000);
    }
}
