//! Crypto Core - 密码学核心库
//!
//! 提供国密算法 (SM2/SM3/SM4) 支持

pub mod gm;

// Re-export types from gm module
pub use gm::CryptoError;
pub use gm::GmCrypto;
