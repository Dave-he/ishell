use crate::ai::AiManager;
use crate::config::ConfigManager;
use crate::history::CommandHistory;
use crate::monitor::SystemMonitor;
use crate::ssh::SshSession;
use crate::types::{AiProviderType, AppConfig, ConnectionStatus, FileEntry, SettingsPage, SftpMessage, SshConfig};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex as TokioMutex};

// ============================================================================
// 消息类型 - 用于异步通信
// ============================================================================

#[derive(Debug, Clone)]
pub enum SshMessage {
    Connected(String),
    Disconnected,
    Output(String),
    Error(String),
}

#[derive(Debug, Clone)]
pub enum SshCommand {
    Connect { config: SshConfig },
    Disconnect,
    ExecuteCommand(String),
}

#[derive(Debug, Clone)]
pub enum AiChannelMessage {
    Response(String),
    Error(String),
}

// ============================================================================
// 主应用状态结构
// ============================================================================

pub struct AppState {
    // 配置管理
    pub config_manager: ConfigManager,
    pub config: AppConfig,

    // 连接管理
    pub connections: Vec<SshConfig>,
    pub selected_connection: Option<usize>,
    pub show_new_connection: bool,

    // 新建连接表单
    pub new_conn_name: String,
    pub new_conn_host: String,
    pub new_conn_port: String,
    pub new_conn_user: String,
    pub new_conn_password: String,
    pub new_conn_use_key: bool,
    pub new_conn_key_path: String,

    // SSH 状态
    pub ssh_sessions: Vec<Option<Arc<std::sync::Mutex<SshSession>>>>,
    pub connection_status: Vec<ConnectionStatus>,

    // 终端
    pub terminal_output: String,
    pub command_input: String,

    // SSH 异步通信
    pub ssh_msg_tx: mpsc::UnboundedSender<SshMessage>, // 后台->UI
    pub ssh_msg_rx: Arc<std::sync::Mutex<mpsc::UnboundedReceiver<SshMessage>>>,

    // AI 配置和状态
    pub ai_manager: Arc<TokioMutex<Option<AiManager>>>,
    pub ai_messages: Vec<(String, String)>,
    pub ai_input: String,
    pub ai_provider: AiProviderType,
    pub ai_loading: bool,

    // AI 异步通信
    pub ai_msg_tx: mpsc::UnboundedSender<AiChannelMessage>, // 后台->UI
    pub ai_msg_rx: Arc<std::sync::Mutex<mpsc::UnboundedReceiver<AiChannelMessage>>>,

    // SFTP 文件浏览器 (v0.3.0)
    pub show_file_browser: bool,
    pub remote_current_path: String,
    pub local_current_path: std::path::PathBuf,
    pub remote_files: Vec<FileEntry>,
    pub local_files: Vec<FileEntry>,
    pub selected_remote_files: Vec<String>,
    pub selected_local_file: Option<std::path::PathBuf>,
    pub sftp_progress: f32,
    pub sftp_status: String,

    // SFTP 异步通信 (v0.3.0)
    pub sftp_msg_tx: mpsc::UnboundedSender<SftpMessage>,
    pub sftp_msg_rx: Arc<std::sync::Mutex<mpsc::UnboundedReceiver<SftpMessage>>>,

    // 系统监控 (v0.3.0)
    pub system_monitor: Arc<SystemMonitor>,
    pub cpu_usage: f32,
    pub mem_usage: f32,

    // 命令历史 (v0.3.0)
    pub command_history: CommandHistory,
    pub show_history_search: bool,
    pub history_search_query: String,

    // 设置界面 (v0.3.0)
    pub show_settings: bool,
    pub settings_page: SettingsPage,

    // Tokio 运行时
    pub runtime: Arc<tokio::runtime::Runtime>,
}
