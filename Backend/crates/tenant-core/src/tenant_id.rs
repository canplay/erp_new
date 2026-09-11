//! 租户 ID Newtype
//!
//! 提供类型安全的租户 ID，防止与其他 i64 ID 混淆。

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// 租户 ID（newtype 包装 i64）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TenantId(pub i64);

impl TenantId {
    /// 创建新的租户 ID
    pub const fn new(id: i64) -> Self {
        Self(id)
    }

    /// 获取内部 i64 值
    pub const fn value(self) -> i64 {
        self.0
    }
}

impl From<i64> for TenantId {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl From<TenantId> for i64 {
    fn from(value: TenantId) -> Self {
        value.0
    }
}

impl std::fmt::Display for TenantId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TenantId({})", self.0)
    }
}

impl Serialize for TenantId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_i64(self.0)
    }
}

impl<'de> Deserialize<'de> for TenantId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        i64::deserialize(deserializer).map(Self)
    }
}

impl std::ops::Deref for TenantId {
    type Target = i64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tenant_id_new() {
        let id = TenantId::new(42);
        assert_eq!(id.value(), 42);
    }

    #[test]
    fn test_tenant_id_from_i64() {
        let id: TenantId = 100.into();
        assert_eq!(id.value(), 100);
    }

    #[test]
    fn test_tenant_id_into_i64() {
        let id = TenantId::new(200);
        let raw: i64 = id.into();
        assert_eq!(raw, 200);
    }

    #[test]
    fn test_tenant_id_display() {
        let id = TenantId::new(42);
        assert_eq!(format!("{}", id), "TenantId(42)");
    }

    #[test]
    fn test_tenant_id_equality() {
        let a = TenantId::new(1);
        let b = TenantId::new(1);
        let c = TenantId::new(2);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn test_tenant_id_serialization() {
        let id = TenantId::new(123);
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "123");

        let deserialized: TenantId = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, id);
    }

    #[test]
    fn test_tenant_id_deref() {
        let id = TenantId::new(42);
        assert_eq!(*id, 42);
    }
}
