use crate::types::FileEntry;
use crate::state::AppState;
use eframe::egui;
use std::path::Path;

/// 渲染文件浏览器面板 (v0.3.0)
pub fn render_file_browser(state: &mut AppState, ctx: &egui::Context) {
    if !state.show_file_browser {
        return;
    }

    egui::Window::new("📁 File Browser")
        .resizable(true)
        .default_width(800.0)
        .default_height(600.0)
        .collapsible(false)
        .show(ctx, |ui| {
            // 顶部工具栏
            ui.horizontal(|ui| {
                ui.label("🖥️ Local:");
                let local_path = state.local_current_path.display().to_string();
                ui.label(egui::RichText::new(&local_path).monospace().small());
                
                if ui.button("📂").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        state.local_current_path = path;
                        state.local_files = list_local_files(&path);
                    }
                }
                
                ui.separator();
                
                ui.label("☁️ Remote:");
                ui.text_edit_singleline(&mut state.remote_current_path);
                if ui.button("🔄").clicked() {
                    refresh_remote_files(state);
                }
            });

            ui.separator();

            // 文件列表（双列）
            ui.horizontal(|ui| {
                // 本地文件
                ui.vertical(|ui| {
                    ui.heading("💻 Local Files");
                    ui.separator();
                    
                    render_file_list_local(state, ui);
                });
                
                ui.separator();
                
                // 远程文件
                ui.vertical(|ui| {
                    ui.heading("☁️ Remote Files");
                    ui.separator();
                    
                    render_file_list_remote(state, ui);
                });
            });

            ui.separator();

            // 操作按钮
            ui.horizontal(|ui| {
                if ui.button_enabled(state.selected_local_file.is_some(), "⬆️ Upload").clicked() {
                    upload_file(state);
                }
                
                if ui.button_enabled(!state.selected_remote_files.is_empty(), "⬇️ Download").clicked() {
                    download_file(state);
                }
                
                if ui.button_enabled(!state.selected_remote_files.is_empty(), "🗑️ Delete").clicked() {
                    delete_remote_file(state);
                }
                
                if ui.button("📁 New Folder").clicked() {
                    create_remote_folder(state);
                }
            });

            // 进度条
            if state.sftp_progress > 0.0 {
                ui.add_space(10.0);
                ui.label(&state.sftp_status);
                ui.add(egui::ProgressBar::new(state.sftp_progress / 100.0));
            }

            // 拖放区域检测
            handle_file_drop(state, ui);
        });
}

/// 渲染本地文件列表
fn render_file_list_local(state: &mut AppState, ui: &mut egui::Ui) {
    egui::ScrollArea::vertical()
        .max_height(400.0)
        .show(ui, |ui| {
            // 父目录
            if state.local_current_path.parent().is_some() {
                if ui.button("📁 ..").clicked() {
                    if let Some(parent) = state.local_current_path.parent() {
                        state.local_current_path = parent.to_path_buf();
                        state.local_files = list_local_files(&state.local_current_path);
                    }
                }
                ui.separator();
            }

            for entry in &state.local_files {
                let icon = if entry.is_dir { "📁" } else { "📄" };
                let is_selected = state.selected_local_file
                    .as_ref()
                    .map(|p| p == &Path::new(&entry.path))
                    .unwrap_or(false);

                let response = ui.selectable_label(is_selected, format!("{} {}", icon, entry.name));
                
                if response.clicked() {
                    if entry.is_dir {
                        state.local_current_path = Path::new(&entry.path);
                        state.local_files = list_local_files(&state.local_current_path);
                    } else {
                        state.selected_local_file = Some(Path::new(&entry.path));
                    }
                }

                if response.double_clicked() {
                    if entry.is_dir {
                        state.local_current_path = Path::new(&entry.path);
                        state.local_files = list_local_files(&state.local_current_path);
                    }
                }
            }
        });
}

/// 渲染远程文件列表
fn render_file_list_remote(state: &mut AppState, ui: &mut egui::Ui) {
    egui::ScrollArea::vertical()
        .max_height(400.0)
        .show(ui, |ui| {
            // 父目录
            if state.remote_current_path != "/" {
                let parent_path = Path::new(&state.remote_current_path)
                    .parent()
                    .map(|p| p.to_str().unwrap().to_string())
                    .unwrap_or("/".to_string());
                
                if ui.button("📁 ..").clicked() {
                    state.remote_current_path = parent_path;
                    refresh_remote_files(state);
                }
                ui.separator();
            }

            for entry in &state.remote_files {
                let icon = if entry.is_dir { "📁" } else { "📄" };
                let is_selected = state.selected_remote_files.contains(&entry.name);

                let response = ui.selectable_label(is_selected, format!("{} {} {} ({})",
                    icon,
                    entry.name,
                    if entry.is_dir { "" } else {
                        &format_file_size(entry.size)
                    },
                    entry.modified.as_ref()
                        .map(|t| format!("{:?}", t))
                        .unwrap_or_default()
                ));

                if response.clicked() {
                    if entry.is_dir {
                        state.remote_current_path = entry.path.clone();
                        refresh_remote_files(state);
                    } else {
                        if is_selected {
                            state.selected_remote_files.retain(|n| n != &entry.name);
                        } else {
                            state.selected_remote_files.push(entry.name.clone());
                        }
                    }
                }

                if response.double_clicked() {
                    if entry.is_dir {
                        state.remote_current_path = entry.path.clone();
                        refresh_remote_files(state);
                    }
                }
            }
        });
}

/// 列出本地文件
fn list_local_files(path: &std::path::Path) -> Vec<FileEntry> {
    let mut files = Vec::new();

    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') {
                continue; // 跳过隐藏文件
            }

            let path_str = entry.path().to_string_lossy().to_string();
            let is_dir = entry.path().is_dir();
            let metadata = entry.metadata().ok();
            let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
            let modified = metadata.and_then(|m| m.modified()).ok();

            files.push(FileEntry {
                name,
                path: path_str,
                is_dir,
                size,
                modified,
                permissions: None,
            });
        }
    }

    files.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        }
    });

    files
}

/// 刷新远程文件列表
fn refresh_remote_files(state: &mut AppState) {
    if let Some(index) = state.selected_connection {
        if let Some(session) = &state.ssh_sessions[index] {
            if let Ok(sftp) = session.sftp() {
                match sftp.list_dir(&state.remote_current_path) {
                    Ok(files) => {
                        state.remote_files = files;
                        state.sftp_status = format!("Loaded {} files", files.len());
                    }
                    Err(e) => {
                        state.sftp_status = format!("Failed to list: {}", e);
                    }
                }
            }
        }
    }
}

/// 上传文件
fn upload_file(state: &mut AppState) {
    if let Some(local_path) = &state.selected_local_file {
        if let Some(index) = state.selected_connection {
            if let Some(session) = &state.ssh_sessions[index] {
                if let Ok(sftp) = session.sftp() {
                    let file_name = local_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("file");
                    
                    let remote_path = format!("{}/{}", state.remote_current_path, file_name);
                    let local_path_clone = local_path.clone();
                    let tx = state.sftp_msg_tx.clone();

                    state.sftp_progress = 0.0;
                    state.sftp_status = format!("Uploading {}...", file_name);

                    std::thread::spawn(move || {
                        if let Ok(mut sftp) = session.sftp() {
                            match sftp.upload_file(&local_path_clone, &remote_path, |progress| {
                                let _ = tx.send(crate::types::SftpMessage::Progress(progress));
                            }) {
                                Ok(_) => {
                                    let _ = tx.send(crate::types::SftpMessage::Complete);
                                }
                                Err(e) => {
                                    let _ = tx.send(crate::types::SftpMessage::Error(
                                        format!("Upload failed: {}", e)
                                    ));
                                }
                            }
                        }
                    });
                }
            }
        }
    }
}

/// 下载文件
fn download_file(state: &mut AppState) {
    if state.selected_remote_files.len() == 1 {
        let remote_file = &state.selected_remote_files[0];
        
        if let Some(index) = state.selected_connection {
            if let Some(session) = &state.ssh_sessions[index] {
                if let Some(save_path) = rfd::FileDialog::new()
                    .set_title("Save file")
                    .save_file()
                {
                    if let Ok(sftp) = session.sftp() {
                        let remote_path = format!("{}/{}", state.remote_current_path, remote_file);
                        let tx = state.sftp_msg_tx.clone();

                        state.sftp_progress = 0.0;
                        state.sftp_status = format!("Downloading {}...", remote_file);

                        std::thread::spawn(move || {
                            if let Ok(mut sftp_client) = session.sftp() {
                                match sftp_client.download_file(&remote_path, &save_path, |progress| {
                                    let _ = tx.send(crate::types::SftpMessage::Progress(progress));
                                }) {
                                    Ok(_) => {
                                        let _ = tx.send(crate::types::SftpMessage::Complete);
                                    }
                                    Err(e) => {
                                        let _ = tx.send(crate::types::SftpMessage::Error(
                                            format!("Download failed: {}", e)
                                        ));
                                    }
                                }
                            }
                        });
                    }
                }
            }
        }
    }
}

/// 删除远程文件
fn delete_remote_file(state: &mut AppState) {
    for file_name in &state.selected_remote_files {
        if let Some(index) = state.selected_connection {
            if let Some(session) = &state.ssh_sessions[index] {
                if let Ok(sftp) = session.sftp() {
                    let remote_path = format!("{}/{}", state.remote_current_path, file_name);
                    
                    match sftp.delete_file(&remote_path) {
                        Ok(_) => {
                            state.sftp_status = format!("Deleted {}", file_name);
                        }
                        Err(e) => {
                            state.sftp_status = format!("Failed to delete {}: {}", file_name, e);
                        }
                    }
                }
            }
        }
    }
    
    refresh_remote_files(state);
    state.selected_remote_files.clear();
}

/// 创建远程文件夹
fn create_remote_folder(state: &mut AppState) {
    if let Some(index) = state.selected_connection {
        if let Some(session) = &state.ssh_sessions[index] {
            if let Ok(sftp) = session.sftp() {
                let folder_name = format!("new_folder_{}", 
                    std::time::UNIX_EPOCH.elapsed().unwrap().as_secs());
                let folder_path = format!("{}/{}", state.remote_current_path, folder_name);
                
                match sftp.create_dir(&folder_path) {
                    Ok(_) => {
                        state.sftp_status = format!("Created folder {}", folder_name);
                        refresh_remote_files(state);
                    }
                    Err(e) => {
                        state.sftp_status = format!("Failed to create folder: {}", e);
                    }
                }
            }
        }
    }
}

/// 处理文件拖放
fn handle_file_drop(state: &mut AppState, ui: &mut egui::Ui) {
    let dropped_files = ui.input(|i| i.raw.dropped_files.clone());
    
    if !dropped_files.is_empty() {
        for file in dropped_files {
            if let Some(path) = file.path {
                state.local_current_path = if path.is_dir() {
                    path.clone()
                } else {
                    path.parent().unwrap().to_path_buf()
                };
                
                state.local_files = list_local_files(&state.local_current_path);
                
                if !path.is_dir() {
                    state.selected_local_file = Some(path);
                    upload_file(state);
                }
            }
        }
    }
}

/// 格式化文件大小
fn format_file_size(size: u64) -> String {
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
