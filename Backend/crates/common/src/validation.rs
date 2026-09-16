//! 参数校验和密码工具
//!
//! 提供通用的参数校验和密码处理功能

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use lazy_static::lazy_static;
use rand::RngExt;
use regex::Regex;

lazy_static! {
    /// 邮箱校验正则表达式（预编译缓存，避免每次调用重复编译）
    static ref EMAIL_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$" ).expect("邮箱正则表达式编译失败" );
    /// 手机号校验正则表达式（中国大陆，预编译缓存）
    static ref PHONE_REGEX: Regex = Regex::new(r"^1[3-9]\d{9}$" ).expect("手机号正则表达式编译失败" );
}

/// 密码哈希（使用 argon2）
///
/// # 参数
/// * `password` - 明文密码
///
/// # 返回
/// 成功返回密码哈希字符串，失败返回错误
pub fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| format!("密码哈希失败: {e}" ))
}

/// 验证密码（优先 argon2，兼容 bcrypt 旧哈希）
///
/// # 参数
/// * `password` - 明文密码
/// * `hash` - 存储的密码哈希
///
/// # 返回
/// 验证成功返回 true，失败返回 false
#[must_use]
pub fn verify_password(password: &str, hash: &str) -> bool {
    // 兼容 bcrypt 旧哈希（$2a$/$2b$/$2y$ 前缀）
    if hash.starts_with("$2a$" ) || hash.starts_with("$2b$" ) || hash.starts_with("$2y$" ) {
        return bcrypt::verify(password, hash).unwrap_or(false);
    }
    let parsed_hash = match PasswordHash::new(hash) {
        Ok(h) => h,
        Err(_) => return false,
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

/// 验证用户名格式
///
/// 要求：3-50 个字符，只能包含字母、数字和下划线
///
/// # 参数
/// * `username` - 用户名
///
/// # 返回
/// 验证成功返回空字符串，失败返回错误消息
pub fn validate_username(username: &str) -> Result<(), &'static str> {
    if username.is_empty() {
        return Err("用户名不能为空" );
    }
    if username.len() < 3 {
        return Err("用户名长度不能少于3个字符" );
    }
    if username.len() > 50 {
        return Err("用户名长度不能超过50个字符" );
    }
    if !username.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err("用户名只能包含字母、数字和下划线" );
    }
    Ok(())
}

/// 验证密码强度
///
/// 要求：至少 8 个字符
///
/// # 参数
/// * `password` - 密码
///
/// # 返回
/// 验证成功返回空字符串，失败返回错误消息
pub const fn validate_password(password: &str) -> Result<(), &'static str> {
    if password.len() < 8 {
        return Err("密码长度至少 8 个字符" );
    }
    Ok(())
}

/// 验证邮箱格式
///
/// # 参数
/// * `email` - 邮箱地址
///
/// # 返回
/// 验证成功返回空字符串，失败返回错误消息
pub fn validate_email(email: &str) -> Result<(), &'static str> {
    if email.is_empty() {
        return Err("邮箱不能为空" );
    }
    // 简单的邮箱格式验证
    if !EMAIL_REGEX.is_match(email) {
        return Err("邮箱格式不正确" );
    }
    Ok(())
}

/// 验证手机号格式
///
/// 支持格式：11 位数字，以 1 开头
///
/// # 参数
/// * `phone` - 手机号
///
/// # 返回
/// 验证成功返回空字符串，失败返回错误消息
pub fn validate_phone(phone: &str) -> Result<(), &'static str> {
    if phone.is_empty() {
        return Err("手机号不能为空" );
    }
    if !PHONE_REGEX.is_match(phone) {
        return Err("手机号格式不正确" );
    }
    Ok(())
}

/// 验证角色值
///
/// # 参数
/// * `role` - 角色名
/// * `valid_roles` - 允许的角色列表
///
/// # 返回
/// 验证成功返回空字符串，失败返回错误消息
pub fn validate_role(role: &str, valid_roles: &[&str]) -> Result<(), &'static str> {
    if !valid_roles.contains(&role) {
        return Err("无效的角色值" );
    }
    Ok(())
}

/// 验证状态值
///
/// # 参数
/// * `status` - 状态值
/// * `valid_statuses` - 允许的状态列表
///
/// # 返回
/// 验证成功返回空字符串，失败返回错误消息
pub fn validate_status(status: i32, valid_statuses: &[i32]) -> Result<(), &'static str> {
    if !valid_statuses.contains(&status) {
        return Err("无效的状态值" );
    }
    Ok(())
}

/// 生成随机密码
///
/// 生成一个指定长度的随机密码，包含大小写字母和数字
///
/// # 参数
/// * `length` - 密码长度（默认 12）
///
/// # 返回
/// 生成的随机密码
#[must_use]
pub fn generate_random_password(length: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::rng();
    (0..length)
        .map(|_| {
            let idx: usize = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// 验证 ID 参数
///
/// # 参数
/// * `id` - ID 值
///
/// # 返回
/// 验证成功返回空字符串，失败返回错误消息
pub const fn validate_id(id: i64) -> Result<(), &'static str> {
    if id <= 0 {
        return Err("ID 必须大于 0" );
    }
    Ok(())
}

/// 验证分页参数
///
/// # 参数
/// * `page` - 页码
/// * `page_size` - 每页大小
///
/// # 返回
/// 验证成功返回 (page, `page_size)，失败返回错误消息`
pub fn validate_pagination(
    page: Option<i32>,
    page_size: Option<i32>,
) -> Result<(i32, i32), &'static str> {
    let p = page.unwrap_or(1).max(1);
    let size = page_size.unwrap_or(10).clamp(1, 100);
    Ok((p, size))
}

/// 验证字符串长度范围
///
/// # 参数
/// * `value` - 待验证的字符串
/// * `min_len` - 最小长度
/// * `max_len` - 最大长度
/// * `field_name` - 字段名称（用于错误消息）
///
/// # 返回
/// 验证成功返回空字符串，失败返回错误消息
pub fn validate_string_length(
    value: &str,
    min_len: usize,
    max_len: usize,
    field_name: &str,
) -> Result<(), String> {
    let len = value.len();
    if len < min_len {
        return Err(format!("{field_name} 长度不能少于 {min_len} 个字符" ));
    }
    if len > max_len {
        return Err(format!("{field_name} 长度不能超过 {max_len} 个字符" ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify_password() {
        let password = "Test@123456";
        let hash = hash_password(password).unwrap();
        assert!(verify_password(password, &hash));
        assert!(!verify_password("wrong_password" , &hash));
    }

    #[test]
    fn test_validate_username() {
        assert!(validate_username("user123" ).is_ok());
        assert!(validate_username("user_name" ).is_ok());
        assert!(validate_username("ab" ).is_err()); // 太短
        assert!(validate_username("user@name" ).is_err()); // 包含非法字符
    }

    #[test]
    fn test_validate_email() {
        assert!(validate_email("test@example.com" ).is_ok());
        assert!(validate_email("invalid" ).is_err());
    }

    #[test]
    fn test_validate_phone() {
        assert!(validate_phone("13800000001" ).is_ok());
        assert!(validate_phone("12345" ).is_err()); // 太短
        assert!(validate_phone("123456789012" ).is_err()); // 太长
    }

    #[test]
    fn test_generate_random_password() {
        let password = generate_random_password(16);
        assert_eq!(password.len(), 16);
    }
}
