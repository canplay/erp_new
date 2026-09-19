use std::time::{SystemTime, UNIX_EPOCH};

/// 生成唯一 ID
pub fn generate_id() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| {
            tracing::error!("system time is before UNIX epoch");
            std::time::Duration::from_secs(0)
        })
        .as_nanos();
    format!("{timestamp:016x}")
}
