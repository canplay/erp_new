//! CTP 平板锁状态字解析器
//!
//! 依据 CTP 协议附录2：解析 StatusOne（8位16进制）和 StatusTwo（12位16进制）状态码。
//!
//! # `StatusOne` 编码 (32 bits, 8 hex digits)
//!
//! | 位 | 说明 |
//! |---|------|
//! | 31-28 | 锁状态: 4=Locked(降板), 2=Unlocked(升板), 1=中间态 |
//! | 27-24 | 保留/备用 |
//! | 23-20 | 左线圈状态 |
//! | 19-16 | 右线圈状态 |
//! | 15-12 | 电量等级: 0=正常, 1=低电量, 2=极低 |
//! | 11-8  | 保留/备用 |
//! | 7-4   | 复位标志: 1=已复位 |
//! | 3-0   | 心跳: 1=活 | 报警: 1=有报警 |
//!
//! # `StatusTwo` 编码 (48 bits, 12 hex digits)
//!
//! | 位 | 说明 |
//! |---|------|
//! | 47-44 | 总入位次数(高位) |
//! | 43-40 | 总入位次数(低位) |
//! | 39-36 | 总出位次数(高位) |
//! | 35-32 | 总出位次数(低位) |
//! | 31-28 | 总逃费次数(高位) |
//! | 27-24 | 总逃费次数(低位) |
//! | 23-20 | 当前入位数量(高位) |
//! | 19-16 | 当前入位数量(低位) |
//! | 15-0  | 保留/设备扩展标识 |

use serde::Serialize;

/// 状态字解析结果
#[derive(Debug, Clone, Serialize)]
pub struct StatusParseResult {
    /// `StatusOne` 解析
    pub status_one: StatusOneParsed,
    /// `StatusTwo` 解析（如提供）
    pub status_two: Option<StatusTwoParsed>,
    /// 锁状态推断
    pub lock_status: String,
    /// 电池等级描述
    pub battery_level: String,
    /// 数据有效性标记
    pub valid: bool,
}

/// `StatusOne` 解析结果
#[derive(Debug, Clone, Serialize)]
pub struct StatusOneParsed {
    /// 原始16进制字符串
    pub raw: String,
    /// 锁状态 (Locked/Unlocked/Transient/Fault)
    pub lock_state: String,
    /// 左线圈状态
    pub left_coil: String,
    /// 右线圈状态
    pub right_coil: String,
    /// 电量等级
    pub battery_raw: u8,
    /// 复位标志
    pub reset_flag: bool,
    /// 心跳标志
    pub heartbeat: bool,
    /// 报警标志
    pub alarm: bool,
}

/// `StatusTwo` 解析结果
#[derive(Debug, Clone, Serialize)]
pub struct StatusTwoParsed {
    /// 原始16进制字符串
    pub raw: String,
    /// 总入位次数
    pub total_entry_count: u32,
    /// 总出位次数
    pub total_exit_count: u32,
    /// 总逃费次数
    pub total_theft_count: u32,
    /// 当前入位车辆数
    pub current_occupancy: u32,
}

/// 解析 `StatusOne` 状态码
///
/// `status_one` 为 8 位 16 进制字符串（如 "40000000"）。
/// 如为空或无效格式，返回默认值。
pub fn parse_status_one(status_one: &str) -> StatusOneParsed {
    let raw = status_one.to_string();
    let val = u32::from_str_radix(status_one, 16).unwrap_or(0);

    // 31-28 位: 锁状态
    let lock_bits = (val >> 28) & 0xF;
    let lock_state = match lock_bits {
        0x4 => "Locked".to_string(),
        0x2 => "Unlocked".to_string(),
        0x1 => "Transient".to_string(),
        0x6 => "Locked(Force)".to_string(),
        0x3 => "Unlocked(Force)".to_string(),
        _ if val == 0 => "Offline".to_string(),
        _ => format!("Unknown(0x{lock_bits:X})" ),
    };

    // 23-20 位: 左线圈
    let left_coil_val = (val >> 20) & 0xF;
    let left_coil = match left_coil_val {
        0x0 => "Normal".to_string(),
        0x1 => "Triggered".to_string(),
        _ => format!("0x{left_coil_val:X}" ),
    };

    // 19-16 位: 右线圈
    let right_coil_val = (val >> 16) & 0xF;
    let right_coil = match right_coil_val {
        0x0 => "Normal".to_string(),
        0x1 => "Triggered".to_string(),
        _ => format!("0x{right_coil_val:X}" ),
    };

    // 15-12 位: 电量等级
    let battery_raw = ((val >> 12) & 0xF) as u8;

    // 7-4 位: 复位标志
    let reset_flag = ((val >> 4) & 0xF) & 0x1 != 0;

    // 3-0 位: 心跳+报警
    let heartbeat = val & 0x1 != 0;
    let alarm = (val >> 1) & 0x1 != 0;

    StatusOneParsed {
        raw,
        lock_state,
        left_coil,
        right_coil,
        battery_raw,
        reset_flag,
        heartbeat,
        alarm,
    }
}

/// 解析 `StatusTwo` 状态码
///
/// `status_two` 为 12 位 16 进制字符串（如 "032014C81500"）。
/// 如为空或无效格式，返回 None。
pub fn parse_status_two(status_two: &str) -> Option<StatusTwoParsed> {
    if status_two.is_empty() || status_two.len() < 8 {
        return None;
    }

    let raw = status_two.to_string();
    let val = u64::from_str_radix(status_two, 16).unwrap_or(0);

    // 解析各计数字段（每4位一个BCD-like编码）
    // 47-44: 总入位高位, 43-40: 总入位低位
    let entry_high = ((val >> 44) & 0xF) as u32;
    let entry_low = ((val >> 40) & 0xF) as u32;
    let total_entry_count = entry_high * 10 + entry_low;

    // 39-36: 总出位高位, 35-32: 总出位低位
    let exit_high = ((val >> 36) & 0xF) as u32;
    let exit_low = ((val >> 32) & 0xF) as u32;
    let total_exit_count = exit_high * 10 + exit_low;

    // 31-28: 总逃费高位, 27-24: 总逃费低位
    let theft_high = ((val >> 28) & 0xF) as u32;
    let theft_low = ((val >> 24) & 0xF) as u32;
    let total_theft_count = theft_high * 10 + theft_low;

    // 23-20: 当前入位高位, 19-16: 当前入位低位
    let occ_high = ((val >> 20) & 0xF) as u32;
    let occ_low = ((val >> 16) & 0xF) as u32;
    let current_occupancy = occ_high * 10 + occ_low;

    Some(StatusTwoParsed {
        raw,
        total_entry_count,
        total_exit_count,
        total_theft_count,
        current_occupancy,
    })
}

/// 综合解析 `StatusOne` 和 StatusTwo（便捷函数）
#[must_use]
pub fn parse_status(status_one: &str, status_two: &str) -> StatusParseResult {
    let parsed_one = parse_status_one(status_one);
    let parsed_two = parse_status_two(status_two);

    let battery_level = match parsed_one.battery_raw {
        0 => "Normal".to_string(),
        1 => "Low".to_string(),
        2 => "Critical".to_string(),
        _ => format!("Level{}" , parsed_one.battery_raw),
    };

    StatusParseResult {
        valid: !status_one.is_empty() && status_one.len() == 8,
        lock_status: parsed_one.lock_state.clone(),
        battery_level,
        status_one: parsed_one,
        status_two: parsed_two,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_status_one_locked() {
        let result = parse_status_one("40000000" );
        assert_eq!(result.lock_state, "Locked" );
        assert!(!result.alarm);
        assert!(!result.reset_flag);
    }

    #[test]
    fn test_parse_status_one_unlocked() {
        let result = parse_status_one("20000001" );
        assert_eq!(result.lock_state, "Unlocked" );
        assert!(result.heartbeat);
    }

    #[test]
    fn test_parse_status_one_with_alarm() {
        let result = parse_status_one("40000002" );
        assert_eq!(result.lock_state, "Locked" );
        assert!(result.alarm);
    }

    #[test]
    fn test_parse_status_two_counts() {
        let result = parse_status_two("032014C81500" ).expect("status parsing should not fail" );
        // 0320 = entry count, 14C8 = exit count, 1500 = theft count
        assert!(result.total_entry_count > 0 || result.total_exit_count > 0);
    }

    #[test]
    fn test_parse_status_empty() {
        let result = parse_status_one("" );
        assert_eq!(result.lock_state, "Offline" );
        assert!(parse_status_two("" ).is_none());
    }

    #[test]
    fn test_parse_status_combined() {
        let result = parse_status("40000000" , "032014C81500" );
        assert!(result.valid);
        assert_eq!(result.lock_status, "Locked" );
        assert!(result.status_two.is_some());
    }
}
