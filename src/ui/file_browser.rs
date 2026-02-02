use crate::state::AppState;
use crate::types::ConnectionStatus;
use eframe::egui;

// ============================================================================
// 文件浏览器 UI (简化版 v0.3.0)
// ============================================================================

pub fn render_file_browser(state: &mut AppState, ctx: &egui::Context) {
    if !state.show_file_browser {
        return;
    }

    egui::Window::new("📁 SFTP File Browser")
        .default_width(900.0)
        .default_height(600.0)
        .show(ctx, |ui| {
            // 检查是否有连接
            if state.selected_connection.is_none() {
                ui.colored_label(
                    egui::Color32::YELLOW,
                    "⚠️ Please connect to a server first!",
                );
                return;
            }

            let selected_idx = state.selected_connection.unwrap();
            let is_connected = state.connection_status.get(selected_idx)
                == Some(&ConnectionStatus::Connected);

            if !is_connected {
                ui.colored_label(
                    egui::Color32::YELLOW,
                    "⚠️ Server not connected! Please connect first.",
                );
                return;
            }

            // 路径导航栏
            ui.horizontal(|ui| {
                ui.label("Remote Path:");
                ui.label(&state.remote_current_path);

                if ui.button("🔄 Refresh").clicked() {
                    request_file_list(state);
                }

                ui.separator();

                if ui.button("🏠 Home").clicked() {
                    state.remote_current_path = "/".to_string();
                    request_file_list(state);
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

// ============================================================================
// 远程文件渲染
// ============================================================================

fn render_remote_files(state: &mut AppState, ui: &mut egui::Ui) {
    // ".." 返回上级目录
    if state.remote_current_path != "/" {
        if ui
            .selectable_label(false, "📁 ..")
            .on_hover_text("Go to parent directory")
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
            if entry.is_dir {
                // 进入目录
                state.remote_current_path = entry.path.clone();
                request_file_list(state);
            } else {
                // 切换文件选择状态
                if is_selected {
                    state.selected_remote_files.retain(|p| p != &entry.path);
                } else {
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

/// 返回上级目录
fn go_parent_dir(state: &mut AppState) {
    let path = &state.remote_current_path;
    if path == "/" {
        return;
    }

    let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    if parts.is_empty() {
        state.remote_current_path = "/".to_string();
    } else {
        state.remote_current_path = format!("/{}", parts[..parts.len() - 1].join("/"));
        if state.remote_current_path.is_empty() {
            state.remote_current_path = "/".to_string();
        }
    }

    request_file_list(state);
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
