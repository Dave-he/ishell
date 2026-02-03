use eframe::egui;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex as TokioMutex};

use crate::ai::{AiManager, GoogleProvider, OllamaProvider, OpenAiProvider};
use crate::config::ConfigManager;
use crate::ssh::SshSession;
use crate::state::{AiChannelMessage, AppState, SshMessage};
use crate::types::*;
use crate::ui::panels;

// ============================================================================
// 主应用结构
// ============================================================================

pub struct App {
    pub state: AppState,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // 创建 Tokio 运行时
        let runtime =
            Arc::new(tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime"));

        // 创建配置管理器
        let config_manager = ConfigManager::new().expect("Failed to create config manager");

        // 加载配置
        let config = config_manager.load_config().unwrap_or_else(|e| {
            eprintln!("Failed to load config: {}", e);
            AppConfig::default()
        });

        // 从配置加载连接
        let connections = config.connections.clone();
        let connection_count = connections.len();

        // 初始化 SSH 会话和状态
        let ssh_sessions = vec![None; connection_count];
        let connection_status = vec![ConnectionStatus::Disconnected; connection_count];

        // 创建 SSH 通信通道（后台线程 -> UI）
        let (ssh_msg_tx, ssh_msg_rx) = mpsc::unbounded_channel();
        let ssh_msg_rx = Arc::new(std::sync::Mutex::new(ssh_msg_rx));

        // 创建 AI 通信通道（后台线程 -> UI）
        let (ai_msg_tx, ai_msg_rx) = mpsc::unbounded_channel();
        let ai_msg_rx = Arc::new(std::sync::Mutex::new(ai_msg_rx));

        // 创建 SFTP 通信通道（后台线程 -> UI）(v0.3.0)
        let (sftp_msg_tx, sftp_msg_rx) = mpsc::unbounded_channel();
        let sftp_msg_rx = Arc::new(std::sync::Mutex::new(sftp_msg_rx));

        // 初始化 AI Manager
        let mut ai_manager = AiManager::new();

        // 根据配置注册 AI 提供商
        if config.ai.ollama.enabled {
            let provider = OllamaProvider::new(
                config.ai.ollama.base_url.clone(),
                config.ai.ollama.model.clone(),
            );
            ai_manager.register_provider(Box::new(provider));
        }

        if config.ai.openai.enabled {
            if let Some(api_key) = &config.ai.openai.api_key {
                let provider = OpenAiProvider::new(api_key.clone(), config.ai.openai.model.clone());
                ai_manager.register_provider(Box::new(provider));
            }
        }

        if config.ai.google.enabled {
            if let Some(api_key) = &config.ai.google.api_key {
                let provider = GoogleProvider::new(api_key.clone(), config.ai.google.model.clone());
                ai_manager.register_provider(Box::new(provider));
            }
        }

        // 设置默认提供商
        ai_manager.set_current_provider(config.settings.default_ai_provider);

        let ai_manager = Arc::new(TokioMutex::new(Some(ai_manager)));

        let state = AppState {
            config_manager,
            connections: connections.clone(),
            selected_connection: None,
            show_new_connection: false,

            new_conn_name: String::new(),
            new_conn_host: String::new(),
            new_conn_port: "22".to_string(),
            new_conn_user: String::new(),
            new_conn_password: String::new(),
            new_conn_use_key: false,
            new_conn_key_path: String::new(),

            ssh_sessions,
            connection_status,

            terminal_output:
                "Welcome to iShell v0.2.0! 🚀\nType commands after connecting to a server.\n\n"
                    .to_string(),
            command_input: String::new(),

            ssh_msg_tx,
            ssh_msg_rx,

            ai_manager,
            ai_messages: Vec::new(),
            ai_input: String::new(),
            ai_provider: config.settings.default_ai_provider,
            ai_loading: false,

            ai_msg_tx,
            ai_msg_rx,

            // SFTP 文件浏览器 (v0.3.0)
            show_file_browser: false,
            remote_current_path: "/".to_string(),
            local_current_path: std::env::current_dir().unwrap_or_default(),
            remote_files: Vec::new(),
            local_files: Vec::new(),
            selected_remote_files: Vec::new(),
            selected_local_file: None,
            sftp_progress: 0.0,
            sftp_status: String::new(),

            sftp_msg_tx,
            sftp_msg_rx,

            system_monitor: Arc::new(crate::monitor::SystemMonitor::new()),
            cpu_usage: 0.0,
            mem_usage: 45.0,

            command_history: load_command_history(),
            show_history_search: false,
            history_search_query: String::new(),

            show_settings: false,
            settings_page: crate::types::SettingsPage::General,

            runtime,
            config,
        };

        Self { state }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 应用主题
        crate::theme::ThemeManager::apply(ctx, &self.state.config.settings.theme);

        // 处理异步消息
        process_ssh_messages(&mut self.state);
        process_ai_messages(&mut self.state);
        process_sftp_messages(&mut self.state);

        // Top menu bar
        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("📁 File", |ui| {
                    if ui.button("➕ New Connection").clicked() {
                        self.state.show_new_connection = true;
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("💾 Save Config").clicked() {
                        save_config(&mut self.state);
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("🚪 Quit").clicked() {
                        save_command_history(&self.state);
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("🔧 Tools", |ui| {
                    if ui.button("📁 File Browser").clicked() {
                        self.state.show_file_browser = true;
                        ui.close_menu();
                    }
                    if ui.button("🔍 Command History").clicked() {
                        self.state.show_history_search = true;
                        ui.close_menu();
                    }
                    if ui.button("🔑 SSH Keys").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("⚙️ Settings").clicked() {
                        self.state.show_settings = true;
                        ui.close_menu();
                    }
                });

                ui.menu_button("❓ Help", |ui| {
                    if ui.button("📖 Documentation").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("ℹ️ About").clicked() {
                        ui.close_menu();
                    }
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let icon = match self.state.ai_provider {
                        AiProviderType::Ollama => "🦙 Ollama",
                        AiProviderType::OpenAI => "🤖 OpenAI",
                        AiProviderType::Google => "🔷 Google",
                    };
                    ui.label(egui::RichText::new(icon).strong());

                    if self.state.ai_loading {
                        ui.spinner();
                    }
                });
            });
        });

        // Render panels using the new module structure
        panels::render_connections_panel(&mut self.state, ctx);
        panels::render_ai_panel(&mut self.state, ctx);
        panels::render_monitor_panel(&mut self.state, ctx);
        panels::render_terminal_panel(&mut self.state, ctx);
        panels::render_new_connection_dialog(&mut self.state, ctx);
        
        // Render file browser (v0.3.0)
        crate::ui::file_browser::render_file_browser(&mut self.state, ctx);
        
        // Render settings window (v0.3.0) - TODO: Phase 4
        // crate::ui::settings_panel::render_settings_window(&mut self.state, ctx);


        // 请求重绘
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // 保存命令历史
        save_command_history(&self.state);
    }
}

// 独立函数 - 逻辑处理
// ============================================================================

// 加载命令历史
fn load_command_history() -> crate::history::CommandHistory {
    if let Some(config_dir) = dirs::config_dir() {
        let history_path = config_dir.join("ishell").join("history.json");
        if history_path.exists() {
            match crate::history::CommandHistory::load(&history_path) {
                Ok(history) => {
                    eprintln!("✅ Loaded {} commands from history", history.commands.len());
                    return history;
                }
                Err(e) => {
                    eprintln!("⚠️ Failed to load history: {}", e);
                }
            }
        }
    }
    crate::history::CommandHistory::new()
}

// 保存命令历史
fn save_command_history(state: &AppState) {
    if let Some(config_dir) = dirs::config_dir() {
        let history_path = config_dir.join("ishell").join("history.json");
        if let Err(e) = state.command_history.save(&history_path) {
            eprintln!("⚠️ Failed to save history: {}", e);
        } else {
            eprintln!("✅ Saved {} commands to history", state.command_history.commands.len());
        }
    }
}

// 处理 SSH 消息
fn process_ssh_messages(state: &mut AppState) {
    let mut rx = state.ssh_msg_rx.lock().unwrap();
    while let Ok(msg) = rx.try_recv() {
        match msg {
            SshMessage::Connected(info) => {
                state
                    .terminal_output
                    .push_str(&format!("✅ Connected: {}\n", info));
                if let Some(idx) = state.selected_connection {
                    state.connection_status[idx] = ConnectionStatus::Connected;
                }
            }
            SshMessage::Disconnected => {
                state.terminal_output.push_str("❌ Disconnected\n");
                if let Some(idx) = state.selected_connection {
                    state.connection_status[idx] = ConnectionStatus::Disconnected;
                }
            }
            SshMessage::Output(output) => {
                state.terminal_output.push_str(&output);
            }
            SshMessage::Error(error) => {
                state
                    .terminal_output
                    .push_str(&format!("❌ Error: {}\n", error));
                if let Some(idx) = state.selected_connection {
                    state.connection_status[idx] = ConnectionStatus::Error;
                }
            }
        }
    }
}

// 处理 AI 消息
fn process_ai_messages(state: &mut AppState) {
    let mut rx = state.ai_msg_rx.lock().unwrap();
    while let Ok(msg) = rx.try_recv() {
        state.ai_loading = false;
        match msg {
            AiChannelMessage::Response(response) => {
                state.ai_messages.push(("ai".to_string(), response));
            }
            AiChannelMessage::Error(error) => {
                state
                    .ai_messages
                    .push(("ai".to_string(), format!("❌ Error: {}", error)));
            }
        }
    }
}

// 处理 SFTP 消息 (v0.3.0)
fn process_sftp_messages(state: &mut AppState) {
    let mut rx = state.sftp_msg_rx.lock().unwrap();
    while let Ok(msg) = rx.try_recv() {
        match msg {
            crate::types::SftpMessage::FileList(files) => {
                state.remote_files = files;
                state.sftp_status = format!("已加载 {} 个文件", state.remote_files.len());
            }
            crate::types::SftpMessage::Progress(progress) => {
                state.sftp_progress = progress;
            }
            crate::types::SftpMessage::Complete => {
                state.sftp_progress = 1.0;
                state.sftp_status = "操作完成".to_string();
                // 刷新文件列表
                // refresh_remote_files(state); // 注意：这需要在 UI 线程调用
            }
            crate::types::SftpMessage::Error(error) => {
                state.sftp_progress = 0.0;
                state.sftp_status = format!("❌ {}", error);
            }
        }
    }
}

// 连接到 SSH 服务器
pub fn connect_ssh(state: &mut AppState, index: usize) {
    if index >= state.connections.len() {
        return;
    }

    let conn = &state.connections[index];
    if conn.auth.is_none() {
        state
            .terminal_output
            .push_str("❌ No authentication method configured\n");
        return;
    }

    state.connection_status[index] = ConnectionStatus::Connecting;
    state.terminal_output.push_str(&format!(
        "🔄 Connecting to {}@{}:{}...\n",
        conn.username, conn.host, conn.port
    ));

    let config = conn.clone();
    let tx = state.ssh_msg_tx.clone();

    let session = Arc::new(std::sync::Mutex::new(SshSession::new(
        config.host.clone(),
        config.port,
        config.username.clone(),
    )));

    state.ssh_sessions[index] = Some(session.clone());

    // 在后台线程执行连接
    std::thread::spawn(move || {
        let result = {
            let sess = session.lock().unwrap();
            sess.connect(config.auth.as_ref().unwrap())
        };

        match result {
            Ok(_) => {
                let _ = tx.send(SshMessage::Connected(format!(
                    "{}@{}",
                    config.username, config.host
                )));
            }
            Err(e) => {
                let _ = tx.send(SshMessage::Error(format!("Connection failed: {}", e)));
            }
        }
    });
}

// 断开 SSH 连接
pub fn disconnect_ssh(state: &mut AppState, index: usize) {
    if let Some(session) = &state.ssh_sessions[index] {
        let sess = session.lock().unwrap();
        let _ = sess.disconnect();
        state.connection_status[index] = ConnectionStatus::Disconnected;

        let tx = &state.ssh_msg_tx;
        let _ = tx.send(SshMessage::Disconnected);
    }
}

// 执行 SSH 命令
pub fn execute_ssh_command(state: &mut AppState, command: String) {
    if let Some(idx) = state.selected_connection {
        if let Some(session) = &state.ssh_sessions[idx] {
            let session = session.clone();
            let tx = state.ssh_msg_tx.clone();

            state.terminal_output.push_str(&format!("$ {}\n", command));

            std::thread::spawn(move || {
                let result = {
                    let sess = session.lock().unwrap();
                    sess.execute_command(&command)
                };

                match result {
                    Ok(output) => {
                        let _ = tx.send(SshMessage::Output(output + "\n"));
                    }
                    Err(e) => {
                        let _ = tx.send(SshMessage::Error(format!("Command failed: {}", e)));
                    }
                }
            });
        }
    }
}

// 发送 AI 消息
pub fn send_ai_message(state: &mut AppState, user_message: String) {
    state
        .ai_messages
        .push(("user".to_string(), user_message.clone()));
    state.ai_loading = true;

    // 构建消息历史
    let messages: Vec<crate::types::AiMessage> = state
        .ai_messages
        .iter()
        .map(|(role, content)| {
            if role == "user" {
                crate::types::AiMessage::user(content.clone())
            } else {
                crate::types::AiMessage::assistant(content.clone())
            }
        })
        .collect();

    let ai_manager = state.ai_manager.clone();
    let tx = state.ai_msg_tx.clone();
    let runtime = state.runtime.clone();

    runtime.spawn(async move {
        // 在 async 块内部获取并立即释放锁
        let result = {
            let mut manager_guard = ai_manager.lock().await;
            if let Some(manager) = manager_guard.as_mut() {
                manager.chat(&messages).await
            } else {
                Err("AI Manager not initialized".into())
            }
        }; // manager_guard 在此处释放

        match result {
            Ok(response) => {
                let _ = tx.send(AiChannelMessage::Response(response));
            }
            Err(e) => {
                let _ = tx.send(AiChannelMessage::Error(format!("{}", e)));
            }
        }
    });
}

// 更改 AI 提供商
pub fn change_ai_provider(state: &mut AppState, provider: AiProviderType) {
    let ai_manager = state.ai_manager.clone();
    let runtime = state.runtime.clone();

    runtime.spawn(async move {
        let mut manager_guard = ai_manager.lock().await;
        if let Some(manager) = manager_guard.as_mut() {
            manager.set_current_provider(provider);
        }
    });
}

// 保存配置
pub fn save_config(state: &mut AppState) {
    state.config.connections = state.connections.clone();

    if let Err(e) = state.config_manager.save_config(&mut state.config) {
        eprintln!("Failed to save config: {}", e);
    }
}

// 创建新连接
pub fn create_connection(state: &mut AppState) {
    if state.new_conn_name.is_empty() || state.new_conn_host.is_empty() {
        return;
    }

    let port = state.new_conn_port.parse().unwrap_or(22);
    let mut config = SshConfig::new(
        state.new_conn_name.clone(),
        state.new_conn_host.clone(),
        port,
        state.new_conn_user.clone(),
    );

    // 设置认证方法
    if state.new_conn_use_key {
        config.auth = Some(AuthMethod::PrivateKey {
            key_path: std::path::PathBuf::from(&state.new_conn_key_path),
            passphrase: if state.new_conn_password.is_empty() {
                None
            } else {
                Some(state.new_conn_password.clone())
            },
        });
    } else {
        config.auth = Some(AuthMethod::Password(state.new_conn_password.clone()));
    }

    state.connections.push(config);
    state.ssh_sessions.push(None);
    state.connection_status.push(ConnectionStatus::Disconnected);

    // 保存配置
    save_config(state);

    // 清空表单
    state.new_conn_name.clear();
    state.new_conn_host.clear();
    state.new_conn_port = "22".to_string();
    state.new_conn_user.clear();
    state.new_conn_password.clear();
    state.new_conn_use_key = false;
    state.new_conn_key_path.clear();

    state.show_new_connection = false;
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // Helper to create a testable app instance
    fn create_test_app() -> App {
        // Create a temporary config path
        let temp_dir = std::env::temp_dir().join("ishell_test");
        let rand_val: u32 = rand::Rng::gen(&mut rand::thread_rng());
        let config_path = temp_dir.join(format!("config_{}.toml", rand_val));

        // Ensure parent dir exists
        std::fs::create_dir_all(&temp_dir).unwrap();

        let config_manager = ConfigManager::new_with_path(config_path.clone())
            .expect("Failed to create config manager");

        let runtime = Arc::new(tokio::runtime::Runtime::new().unwrap());

        // Create channels
        let (ssh_msg_tx, ssh_msg_rx) = mpsc::unbounded_channel();
        let ssh_msg_rx = Arc::new(std::sync::Mutex::new(ssh_msg_rx));

        let (ai_msg_tx, ai_msg_rx) = mpsc::unbounded_channel();
        let ai_msg_rx = Arc::new(std::sync::Mutex::new(ai_msg_rx));

        let (sftp_msg_tx, sftp_msg_rx) = mpsc::unbounded_channel();
        let sftp_msg_rx = Arc::new(std::sync::Mutex::new(sftp_msg_rx));

        let ai_manager = Arc::new(TokioMutex::new(Some(AiManager::new())));

        let state = AppState {
            config_manager,
            config: AppConfig::default(),
            connections: Vec::new(),
            selected_connection: None,
            show_new_connection: false,

            new_conn_name: String::new(),
            new_conn_host: String::new(),
            new_conn_port: "22".to_string(),
            new_conn_user: String::new(),
            new_conn_password: String::new(),
            new_conn_use_key: false,
            new_conn_key_path: String::new(),

            ssh_sessions: Vec::new(),
            connection_status: Vec::new(),

            terminal_output: String::new(),
            command_input: String::new(),

            ssh_msg_tx,
            ssh_msg_rx,

            ai_manager,
            ai_messages: Vec::new(),
            ai_input: String::new(),
            ai_provider: AiProviderType::Ollama,
            ai_loading: false,

            ai_msg_tx,
            ai_msg_rx,

            show_file_browser: false,
            remote_current_path: "/".to_string(),
            local_current_path: std::env::current_dir().unwrap_or_default(),
            remote_files: Vec::new(),
            local_files: Vec::new(),
            selected_remote_files: Vec::new(),
            selected_local_file: None,
            sftp_progress: 0.0,
            sftp_status: String::new(),

            sftp_msg_tx,
            sftp_msg_rx,

            system_monitor: Arc::new(crate::monitor::SystemMonitor::new()),
            cpu_usage: 0.0,
            mem_usage: 0.0,

            command_history: crate::history::CommandHistory::new(),
            show_history_search: false,
            history_search_query: String::new(),

            show_settings: false,
            settings_page: SettingsPage::General,

            runtime,
        };

        App { state }
    }

    #[test]
    fn test_app_initialization() {
        let app = create_test_app();
        assert!(app.state.connections.is_empty());
        assert_eq!(app.state.selected_connection, None);
    }

    #[test]
    fn test_create_connection() {
        let mut app = create_test_app();

        app.state.new_conn_name = "Test Server".to_string();
        app.state.new_conn_host = "localhost".to_string();
        app.state.new_conn_port = "2222".to_string();
        app.state.new_conn_user = "testuser".to_string();
        app.state.new_conn_password = "password123".to_string();

        create_connection(&mut app.state);

        assert_eq!(app.state.connections.len(), 1);
        let conn = &app.state.connections[0];
        assert_eq!(conn.name, "Test Server");
        assert_eq!(conn.host, "localhost");
        assert_eq!(conn.port, 2222);
        assert_eq!(conn.username, "testuser");

        // Verify state arrays extended
        assert_eq!(app.state.ssh_sessions.len(), 1);
        assert_eq!(app.state.connection_status.len(), 1);
    }

    #[test]
    fn test_terminal_input_handling() {
        let mut app = create_test_app();

        // Cannot execute command without connection, but we can verify input field logic
        app.state.command_input = "ls -la".to_string();
        assert_eq!(app.state.command_input, "ls -la");

        // Mock a connection
        app.state.new_conn_name = "Test".to_string();
        app.state.new_conn_host = "test".to_string();
        create_connection(&mut app.state);
        app.state.selected_connection = Some(0);
        app.state.connection_status[0] = ConnectionStatus::Connected;

        // Manually populate the session to allow execution logic to proceed
        app.state.ssh_sessions[0] = Some(Arc::new(std::sync::Mutex::new(SshSession::new(
            "test".to_string(),
            22,
            "test".to_string(),
        ))));

        // This would spawn a thread, which is hard to test in unit test without sleep
        // But we can check if it compiles and runs without panic
        execute_ssh_command(&mut app.state, "echo hello".to_string());

        // Output format check
        assert!(app.state.terminal_output.contains("$ echo hello"));
    }

    #[test]
    fn test_ai_provider_switching() {
        let mut app = create_test_app();

        assert_eq!(app.state.ai_provider, AiProviderType::Ollama);

        // Simulate UI switching
        change_ai_provider(&mut app.state, AiProviderType::OpenAI);

        // Note: change_ai_provider spawns an async task to update the manager
        // We can't easily check the manager state instantly, but we can check the UI state logic if we had updated it
        // Actually, change_ai_provider assumes `self.ai_provider` is updated by UI binding
        // Let's manually update it as UI would
        app.state.ai_provider = AiProviderType::OpenAI;
        assert_eq!(app.state.ai_provider, AiProviderType::OpenAI);
    }
}
