//! 数据库仓库通用接口

use sqlx::postgres::PgPool;

/// 通用数据仓库 trait
pub trait GenericRepository: Send + Sync {
    type Model;
    fn pool(&self) -> &PgPool;
}

/// 验证并构造安全的分表表名
///
/// 仅允许 `prefix_YYYY_MM` 格式，其中 YYYY 在 2020-2100 范围内，MM 在 1-12 范围内。
/// 这是为了防止 SQL 注入——虽然 year/month 通常来自 chrono 而非用户输入，
/// 但显式验证提供了纵深防御。
pub fn safe_table_name(prefix: &str, year: i32, month: u32) -> Result<String, String> {
    // 验证前缀只包含字母和下划线
    if prefix.is_empty() || !prefix.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(format!("Invalid table prefix: {prefix}" ));
    }
    // 验证年份范围
    if !(2020..=2100).contains(&year) {
        return Err(format!("Year out of range: {year}" ));
    }
    // 验证月份范围
    if !(1..=12).contains(&month) {
        return Err(format!("Month out of range: {month}" ));
    }
    Ok(format!("{prefix}_{year}_{month}" ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_table_name() {
        assert_eq!(safe_table_name("car_history" , 2024, 6).expect("table name should be valid" ), "car_history_2024_6" );
        assert_eq!(safe_table_name("order" , 2025, 12).expect("table name should be valid" ), "order_2025_12" );
    }

    #[test]
    fn test_invalid_year() {
        assert!(safe_table_name("car" , 1999, 6).is_err());
        assert!(safe_table_name("car" , 2101, 6).is_err());
    }

    #[test]
    fn test_invalid_month() {
        assert!(safe_table_name("car" , 2024, 0).is_err());
        assert!(safe_table_name("car" , 2024, 13).is_err());
    }

    #[test]
    fn test_invalid_prefix() {
        assert!(safe_table_name("" , 2024, 6).is_err());
        assert!(safe_table_name("car;drop" , 2024, 6).is_err());
        assert!(safe_table_name("car history" , 2024, 6).is_err());
    }
}
