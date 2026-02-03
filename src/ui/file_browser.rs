use crate::state::AppState;
use crate::types::{ConnectionStatus, FileEntry};
use crate::tabs::TabManager;  // v1.0.0: 标签管理器
use eframe::egui;
use std::path::PathBuf;

// ============================================================================
// 文件浏览器 UI (简化版 v0.3.0)
// v1.0.0: 支持 SFTP 状态绑定到活跃标签
// ============================================================================

pub fn render_file_browser(state: &mut AppState, ctx: &egui::Context) {
    if !state.show_file_browser {
        return;
    }

    // 检查连接状态
    let is_connected = check_connected_to_any(state);
    
    egui::Window::new("📁 SFTP File Browser")
        .default_width(900.0)
        .default_height(600.0)
        .show(ctx, |ui| {
            // 检查是否有连接
            if !is_connected {
                ui.colored_label(
                    egui::Color32::YELLOW,
                    "⚠️ Please connect to a server first!",
                );
                return;
            }

            // 工具栏
            ui.horizontal(|ui| {
                ui.heading("File Transfer");
                
                ui.separator();
                
                // 上传按钮
                let upload_enabled = !state.selected_local_files.is_empty();
                if ui.add_enabled(upload_enabled, egui::Button::new("⬆️ Upload"))
                    .on_hover_text("Upload selected file(s) to Downloads folder")
                    .clicked()
                {
                    upload_files_to_active_tab(state);
                }

                // 下载按钮
                let download_enabled = !state.selected_remote_files.is_empty();
                if ui.add_enabled(download_enabled, egui::Button::new("⬇️ Download"))
                    .on_hover_text("Download selected file(s) to Downloads folder")
                    .clicked()
                {
                    download_files_from_active_tab(state);
                }

                ui.separator();

                // 刷新按钮
                if ui.button("🔄 Refresh").clicked() {
                    refresh_remote_files(state);
                }

                ui.separator();

                if ui.button("🏠 Home").clicked() {
                    go_to_home(state);
                }

                if ui.button("⬆️ Up").clicked() {
                    go_parent_dir(state);
                }
            });

            ui.separator();

            // 远程文件列表
            ui.heading("☁️ Remote Files");
            ui.separator();

            egui::ScrollArea::vertical()
                .max_height(450.0)
                .show(ui, |ui| {
                    render_remote_files(state, ui);
                });

            ui.separator();

            // 状态栏
            ui.horizontal(|ui| {
                if ui.button("❌ Close").clicked() {
                    state.show_file_browser = false;
                }

                ui.separator();

                if !state.sftp_status.is_empty() {
                    ui.label(&state.sftp_status);
                }
            });

            // 进度条
            if state.sftp_progress > 0.0 && state.sftp_progress < 1.0 {
                ui.separator();
                ui.add(
                    egui::ProgressBar::new(state.sftp_progress)
                        .text(format!("{:.0}%", state.sftp_progress * 100.0)),
                );
            }
        });
}

/// 检查是否有任何连接（用于 SFTP）
fn check_connected_to_any(state: &AppState) -> bool {
    state.connection_status.iter()
        .any(|&s| s == ConnectionStatus::Connected)
}

/// 获取活跃标签的 SFTP 状态（如果有）
fn get_active_tab_sftp_state(state: &mut AppState) -> Option<&mut crate::state::SftpTabState> {
    state.tab_manager.active_tab_mut()
        .map(|tab| tab.state.sftp_state.as_mut())
        .flatten()
}

/// 初始化活跃标签的 SFTP 状态
fn init_tab_sftp_state(state: &mut AppState) {
    if let Some(tab) = state.tab_manager.active_tab_mut() {
        if tab.state.sftp_state.is_none() {
            tab.state.sftp_state = Some(crate::state::SftpTabState {
                remote_path: "/".to_string(),
                remote_files: Vec::new(),
                selected_files: Vec::new(),
            });
        }
    }
}

/// 上传文件到活跃标签
fn upload_files_to_active_tab(state: &mut AppState) {
    init_tab_sftp_state(state);
    // TODO: 实现上传逻辑
    eprintln!("Upload to active tab - TODO");
}

/// 从活跃标签下载文件
fn download_files_from_active_tab(state: &mut AppState) {
    // TODO: 实现下载逻辑
    eprintln!("Download from active tab - TODO");
}

/// 刷新活跃标签的远程文件
fn refresh_remote_files(state: &mut AppState) {
    // TODO: 实现刷新逻辑
    eprintln!("Refresh remote files - TODO");
}

/// 转到上级目录（活跃标签）
fn go_parent_dir(state: &mut AppState) {
    if let Some(tab) = state.tab_manager.active_tab_mut() {
        if let Some(sftp_state) = tab.state.sftp_state.as_mut() {
            let path = &sftp_state.remote_path;
            if path == "/" {
                return;
            }

            let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
            if parts.is_empty() {
                sftp_state.remote_path = "/".to_string();
            } else {
                sftp_state.remote_path = format!("/{}", parts[..parts.len() - 1].join("/"));
                if sftp_state.remote_path.is_empty() {
                    sftp_state.remote_path = "/".to_string();
                }
            }
        }
    }
}

/// 转到主目录（活跃标签）
fn go_to_home(state: &mut AppState) {
    if let Some(tab) = state.tab_manager.active_tab_mut() {
        if let Some(sftp_state) = tab.state.sftp_state.as_mut() {
            sftp_state.remote_path = "/".to_string();
        }
    }
}

// ============================================================================
// 本地文件面板
// ============================================================================

fn render_local_panel(state: &mut AppState, ui: &mut egui::Ui) {
    ui.vertical(|ui| {
        ui.heading("💻 本地文件");
        ui.separator();
        
        // 路径导航
        ui.horizontal(|ui| {
            ui.label("路径:");
            
            // Browse 按钮
            if ui.button("📂 浏览...").on_hover_text("选择文件夹").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .set_directory(&state.local_current_path)
                    .pick_folder() 
                {
                    state.local_current_path = path;
                    refresh_local_files(state);
                }
            }
            
            // 上级目录
            if ui.button("⬆️").on_hover_text("返回上级").clicked() {
                go_local_parent_dir(state);
            }
            
            // Home
            if ui.button("🏠").on_hover_text("主目录").clicked() {
                if let Some(home) = dirs::home_dir() {
                    state.local_current_path = home;
                    refresh_local_files(state);
                }
            }
        });
        
        // 当前路径
        ui.label(
            egui::RichText::new(state.local_current_path.to_string_lossy())
                .small()
                .weak()
        );
        
        ui.separator();
        
        // 文件列表
        egui::ScrollArea::vertical()
            .max_height(400.0)
            .show(ui, |ui| {
                render_local_files(state, ui);
            });
        
        // 底部信息
        ui.separator();
        ui.label(format!("已选择: {} 个文件", state.selected_local_files.len()));
    });
}

fn render_local_files(state: &mut AppState, ui: &mut egui::Ui) {
    // 初始化加载
    if state.local_files.is_empty() {
        refresh_local_files(state);
    }
    
    // ".." 返回上级
    if state.local_current_path.parent().is_some() {
        if ui.selectable_label(false, "📁 ..").on_hover_text("返回上级").clicked() {
            go_local_parent_dir(state);
        }
    }
    
    // 显示文件列表
    for entry in state.local_files.clone() {
        let icon = if entry.is_dir { "📁" } else { "📄" };
        let size_str = if !entry.is_dir && entry.size > 0 {
            format_size(entry.size)
        } else {
            String::new()
        };
        
        let label = format!("{} {}  {}", icon, entry.name, size_str);
        
        let is_selected = state.selected_local_files
            .iter()
            .any(|p| p.to_string_lossy() == entry.path);
        
        let response = ui.selectable_label(is_selected, label)
            .on_hover_text(&entry.path);
        
        if response.clicked() {
            let path = PathBuf::from(&entry.path);
            let modifiers = ui.input(|i| i.modifiers);
            
            if entry.is_dir {
                // 进入目录
                state.local_current_path = path;
                refresh_local_files(state);
            } else {
                // 文件选择（支持多选）
                if modifiers.ctrl || modifiers.command {
                    // Ctrl/Cmd: 切换选中状态
                    if let Some(pos) = state.selected_local_files.iter().position(|p| p == &path) {
                        state.selected_local_files.remove(pos);
                    } else {
                        state.selected_local_files.push(path);
                    }
                } else {
                    // 普通点击: 单选
                    state.selected_local_files.clear();
                    state.selected_local_files.push(path);
                }
            }
        }
    }
}

// ============================================================================
// 远程文件面板
// ============================================================================

fn render_remote_panel(state: &mut AppState, ui: &mut egui::Ui) {
    ui.vertical(|ui| {
        ui.heading("☁️ 远程文件");
        ui.separator();
        
        // 检查连接状态
        if !is_connected(state) {
            ui.colored_label(
                egui::Color32::YELLOW,
                "⚠️ 请先连接到 SSH 服务器"
            );
            return;
        }
        
        // 路径导航
        ui.horizontal(|ui| {
            ui.label("路径:");
            
            if ui.button("🏠").on_hover_text("根目录").clicked() {
                state.remote_current_path = "/".to_string();
                request_file_list(state);
            }
            
            if ui.button("⬆️").on_hover_text("返回上级").clicked() {
                go_parent_dir(state);
            }
            
            if ui.button("🔄").on_hover_text("刷新").clicked() {
                request_file_list(state);
            }
        });
        
        // 当前路径
        ui.label(
            egui::RichText::new(&state.remote_current_path)
                .small()
                .weak()
        );
        
        ui.separator();
        
        // 文件列表
        egui::ScrollArea::vertical()
            .max_height(400.0)
            .show(ui, |ui| {
                render_remote_files(state, ui);
            });
        
        // 底部信息
        ui.separator();
        ui.label(format!("已选择: {} 个文件", state.selected_remote_files.len()));
    });
}

// ============================================================================
// 远程文件渲染
// ============================================================================

fn render_remote_files(state: &mut AppState, ui: &mut egui::Ui) {
    // 初始化加载
    if state.remote_files.is_empty() {
        request_file_list(state);
    }
    
    // ".." 返回上级目录
    if state.remote_current_path != "/" {
        if ui
            .selectable_label(false, "📁 ..")
            .on_hover_text("返回上级")
            .clicked()
        {
            go_parent_dir(state);
        }
    }

    // 显示远程文件列表
    for entry in state.remote_files.clone() {
        let icon = if entry.is_dir { "📁" } else { "📄" };
        let size_str = if entry.size > 0 {
            format_size(entry.size)
        } else {
            String::new()
        };
        let label = format!("{} {}  {}", icon, entry.name, size_str);

        let is_selected = state.selected_remote_files.contains(&entry.path);

        let response = ui.selectable_label(is_selected, label)
            .on_hover_text(&entry.path);

        if response.clicked() {
            let modifiers = ui.input(|i| i.modifiers);
            
            if entry.is_dir {
                // 进入目录
                state.remote_current_path = entry.path.clone();
                request_file_list(state);
            } else {
                // 文件选择（支持多选）
                if modifiers.ctrl || modifiers.command {
                    // Ctrl/Cmd: 切换选中状态
                    if let Some(pos) = state.selected_remote_files.iter().position(|p| p == &entry.path) {
                        state.selected_remote_files.remove(pos);
                    } else {
                        state.selected_remote_files.push(entry.path.clone());
                    }
                } else {
                    // 普通点击: 单选
                    state.selected_remote_files.clear();
                    state.selected_remote_files.push(entry.path.clone());
                }
            }
        }
    }
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 请求远程文件列表
fn request_file_list(state: &mut AppState) {
    use std::sync::Arc;
    
    if let Some(selected_idx) = state.selected_connection {
        if let Some(Some(session)) = state.ssh_sessions.get(selected_idx) {
            let session_clone = Arc::clone(session);
            let path = state.remote_current_path.clone();
            let tx = state.sftp_msg_tx.clone();

            state.sftp_status = "Loading...".to_string();

            // 在后台线程执行 SFTP 操作
            std::thread::spawn(move || {
                let session = session_clone.lock().unwrap();
                match session.sftp() {
                    Ok(sftp_client) => {
                        match sftp_client.list_dir(&path) {
                            Ok(files) => {
                                let _ = tx.send(crate::types::SftpMessage::FileList(files));
                            }
                            Err(e) => {
                                let _ = tx.send(crate::types::SftpMessage::Error(format!(
                                    "Failed to list directory: {}",
                                    e
                                )));
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(crate::types::SftpMessage::Error(format!(
                            "SFTP connection failed: {}",
                            e
                        )));
                    }
                }
            });
        }
    }
}

// ============================================================================
// 本地文件操作辅助函数
// ============================================================================

/// 刷新本地文件列表
fn refresh_local_files(state: &mut AppState) {
    state.local_files.clear();
    
    match std::fs::read_dir(&state.local_current_path) {
        Ok(entries) => {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    let path = entry.path();
                    let name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string();
                    
                    let file_entry = FileEntry {
                        name,
                        path: path.to_string_lossy().to_string(),
                        is_dir: metadata.is_dir(),
                        size: metadata.len(),
                        modified: metadata.modified().ok(),
                        permissions: None,
                    };
                    
                    state.local_files.push(file_entry);
                }
            }
            
            // 排序：目录在前，然后按名称
            state.local_files.sort_by(|a, b| {
                match (a.is_dir, b.is_dir) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                }
            });
        }
        Err(e) => {
            eprintln!("Failed to read local directory: {}", e);
        }
    }
}

/// 本地目录返回上级
fn go_local_parent_dir(state: &mut AppState) {
    if let Some(parent) = state.local_current_path.parent() {
        state.local_current_path = parent.to_path_buf();
        refresh_local_files(state);
        state.selected_local_files.clear();
    }
}

/// 检查是否已连接
fn is_connected(state: &AppState) -> bool {
    state.selected_connection
        .and_then(|idx| state.connection_status.get(idx))
        .map(|s| *s == ConnectionStatus::Connected)
        .unwrap_or(false)
}

/// 上传选中的本地文件
fn upload_selected_files(state: &mut AppState) {
    if state.selected_local_files.is_empty() {
        return;
    }
    
    for local_path in state.selected_local_files.clone() {
        upload_file(state, local_path);
    }
    
    state.selected_local_files.clear();
}

/// 处理文件拖入
fn handle_file_drop(state: &mut AppState, ctx: &egui::Context) {
    ctx.input(|i| {
        if !i.raw.dropped_files.is_empty() {
            let files = i.raw.dropped_files.clone();
            
            // 检查是否已连接
            if !is_connected(state) {
                state.sftp_status = "❌ Error: Please connect to server first".to_string();
                return;
            }
            
            state.sftp_status = format!("Preparing to upload {} file(s)...", files.len());
            
            // 上传拖入的文件
            for dropped_file in files {
                if let Some(path) = dropped_file.path {
                    upload_file(state, path);
                }
            }
        }
    });
}

/// 格式化文件大小
fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if size >= GB {
        format!("{:.2} GB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.2} MB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.2} KB", size as f64 / KB as f64)
    } else {
        format!("{} B", size)
    }
}

/// 上传文件到远程服务器
fn upload_file(state: &mut AppState, local_path: std::path::PathBuf) {
    use std::sync::Arc;
    
    if let Some(selected_idx) = state.selected_connection {
        if let Some(Some(session)) = state.ssh_sessions.get(selected_idx) {
            let session_clone = Arc::clone(session);
            let remote_path = state.remote_current_path.clone();
            let tx = state.sftp_msg_tx.clone();
            
            // 获取文件名
            let file_name = local_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();
            
            // 构建完整的远程路径
            let remote_file_path = if remote_path.ends_with('/') {
                format!("{}{}", remote_path, file_name)
            } else {
                format!("{}/{}", remote_path, file_name)
            };
            
            state.sftp_status = format!("Uploading {}...", file_name);
            state.sftp_progress = 0.0;
            
            // 在后台线程执行上传
            std::thread::spawn(move || {
                let session = session_clone.lock().unwrap();
                match session.sftp() {
                    Ok(sftp_client) => {
                        let tx_clone = tx.clone();
                        let result = sftp_client.upload_file(
                            &local_path,
                            &remote_file_path,
                            move |progress| {
                                let _ = tx_clone.send(crate::types::SftpMessage::Progress(progress));
                            }
                        );
                        
                        match result {
                            Ok(_) => {
                                let _ = tx.send(crate::types::SftpMessage::Complete);
                            }
                            Err(e) => {
                                let _ = tx.send(crate::types::SftpMessage::Error(format!(
                                    "Upload failed: {}",
                                    e
                                )));
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(crate::types::SftpMessage::Error(format!(
                            "SFTP connection failed: {}",
                            e
                        )));
                    }
                }
            });
        }
    }
}

/// 下载选中的文件到本地 Downloads 文件夹
fn download_selected_files(state: &mut AppState) {
    use std::sync::Arc;
    
    if state.selected_remote_files.is_empty() {
        return;
    }
    
    // 获取下载目录（使用 Downloads 文件夹）
    let download_dir = dirs::download_dir().unwrap_or_else(|| {
        std::env::current_dir().unwrap_or_default()
    });
    
    if let Some(selected_idx) = state.selected_connection {
        if let Some(Some(session)) = state.ssh_sessions.get(selected_idx) {
            let session_clone = Arc::clone(session);
            let remote_files = state.selected_remote_files.clone();
            let tx = state.sftp_msg_tx.clone();
            
            state.sftp_status = format!("Downloading {} file(s)...", remote_files.len());
            state.sftp_progress = 0.0;
            
            // 在后台线程执行下载
            std::thread::spawn(move || {
                let session = session_clone.lock().unwrap();
                match session.sftp() {
                    Ok(sftp_client) => {
                        let total_files = remote_files.len();
                        for (idx, remote_path) in remote_files.iter().enumerate() {
                            // 提取文件名
                            let file_name = std::path::Path::new(remote_path)
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("download")
                                .to_string();
                            
                            let local_path = download_dir.join(&file_name);
                            
                            let tx_clone = tx.clone();
                            let result = sftp_client.download_file(
                                remote_path,
                                &local_path,
                                move |progress| {
                                    // 计算总体进度（考虑多个文件）
                                    let total_progress = (idx as f32 + progress) / total_files as f32;
                                    let _ = tx_clone.send(crate::types::SftpMessage::Progress(total_progress));
                                }
                            );
                            
                            if let Err(e) = result {
                                let _ = tx.send(crate::types::SftpMessage::Error(format!(
                                    "Failed to download {}: {}",
                                    file_name, e
                                )));
                                return;
                            }
                        }
                        
                        let _ = tx.send(crate::types::SftpMessage::Complete);
                    }
                    Err(e) => {
                        let _ = tx.send(crate::types::SftpMessage::Error(format!(
                            "SFTP connection failed: {}",
                            e
                        )));
                    }
                }
            });
        }
    }
}
