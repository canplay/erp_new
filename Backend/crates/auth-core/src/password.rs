//! 密码服务

use common::validation;

/// 密码服务
#[derive(Clone)]
pub struct PasswordService;

impl Default for PasswordService {
    fn default() -> Self {
        Self
    }
}

impl PasswordService {
    /// 创建新的密码服务（兼容旧版，cost 参数不再使用）
    #[must_use]
    pub const fn new(_cost: u32) -> Self {
        Self
    }

    /// 哈希密码（委托 `common::validation::hash_password`）
    pub fn hash_password(&self, password: &str) -> Result<String, String> {
        validation::hash_password(password)
    }

    /// 验证密码（委托 `common::validation::verify_password`）
    #[must_use]
    pub fn verify_password(&self, password: &str, hash: &str) -> bool {
        validation::verify_password(password, hash)
    }

    /// 兼容别名：哈希密码
    pub fn hash_bcrypt(&self, password: &str) -> Result<String, String> {
        self.hash_password(password)
    }

    /// 兼容别名：验证密码
    #[must_use]
    pub fn verify_bcrypt(&self, password: &str, hash: &str) -> bool {
        self.verify_password(password, hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hash_and_verify() {
        let service = PasswordService;
        let hash = service.hash_password("password123" ).unwrap();
        assert!(service.verify_password("password123" , &hash));
        assert!(!service.verify_password("wrongpassword" , &hash));
    }

    #[test]
    fn test_compat_aliases() {
        let service = PasswordService;
        let hash = service.hash_bcrypt("password123" ).unwrap();
        assert!(service.verify_bcrypt("password123" , &hash));
        assert!(!service.verify_bcrypt("wrongpassword" , &hash));
    }
}
