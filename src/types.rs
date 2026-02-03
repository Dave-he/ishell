use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// SSH 认证方法
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMethod {
    /// 密码认证
    Password(String),
    /// 密钥认证（私钥路径，可选密码）
    PrivateKey {
        key_path: PathBuf,
        passphrase: Option<String>,
    },
}

/// SSH 连接配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshConfig {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    #[serde(skip)] // 不序列化，从加密字段读取
    pub auth: Option<AuthMethod>,
    /// 加密后的密码（base64 编码）
    pub password_encrypted: Option<String>,
    /// SSH 私钥路径
    pub key_path: Option<String>,
    /// 私钥密码（加密）
    pub key_passphrase_encrypted: Option<String>,
}

impl SshConfig {
    pub fn new(name: String, host: String, port: u16, username: String) -> Self {
        Self {
            name,
            host,
            port,
            username,
            auth: None,
            password_encrypted: None,
            key_path: None,
            key_passphrase_encrypted: None,
        }
    }
}

/// AI 提供商类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AiProviderType {
    Ollama,
    OpenAI,
    Google,
}

impl std::fmt::Display for AiProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AiProviderType::Ollama => write!(f, "Ollama"),
            AiProviderType::OpenAI => write!(f, "OpenAI"),
            AiProviderType::Google => write!(f, "Google"),
        }
    }
}

/// AI 消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiMessage {
    pub role: String, // "user" or "assistant"
    pub content: String,
}

impl AiMessage {
    pub fn user(content: String) -> Self {
        Self {
            role: "user".to_string(),
            content,
        }
    }

    pub fn assistant(content: String) -> Self {
        Self {
            role: "assistant".to_string(),
            content,
        }
    }
}

/// Ollama 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    pub enabled: bool,
    pub base_url: String,
    pub model: String,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            base_url: "http://localhost:11434".to_string(),
            model: "llama3.2".to_string(),
        }
    }
}

/// OpenAI 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiConfig {
    pub enabled: bool,
    #[serde(skip)]
    pub api_key: Option<String>,
    pub api_key_encrypted: Option<String>,
    pub model: String,
}

impl Default for OpenAiConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            api_key: None,
            api_key_encrypted: None,
            model: "gpt-4o-mini".to_string(),
        }
    }
}

/// Google AI 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleConfig {
    pub enabled: bool,
    #[serde(skip)]
    pub api_key: Option<String>,
    pub api_key_encrypted: Option<String>,
    pub model: String,
}

impl Default for GoogleConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            api_key: None,
            api_key_encrypted: None,
            model: "gemini-1.5-flash".to_string(),
        }
    }
}

/// AI 配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AiConfig {
    pub ollama: OllamaConfig,
    pub openai: OpenAiConfig,
    pub google: GoogleConfig,
}

/// 应用设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    // AI 设置
    pub default_ai_provider: AiProviderType,
    
    // 外观设置
    pub theme: String,
    pub font_size: f32,
    pub terminal_font_size: f32,
    
    // 行为设置
    pub auto_save_config: bool,
    pub confirm_before_delete: bool,
    
    // 终端设置
    pub terminal_scrollback: usize,
    pub terminal_word_wrap: bool,
    
    // 历史设置
    pub history_max_size: usize,
    pub save_history_on_exit: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            default_ai_provider: AiProviderType::Ollama,
            theme: "dark".to_string(),
            font_size: 14.0,
            terminal_font_size: 14.0,
            auto_save_config: true,
            confirm_before_delete: true,
            terminal_scrollback: 10000,
            terminal_word_wrap: false,
            history_max_size: 1000,
            save_history_on_exit: true,
        }
    }
}

/// 应用配置（完整配置结构）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub version: String,
    pub connections: Vec<SshConfig>,
    pub ai: AiConfig,
    pub settings: Settings,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: "0.2.0".to_string(),
            connections: Vec::new(),
            ai: AiConfig::default(),
            settings: Settings::default(),
        }
    }
}

/// SSH 连接状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Error,
}

/// 操作结果
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

// ============================================================================
// 设置界面 (v0.3.0)
// ============================================================================

/// 设置页面
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsPage {
    General,
    Appearance,
    Terminal,
    Ai,
    History,
}

// ============================================================================
// SFTP 类型 (v0.3.0)
// ============================================================================

use std::time::SystemTime;

/// 文件条目（本地或远程）
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: Option<SystemTime>,
    pub permissions: Option<String>,
}

impl FileEntry {
    pub fn new(name: String, path: String, is_dir: bool) -> Self {
        Self {
            name,
            path,
            is_dir,
            size: 0,
            modified: None,
            permissions: None,
        }
    }
}

/// SFTP 操作
#[derive(Debug, Clone)]
pub enum SftpOperation {
    Upload {
        local_path: PathBuf,
        remote_path: String,
    },
    Download {
        remote_path: String,
        local_path: PathBuf,
    },
    List {
        path: String,
    },
    Delete {
        path: String,
    },
    CreateDir {
        path: String,
    },
}

/// SFTP 消息（后台 -> UI）
#[derive(Debug, Clone)]
pub enum SftpMessage {
    FileList(Vec<FileEntry>),
    Progress(f32),
    Complete,
    Error(String),
}
