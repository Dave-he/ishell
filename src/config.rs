use crate::crypto::PasswordEncryptor;
use crate::types::{AppConfig, AuthMethod, Result, SshConfig};
use std::fs;
use std::path::PathBuf;

/// 配置管理器
pub struct ConfigManager {
    config_path: PathBuf,
    encryptor: PasswordEncryptor,
}

impl ConfigManager {
    /// 创建新的配置管理器
    pub fn new() -> Result<Self> {
        let config_dir = Self::get_config_dir()?;
        fs::create_dir_all(&config_dir)?;

        let config_path = config_dir.join("config.toml");
        let encryptor = PasswordEncryptor::new()?;

        Ok(Self {
            config_path,
            encryptor,
        })
    }

    /// 创建新的配置管理器（指定路径）
    pub fn new_with_path(config_path: PathBuf) -> Result<Self> {
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let encryptor = PasswordEncryptor::new()?;

        Ok(Self {
            config_path,
            encryptor,
        })
    }

    /// 获取配置目录路径
    fn get_config_dir() -> Result<PathBuf> {
        let home_dir = dirs::home_dir().ok_or("Cannot determine home directory")?;
        Ok(home_dir.join(".ishell"))
    }

    /// 加载配置
    pub fn load_config(&self) -> Result<AppConfig> {
        if !self.config_path.exists() {
            // 如果配置文件不存在，返回默认配置
            return Ok(AppConfig::default());
        }

        let content = fs::read_to_string(&self.config_path)?;
        let mut config: AppConfig = toml::from_str(&content)?;

        // 解密所有连接的密码
        for conn in &mut config.connections {
            self.decrypt_connection(conn)?;
        }

        // 解密 AI API 密钥
        if let Some(encrypted) = &config.ai.openai.api_key_encrypted {
            config.ai.openai.api_key = Some(self.encryptor.decrypt(encrypted)?);
        }

        if let Some(encrypted) = &config.ai.google.api_key_encrypted {
            config.ai.google.api_key = Some(self.encryptor.decrypt(encrypted)?);
        }

        Ok(config)
    }

    /// 保存配置
    pub fn save_config(&self, config: &mut AppConfig) -> Result<()> {
        // 加密所有连接的密码
        for conn in &mut config.connections {
            self.encrypt_connection(conn)?;
        }

        // 加密 AI API 密钥
        if let Some(api_key) = &config.ai.openai.api_key {
            if !api_key.is_empty() {
                config.ai.openai.api_key_encrypted = Some(self.encryptor.encrypt(api_key)?);
            }
        }

        if let Some(api_key) = &config.ai.google.api_key {
            if !api_key.is_empty() {
                config.ai.google.api_key_encrypted = Some(self.encryptor.encrypt(api_key)?);
            }
        }

        // 序列化为 TOML
        let content = toml::to_string_pretty(config)?;

        // 写入文件
        fs::write(&self.config_path, content)?;

        Ok(())
    }

    /// 加密连接配置
    pub fn encrypt_connection(&self, conn: &mut SshConfig) -> Result<()> {
        if let Some(auth) = &conn.auth {
            match auth {
                AuthMethod::Password(password) => {
                    if !password.is_empty() {
                        conn.password_encrypted = Some(self.encryptor.encrypt(password)?);
                    }
                }
                AuthMethod::PrivateKey {
                    key_path,
                    passphrase,
                } => {
                    conn.key_path = Some(key_path.to_string_lossy().to_string());
                    if let Some(pass) = passphrase {
                        if !pass.is_empty() {
                            conn.key_passphrase_encrypted = Some(self.encryptor.encrypt(pass)?);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// 解密连接配置
    pub fn decrypt_connection(&self, conn: &mut SshConfig) -> Result<()> {
        // 尝试解密密码
        if let Some(encrypted) = &conn.password_encrypted {
            let password = self.encryptor.decrypt(encrypted)?;
            conn.auth = Some(AuthMethod::Password(password));
            return Ok(());
        }

        // 尝试解密密钥
        if let Some(key_path_str) = &conn.key_path {
            let key_path = PathBuf::from(key_path_str);
            let passphrase = if let Some(encrypted) = &conn.key_passphrase_encrypted {
                Some(self.encryptor.decrypt(encrypted)?)
            } else {
                None
            };

            conn.auth = Some(AuthMethod::PrivateKey {
                key_path,
                passphrase,
            });
        }

        Ok(())
    }

    /// 添加连接配置
    pub fn add_connection(&self, config: &mut AppConfig, conn: SshConfig) -> Result<()> {
        config.connections.push(conn);
        self.save_config(config)?;
        Ok(())
    }

    /// 删除连接配置
    pub fn remove_connection(&self, config: &mut AppConfig, index: usize) -> Result<()> {
        if index < config.connections.len() {
            config.connections.remove(index);
            self.save_config(config)?;
        }
        Ok(())
    }

    /// 更新连接配置
    pub fn update_connection(
        &self,
        config: &mut AppConfig,
        index: usize,
        conn: SshConfig,
    ) -> Result<()> {
        if index < config.connections.len() {
            config.connections[index] = conn;
            self.save_config(config)?;
        }
        Ok(())
    }

    /// 获取配置文件路径
    pub fn config_path(&self) -> &PathBuf {
        &self.config_path
    }

    /// 导出配置（不含敏感信息）
    pub fn export_config_safe(&self, config: &AppConfig) -> Result<String> {
        let mut safe_config = config.clone();

        // 清除所有加密字段
        for conn in &mut safe_config.connections {
            conn.password_encrypted = None;
            conn.key_passphrase_encrypted = None;
            conn.auth = None;
        }

        safe_config.ai.openai.api_key = None;
        safe_config.ai.openai.api_key_encrypted = None;
        safe_config.ai.google.api_key = None;
        safe_config.ai.google.api_key_encrypted = None;

        Ok(toml::to_string_pretty(&safe_config)?)
    }

    /// 备份配置文件
    pub fn backup_config(&self) -> Result<PathBuf> {
        if !self.config_path.exists() {
            return Err("Config file does not exist".into());
        }

        let backup_name = format!(
            "config.backup.{}.toml",
            chrono::Local::now().format("%Y%m%d_%H%M%S")
        );

        let backup_path = self
            .config_path
            .parent()
            .ok_or("Invalid config path")?
            .join(backup_name);

        fs::copy(&self.config_path, &backup_path)?;
        Ok(backup_path)
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new().expect("Failed to create config manager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use rand::Rng;

    fn get_test_manager() -> ConfigManager {
        let temp_dir = std::env::temp_dir().join("ishell_config_test");
        let rand_val: u32 = rand::thread_rng().gen();
        let config_path = temp_dir.join(format!("config_{}.toml", rand_val));

        // Ensure parent exists
        std::fs::create_dir_all(&temp_dir).unwrap();

        ConfigManager::new_with_path(config_path).unwrap()
    }

    #[test]
    fn test_config_dir() {
        // This relies on system environment, keep as is or ignore
        if let Ok(config_dir) = ConfigManager::get_config_dir() {
            assert!(config_dir.ends_with(".ishell"));
        }
    }

    #[test]
    fn test_config_manager_creation() {
        let _manager = get_test_manager();
        // Implicitly checked by creation
    }

    #[test]
    fn test_load_default_config() {
        let manager = get_test_manager();
        // 如果文件不存在，应返回默认配置
        let config = manager.load_config();
        assert!(config.is_ok());
    }

    #[test]
    fn test_encryption_roundtrip() {
        let manager = get_test_manager();
        let mut conn = SshConfig::new(
            "test".to_string(),
            "localhost".to_string(),
            22,
            "user".to_string(),
        );
        conn.auth = Some(AuthMethod::Password("secret123".to_string()));

        // 加密
        manager.encrypt_connection(&mut conn).unwrap();
        assert!(conn.password_encrypted.is_some());

        // 解密
        manager.decrypt_connection(&mut conn).unwrap();
        if let Some(AuthMethod::Password(pass)) = &conn.auth {
            assert_eq!(pass, "secret123");
        } else {
            panic!("Expected password auth");
        }
    }
}
