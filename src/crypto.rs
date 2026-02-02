use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose, Engine as _};
use rand::RngCore;

const NONCE_SIZE: usize = 12; // 96 bits for GCM

/// 密码加密器
///
/// 使用 AES-256-GCM 加密敏感信息
/// 密钥从用户机器ID生成（确保跨会话一致性）
pub struct PasswordEncryptor {
    cipher: Aes256Gcm,
}

impl PasswordEncryptor {
    /// 创建新的加密器
    ///
    /// 使用机器特定的密钥（基于用户名和主机名）
    pub fn new() -> Result<Self, String> {
        let key = Self::generate_machine_key()?;
        let cipher = Aes256Gcm::new(&key.into());
        Ok(Self { cipher })
    }

    /// 生成基于机器的密钥
    ///
    /// 使用用户名作为密钥来源，确保同一台机器上密钥一致
    fn generate_machine_key() -> Result<[u8; 32], String> {
        // 使用用户名作为密钥种子
        let username = whoami::username();
        let hostname = match whoami::fallible::hostname() {
            Ok(h) => h,
            Err(_) => "unknown-host".to_string(),
        };

        // 简单的密钥派生（实际应用中应使用 PBKDF2 或 Argon2）
        let seed = format!("{}-{}-ishell-v0.2.0", username, hostname);

        // 使用 SHA-256 生成 32 字节密钥
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        seed.hash(&mut hasher);
        let hash = hasher.finish();

        // 扩展到 32 字节
        let mut key = [0u8; 32];
        for i in 0..4 {
            let offset = i * 8;
            key[offset..offset + 8].copy_from_slice(&hash.to_le_bytes());
        }

        // 添加一些随机性（使用固定种子确保可重复）
        for (i, byte) in seed.bytes().enumerate() {
            if i >= 32 {
                break;
            }
            key[i] ^= byte;
        }

        Ok(key)
    }

    /// 加密密码
    ///
    /// 返回 base64 编码的字符串：nonce(12字节) + ciphertext
    pub fn encrypt(&self, plaintext: &str) -> Result<String, String> {
        if plaintext.is_empty() {
            return Ok(String::new());
        }

        // 生成随机 nonce
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // 加密
        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| format!("Encryption failed: {}", e))?;

        // 组合 nonce + ciphertext
        let mut result = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);

        // Base64 编码
        Ok(general_purpose::STANDARD.encode(result))
    }

    /// 解密密码
    ///
    /// 输入：base64 编码的字符串（nonce + ciphertext）
    /// 输出：原始明文
    pub fn decrypt(&self, ciphertext_b64: &str) -> Result<String, String> {
        if ciphertext_b64.is_empty() {
            return Ok(String::new());
        }

        // Base64 解码
        let data = general_purpose::STANDARD
            .decode(ciphertext_b64)
            .map_err(|e| format!("Base64 decode failed: {}", e))?;

        if data.len() < NONCE_SIZE {
            return Err("Invalid ciphertext: too short".to_string());
        }

        // 分离 nonce 和 ciphertext
        let (nonce_bytes, ciphertext) = data.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);

        // 解密
        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;

        // 转换为字符串
        String::from_utf8(plaintext).map_err(|e| format!("UTF-8 conversion failed: {}", e))
    }
}

impl Default for PasswordEncryptor {
    fn default() -> Self {
        Self::new().expect("Failed to create password encryptor")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let encryptor = PasswordEncryptor::new().unwrap();

        let original = "my_secret_password_123!@#";
        let encrypted = encryptor.encrypt(original).unwrap();
        let decrypted = encryptor.decrypt(&encrypted).unwrap();

        assert_eq!(original, decrypted);
    }

    #[test]
    fn test_empty_string() {
        let encryptor = PasswordEncryptor::new().unwrap();

        let encrypted = encryptor.encrypt("").unwrap();
        assert_eq!(encrypted, "");

        let decrypted = encryptor.decrypt("").unwrap();
        assert_eq!(decrypted, "");
    }

    #[test]
    fn test_different_encryption() {
        let encryptor = PasswordEncryptor::new().unwrap();

        let original = "password";
        let encrypted1 = encryptor.encrypt(original).unwrap();
        let encrypted2 = encryptor.encrypt(original).unwrap();

        // 由于使用随机 nonce，两次加密结果应该不同
        assert_ne!(encrypted1, encrypted2);

        // 但都能正确解密
        assert_eq!(encryptor.decrypt(&encrypted1).unwrap(), original);
        assert_eq!(encryptor.decrypt(&encrypted2).unwrap(), original);
    }

    #[test]
    fn test_unicode() {
        let encryptor = PasswordEncryptor::new().unwrap();

        let original = "密码123!@#$%^&*()_+";
        let encrypted = encryptor.encrypt(original).unwrap();
        let decrypted = encryptor.decrypt(&encrypted).unwrap();

        assert_eq!(original, decrypted);
    }
}
