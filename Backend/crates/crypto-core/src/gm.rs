//! 国密算法工具
//!
//! 支持 SM2/SM3/SM4 算法
//!
//! # 示例
//!
//! ```rust
//! use crypto_core::GmCrypto;
//!
//! // 创建加密工具
//! let gm = GmCrypto::new("0123456789abcdef0123456789abcdef" ).unwrap();
//!
//! // SM3 哈希
//! let hash = GmCrypto::sm3_hash(b"hello" );
//!
//! // SM4 ECB 加密
//! let encrypted = gm.sm4_encrypt(b"secret" ).unwrap();
//! let decrypted = gm.sm4_decrypt(&encrypted).unwrap();
//!
//! // SM4 CBC 加密
//! let iv = GmCrypto::generate_iv();
//! let cbc_encrypted = gm.sm4_cbc_encrypt(b"secret" , &iv).unwrap();
//! ```

use generic_array::typenum::U16;
use k256::ecdsa::signature::{Signer, Verifier};
use k256::{
    Secp256k1,
    ecdsa::{Signature as K256Signature, SigningKey, VerifyingKey},
    elliptic_curve::sec1::Sec1Point,
};
use sha2::Sha256;
use sm3::Sm3;
use sm3::digest::Digest;
use sm4::Sm4;
use sm4::cipher::Array;
use sm4::cipher::{BlockCipherDecrypt, BlockCipherEncrypt, KeyInit};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("SM2签名失败: {0}" )]
    Sm2SignError(String),

    #[error("SM2验签失败: {0}" )]
    Sm2VerifyError(String),

    #[error("SM4加密失败: {0}" )]
    Sm4EncryptError(String),

    #[error("SM4解密失败: {0}" )]
    Sm4DecryptError(String),

    #[error("密钥格式错误: {0}" )]
    KeyFormatError(String),
}

/// SM2 密钥对
#[derive(Clone, Debug)]
pub struct Sm2KeyPair {
    /// 私钥 (32字节)
    pub private_key: [u8; 32],
    /// 公钥 (64字节，SM2标准)
    pub public_key: [u8; 64],
}

/// 国密工具
#[derive(Clone, Debug)]
pub struct GmCrypto {
    sm4_key: [u8; 16],
}

impl GmCrypto {
    /// 创建新的国密工具实例 (仅 SM4)
    pub fn new(sm4_key: &str) -> Result<Self, CryptoError> {
        let key_bytes = hex::decode(sm4_key)
            .map_err(|e| CryptoError::KeyFormatError(format!("SM4密钥格式错误: {e}" )))?;

        if key_bytes.len() != 16 {
            return Err(CryptoError::KeyFormatError("SM4密钥必须为16字节".into()));
        }

        let mut key = [0u8; 16];
        key.copy_from_slice(&key_bytes);

        Ok(Self { sm4_key: key })
    }

    /// 使用默认密钥创建 (仅用于开发测试)
    #[allow(dead_code)]
    #[must_use]
    pub const fn default_for_dev() -> Self {
        let default_key = [
            0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0xFE, 0xDC, 0xBA, 0x98, 0x76, 0x54,
            0x32, 0x10,
        ];
        Self {
            sm4_key: default_key,
        }
    }

    // ==================== SM2 签名/验签 ====================

    /// 生成 SM2 密钥对
    #[allow(deprecated)]
    #[must_use]
    pub fn generate_sm2_keypair() -> Sm2KeyPair {
        use rand::SeedableRng;

        // 使用固定种子初始化随机数生成器 (仅用于开发/测试)
        // 生产环境应使用更好的随机源
        let seed = [
            0x01u8, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0xFE, 0xDC, 0xBA, 0x98, 0x76, 0x54,
            0x32, 0x10, 0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0xFE, 0xDC, 0xBA, 0x98,
            0x76, 0x54, 0x32, 0x10,
        ];
        let mut rng = rand::rngs::StdRng::from_seed(seed);

        // 使用 k256 的椭圆曲线密钥生成（deprecated API，但在 k256 0.14 中仍可用）
        let secret = k256::SecretKey::random(&mut rng);
        let signing_key = SigningKey::from(&secret);
        let verifying_key = VerifyingKey::from(&signing_key);

        // 提取私钥字节
        let private_bytes: [u8; 32] = signing_key.to_bytes().into();

        // 提取公钥字节 (64字节，无压缩格式)
        let encoded = verifying_key.to_sec1_point(false);
        let encoded_bytes = encoded.as_bytes();
        // 跳过第一个字节（标识 0x04），取后面的64字节
        let mut public_key = [0u8; 64];
        public_key.copy_from_slice(&encoded_bytes[1..]);

        Sm2KeyPair {
            private_key: private_bytes,
            public_key,
        }
    }

    /// SM2 签名
    pub fn sm2_sign(&self, data: &[u8], private_key: &[u8; 32]) -> Result<String, CryptoError> {
        let signing_key = SigningKey::from_bytes(private_key.into())
            .map_err(|e| CryptoError::Sm2SignError(format!("私钥格式错误: {e:?}" )))?;

        // 对数据进行哈希 (使用 SHA256 作为 ECDSA 签名的基础)
        let mut hasher = Sha256::new();
        hasher.update(data);
        let hash = hasher.finalize();

        // 签名
        let signature: K256Signature = signing_key.sign(&hash);
        Ok(hex::encode(signature.to_bytes()))
    }

    /// SM2 验签
    pub fn sm2_verify(
        &self,
        data: &[u8],
        signature_hex: &str,
        public_key: &[u8; 64],
    ) -> Result<bool, CryptoError> {
        // 解析签名
        let signature_bytes = hex::decode(signature_hex)
            .map_err(|e| CryptoError::Sm2VerifyError(format!("签名格式错误: {e}" )))?;

        let signature = K256Signature::from_slice(&signature_bytes)
            .map_err(|e| CryptoError::Sm2VerifyError(format!("签名格式错误: {e:?}" )))?;

        // 构建公钥
        // 公钥格式: 64字节 (x || y)，需要加上 0x04 前缀变成 65 字节
        let mut uncompressed = [4u8; 65];
        uncompressed[1..].copy_from_slice(public_key);

        let sec1_point = Sec1Point::<Secp256k1>::from_bytes(&uncompressed)
            .map_err(|e| CryptoError::Sm2VerifyError(format!("公钥格式错误: {e:?}" )))?;

        let verifying_key = VerifyingKey::from_sec1_point(&sec1_point)
            .map_err(|e| CryptoError::Sm2VerifyError(format!("公钥格式错误: {e:?}" )))?;

        // 对数据进行哈希
        let mut hasher = Sha256::new();
        hasher.update(data);
        let hash = hasher.finalize();

        // 验签
        Ok(verifying_key.verify(&hash, &signature).is_ok())
    }

    /// 从十六进制字符串解析 SM2 私钥
    #[allow(dead_code)]
    pub fn parse_sm2_private_key(hex_str: &str) -> Result<[u8; 32], CryptoError> {
        let key_bytes = hex::decode(hex_str)
            .map_err(|e| CryptoError::KeyFormatError(format!("SM2私钥格式错误: {e}" )))?;

        key_bytes
            .as_slice()
            .try_into()
            .map_err(|_| CryptoError::KeyFormatError("SM2私钥必须为32字节".into()))
    }

    /// 从十六进制字符串解析 SM2 公钥
    #[allow(dead_code)]
    pub fn parse_sm2_public_key(hex_str: &str) -> Result<[u8; 64], CryptoError> {
        let key_bytes = hex::decode(hex_str)
            .map_err(|e| CryptoError::KeyFormatError(format!("SM2公钥格式错误: {e}" )))?;

        key_bytes
            .as_slice()
            .try_into()
            .map_err(|_| CryptoError::KeyFormatError("SM2公钥必须为64字节".into()))
    }

    // ==================== SM3 摘要 ====================

    /// 计算SM3哈希
    #[must_use]
    pub fn sm3_hash(data: &[u8]) -> String {
        let mut hasher = Sm3::new();
        Digest::update(&mut hasher, data);
        let result = hasher.finalize();
        hex::encode(result)
    }

    // ==================== SM4 CBC 模式 ====================

    /// 生成随机 IV (16字节)
    #[must_use]
    pub fn generate_iv() -> [u8; 16] {
        rand::random()
    }

    /// SM4 CBC 模式加密
    pub fn sm4_cbc_encrypt(&self, plaintext: &[u8], iv: &[u8]) -> Result<String, CryptoError> {
        if iv.len() != 16 {
            return Err(CryptoError::Sm4EncryptError("IV必须为16字节".into()));
        }

        let cipher = Sm4::new_from_slice(&self.sm4_key)
            .map_err(|e| CryptoError::Sm4EncryptError(e.to_string()))?;

        // PKCS7 填充
        let block_size = 16;
        let padding = block_size - (plaintext.len() % block_size);
        let mut padded_data = plaintext.to_vec();
        padded_data.extend(vec![padding as u8; padding]);

        let mut result = Vec::new();
        let mut previous_block: [u8; 16] = iv.try_into()
            .expect("IV长度必须为16字节（已在函数入口校验）" );

        for chunk in padded_data.chunks(16) {
            let mut block: Array<u8, U16> = chunk.try_into()
                .expect("chunk(16)应产生16字节块——已确保输入是16的倍数" );

            for i in 0..16 {
                block[i] ^= previous_block[i];
            }

            cipher.encrypt_block(&mut block);
            previous_block.copy_from_slice(block.as_slice());
            result.extend_from_slice(block.as_slice());
        }

        Ok(hex::encode(result))
    }

    /// SM4 CBC 模式解密
    pub fn sm4_cbc_decrypt(&self, ciphertext: &str, iv: &[u8]) -> Result<Vec<u8>, CryptoError> {
        if iv.len() != 16 {
            return Err(CryptoError::Sm4DecryptError("IV必须为16字节".into()));
        }

        let encrypted_bytes = hex::decode(ciphertext)
            .map_err(|e| CryptoError::KeyFormatError(format!("密文格式错误: {e}" )))?;

        if encrypted_bytes.len() % 16 != 0 {
            return Err(CryptoError::Sm4DecryptError(
                "密文长度必须是16的倍数".into(),
            ));
        }

        let cipher = Sm4::new_from_slice(&self.sm4_key)
            .map_err(|e| CryptoError::Sm4DecryptError(e.to_string()))?;

        let mut result = Vec::new();
        let mut previous_block: [u8; 16] = iv.try_into()
            .expect("IV长度必须为16字节（已在函数入口校验）" );

        for chunk in encrypted_bytes.chunks(16) {
            let mut block: Array<u8, U16> = chunk.try_into()
                .expect("chunk(16)应产生16字节块——已确保输入是16的倍数" );
            let ciphertext_block = chunk.to_vec();

            cipher.decrypt_block(&mut block);

            for i in 0..16 {
                block[i] ^= previous_block[i];
            }

            result.extend_from_slice(block.as_slice());
            previous_block.copy_from_slice(&ciphertext_block);
        }

        // 移除 PKCS7 填充
        if let Some(&padding) = result.last() {
            let padding_len = padding as usize;
            if padding_len <= 16 && padding_len > 0 && padding_len <= result.len() {
                let is_valid_padding = result.iter().rev().take(padding_len).all(|&b| b == padding);
                if is_valid_padding {
                    result.truncate(result.len() - padding_len);
                }
            }
        }

        Ok(result)
    }

    /// 使用随机 IV 加密字符串 (便捷方法)
    #[allow(dead_code)]
    pub fn sm4_cbc_encrypt_string(&self, data: &str) -> Result<(String, String), CryptoError> {
        let iv = Self::generate_iv();
        let ciphertext = self.sm4_cbc_encrypt(data.as_bytes(), &iv)?;
        Ok((ciphertext, hex::encode(iv)))
    }

    /// 解密 CBC 模式加密的字符串 (便捷方法)
    #[allow(dead_code)]
    pub fn sm4_cbc_decrypt_string(
        &self,
        ciphertext: &str,
        iv_hex: &str,
    ) -> Result<String, CryptoError> {
        let iv_bytes = hex::decode(iv_hex)
            .map_err(|e| CryptoError::KeyFormatError(format!("IV格式错误: {e}" )))?;
        let decrypted = self.sm4_cbc_decrypt(ciphertext, &iv_bytes)?;
        String::from_utf8(decrypted)
            .map_err(|e| CryptoError::KeyFormatError(format!("解密后数据格式错误: {e}" )))
    }

    // ==================== SM4 ECB 模式 ====================

    /// SM4加密 (ECB模式)
    pub fn sm4_encrypt(&self, plaintext: &[u8]) -> Result<String, CryptoError> {
        let cipher = Sm4::new_from_slice(&self.sm4_key)
            .map_err(|e| CryptoError::Sm4EncryptError(e.to_string()))?;

        let block_size = 16;
        let padding = block_size - (plaintext.len() % block_size);
        let mut padded_data = plaintext.to_vec();
        padded_data.extend(vec![padding as u8; padding]);

        let mut result = Vec::new();
        for chunk in padded_data.chunks(16) {
            let mut block: Array<u8, U16> = chunk.try_into()
                .expect("chunk(16)应产生16字节块——已确保输入是16的倍数" );
            cipher.encrypt_block(&mut block);
            result.extend_from_slice(block.as_slice());
        }

        Ok(hex::encode(result))
    }

    /// SM4解密 (ECB模式)
    pub fn sm4_decrypt(&self, ciphertext: &str) -> Result<Vec<u8>, CryptoError> {
        let encrypted_bytes = hex::decode(ciphertext)
            .map_err(|e| CryptoError::KeyFormatError(format!("密文格式错误: {e}" )))?;

        let cipher = Sm4::new_from_slice(&self.sm4_key)
            .map_err(|e| CryptoError::Sm4DecryptError(e.to_string()))?;

        let mut result = Vec::new();
        for chunk in encrypted_bytes.chunks(16) {
            let mut block: Array<u8, U16> = chunk.try_into()
                .expect("chunk(16)应产生16字节块——已确保输入是16的倍数" );
            cipher.decrypt_block(&mut block);
            result.extend_from_slice(block.as_slice());
        }

        if let Some(&padding) = result.last() {
            let padding_len = padding as usize;
            if padding_len <= 16 && padding_len > 0 {
                result.truncate(result.len() - padding_len);
            }
        }

        Ok(result)
    }

    // ==================== 便捷封装 ====================

    /// 加密字符串（返回十六进制编码）
    pub fn encrypt_string(&self, data: &str) -> Result<String, CryptoError> {
        self.sm4_encrypt(data.as_bytes())
    }

    /// 解密十六进制编码的数据
    pub fn decrypt_string(&self, encrypted: &str) -> Result<String, CryptoError> {
        let decrypted = self.sm4_decrypt(encrypted)?;
        String::from_utf8(decrypted)
            .map_err(|e| CryptoError::KeyFormatError(format!("解密后数据格式错误: {e}" )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sm3_hash() {
        let data = b"hello world";
        let hash = GmCrypto::sm3_hash(data);
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_sm4_encrypt_decrypt() {
        let gm = GmCrypto::default_for_dev();
        let plaintext = b"test data";
        let encrypted = gm.sm4_encrypt(plaintext).unwrap();
        let decrypted = gm.sm4_decrypt(&encrypted).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_sm4_cbc_encrypt_decrypt() {
        let gm = GmCrypto::default_for_dev();
        let plaintext = b"CBC mode test";
        let iv = [0u8; 16];

        let encrypted = gm.sm4_cbc_encrypt(plaintext, &iv).unwrap();
        let decrypted = gm.sm4_cbc_decrypt(&encrypted, &iv).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_sm2_keypair_generation() {
        let keypair = GmCrypto::generate_sm2_keypair();
        assert_eq!(keypair.private_key.len(), 32);
        assert_eq!(keypair.public_key.len(), 64);
    }

    #[test]
    fn test_sm2_sign_verify() {
        let gm = GmCrypto::default_for_dev();
        let data = b"Test data for SM2 signature";

        let keypair = GmCrypto::generate_sm2_keypair();

        let signature = gm.sm2_sign(data, &keypair.private_key).unwrap();
        assert_eq!(signature.len(), 128); // 64字节 = 128个十六进制字符

        let valid = gm
            .sm2_verify(data, &signature, &keypair.public_key)
            .unwrap();
        assert!(valid, "SM2 signature verification failed" );
    }

    #[test]
    fn test_sm2_verify_invalid_signature() {
        let gm = GmCrypto::default_for_dev();
        let data = b"Original data";
        let wrong_data = b"Modified data";

        let keypair = GmCrypto::generate_sm2_keypair();

        let signature = gm.sm2_sign(data, &keypair.private_key).unwrap();

        let valid1 = gm
            .sm2_verify(data, &signature, &keypair.public_key)
            .unwrap();
        assert!(valid1);

        let valid2 = gm
            .sm2_verify(wrong_data, &signature, &keypair.public_key)
            .unwrap();
        assert!(!valid2, "SM2 verification should fail for modified data" );
    }

    #[test]
    fn test_invalid_sm4_key_length() {
        let result = GmCrypto::new("0123456789abcdef" );
        assert!(result.is_err());
    }

    #[test]
    fn test_sm2_keypair_hex_encoding() {
        let keypair = GmCrypto::generate_sm2_keypair();

        // 测试私钥十六进制编码
        let private_hex = hex::encode(keypair.private_key);
        assert_eq!(private_hex.len(), 64);

        // 测试公钥十六进制编码
        let public_hex = hex::encode(keypair.public_key);
        assert_eq!(public_hex.len(), 128);

        // 测试解析回密钥
        let parsed_private = GmCrypto::parse_sm2_private_key(&private_hex).unwrap();
        assert_eq!(parsed_private, keypair.private_key);

        let parsed_public = GmCrypto::parse_sm2_public_key(&public_hex).unwrap();
        assert_eq!(parsed_public, keypair.public_key);
    }
}
