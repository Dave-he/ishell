use crate::app::{
    connect_ssh, create_connection, disconnect_ssh, execute_ssh_command, send_ai_message,
};
use crate::state::AppState;
use crate::types::*;
use eframe::egui;

pub fn render_connections_panel(state: &mut AppState, ctx: &egui::Context) {
    egui::SidePanel::left("connections")
        .default_width(250.0)
        .show(ctx, |ui| {
            ui.heading("🖥️ Connections");
            ui.separator();

            if ui.button("➕ New Connection").clicked() {
                state.show_new_connection = true;
            }

            ui.separator();
            ui.heading("Servers");

            let mut connect_idx = None;
            let mut disconnect_idx = None;

            for (index, conn) in state.connections.iter().enumerate() {
                let status = state
                    .connection_status
                    .get(index)
                    .copied()
                    .unwrap_or(ConnectionStatus::Disconnected);

                let status_icon = match status {
                    ConnectionStatus::Connected => "🟢",
                    ConnectionStatus::Connecting => "🟡",
                    ConnectionStatus::Disconnected => "🔴",
                    ConnectionStatus::Error => "❌",
                };

                let response = ui.selectable_label(
                    state.selected_connection == Some(index),
                    format!("{} {}", status_icon, conn.name),
                );

                if response.clicked() {
                    state.selected_connection = Some(index);
                }

                ui.horizontal(|ui| {
                    if ui.button("Connect").clicked() {
                        connect_idx = Some(index);
                    }
                    if status == ConnectionStatus::Connected && ui.button("Disconnect").clicked() {
                        disconnect_idx = Some(index);
                    }
                });
                ui.separator();
            }

            if let Some(idx) = connect_idx {
                connect_ssh(state, idx);
            }
            if let Some(idx) = disconnect_idx {
                disconnect_ssh(state, idx);
            }
        });
}

pub fn render_ai_panel(state: &mut AppState, ctx: &egui::Context) {
    egui::SidePanel::right("ai_panel")
        .default_width(300.0)
        .show(ctx, |ui| {
            ui.heading("🤖 AI Assistant");
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Provider:");
                ui.heading(match state.ai_provider {
                    AiProviderType::Ollama => "🦙 Ollama",
                    AiProviderType::OpenAI => "🤖 OpenAI",
                    AiProviderType::Google => "🔷 Google",
                });
            });

            ui.separator();

            egui::ScrollArea::vertical()
                .min_scrolled_height(200.0)
                .show(ui, |ui| {
                    for (role, message) in &state.ai_messages {
                        let color = if role == "user" {
                            egui::Color32::from_rgb(200, 230, 201)
                        } else {
                            egui::Color32::from_rgb(225, 225, 225)
                        };

                        ui.scope(|ui| {
                            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
                            ui.painter().rect_filled(
                                ui.available_rect_before_wrap(),
                                egui::Rounding::default(),
                                color,
                            );
                            ui.label(message);
                        });
                        ui.add_space(4.0);
                    }

                    if state.ai_loading {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label("AI is thinking...");
                        });
                    }
                });

            ui.separator();
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut state.ai_input).hint_text("Ask AI..."));
                if (ui.button("Send").clicked() || ui.input(|i| i.key_pressed(egui::Key::Enter)))
                    && !state.ai_input.trim().is_empty() {
                        send_ai_message(state, state.ai_input.clone());
                        state.ai_input.clear();
                    }
            });
        });
}

pub fn render_monitor_panel(state: &mut AppState, ctx: &egui::Context) {
    // 更新系统监控数据 (v0.3.0 真实监控)
    state.system_monitor.update();
    let summary = state.system_monitor.summary();
    
    // 同时更新状态字段（用于向后兼容）
    state.cpu_usage = summary.cpu_usage;
    state.mem_usage = summary.memory_percent;
    
    egui::TopBottomPanel::bottom("monitor")
        .default_height(100.0)
        .show(ctx, |ui| {
            ui.heading("📊 System Monitor");
            ui.separator();

            ui.columns(4, |columns| {
                // CPU (真实数据)
                columns[0].vertical(|ui| {
                    ui.label(egui::RichText::new("🔥 CPU").strong());
                    ui.add(
                        egui::ProgressBar::new(summary.cpu_usage / 100.0)
                            .text(format!("{:.1}%", summary.cpu_usage))
                    );
                });

                // Memory (真实数据)
                columns[1].vertical(|ui| {
                    ui.label(egui::RichText::new("💾 Memory").strong());
                    ui.add(
                        egui::ProgressBar::new(summary.memory_percent / 100.0)
                            .text(format!("{:.1}%", summary.memory_percent))
                    );
                    ui.label(format!(
                        "{} / {}",
                        crate::monitor::SystemSummary::format_memory(summary.memory_used),
                        crate::monitor::SystemSummary::format_memory(summary.memory_total)
                    ));
                });

                // Disk (真实数据 - 显示第一个磁盘)
                columns[2].vertical(|ui| {
                    ui.label(egui::RichText::new("💿 Disk").strong());
                    if let Some((mount, used, total)) = summary.disk_info.first() {
                        let percent = if *total > 0 {
                            (*used as f64 / *total as f64 * 100.0) as f32
                        } else {
                            0.0
                        };
                        ui.add(
                            egui::ProgressBar::new(percent / 100.0)
                                .text(format!("{:.1}%", percent))
                        );
                        ui.label(format!(
                            "{}: {} / {}",
                            mount,
                            crate::monitor::SystemSummary::format_memory(*used),
                            crate::monitor::SystemSummary::format_memory(*total)
                        ));
                    } else {
                        ui.label("No disk data");
                    }
                });

                // Network (真实数据)
                columns[3].vertical(|ui| {
                    ui.label(egui::RichText::new("🌐 Network").strong());
                    ui.label(format!(
                        "↓ {}",
                        crate::monitor::SystemSummary::format_memory(summary.network_received)
                    ));
                    ui.label(format!(
                        "↑ {}",
                        crate::monitor::SystemSummary::format_memory(summary.network_transmitted)
                    ));
                });
            });
        });
}

pub fn render_terminal_panel(state: &mut AppState, ctx: &egui::Context) {
    // 获取活跃标签的可变引用
    if let Some(tab) = state.tab_manager.active_tab_mut() {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("💻 Terminal");
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // 历史搜索按钮
                    if ui.button("🔍 History (Ctrl+R)").clicked() 
                        || ui.input(|i| i.key_pressed(egui::Key::R) && i.modifiers.ctrl) {
                        // TODO: 使用标签的历史
                    }
                });
            });
            
            ui.separator();

            egui::ScrollArea::vertical()
                .min_scrolled_height(ui.available_height() - 60.0)
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut tab.state.terminal_output)
                            .code_editor()
                            .desired_rows(10)
                            .lock_focus(true),
                    );
                });

            ui.separator();
            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut tab.state.command_input).hint_text("Enter command..."),
                );
                
                let connected = tab.state.connection_status == crate::types::ConnectionStatus::Connected;
                
                ui.label(if connected {
                    "🟢 Connected"
                } else {
                    "🔴 Disconnected"
                });
            });
        });
    }
    
    // 渲染历史搜索窗口
    render_history_search_window(state, ctx);
}

pub fn render_new_connection_dialog(state: &mut AppState, ctx: &egui::Context) {
    if state.show_new_connection {
        egui::Window::new("New SSH Connection")
            .collapsible(false)
            .show(ctx, |ui| {
                ui.heading("Connection Details");
                ui.separator();

                ui.label("Name:");
                ui.text_edit_singleline(&mut state.new_conn_name);

                ui.label("Host:");
                ui.text_edit_singleline(&mut state.new_conn_host);

                ui.label("Port:");
                ui.text_edit_singleline(&mut state.new_conn_port);

                ui.label("Username:");
                ui.text_edit_singleline(&mut state.new_conn_user);

                ui.checkbox(&mut state.new_conn_use_key, "Use Private Key");

                if state.new_conn_use_key {
                    ui.label("Key Path:");
                    ui.text_edit_singleline(&mut state.new_conn_key_path);

                    ui.label("Passphrase (optional):");
                    ui.add(egui::TextEdit::singleline(&mut state.new_conn_password).password(true));
                } else {
                    ui.label("Password:");
                    ui.add(egui::TextEdit::singleline(&mut state.new_conn_password).password(true));
                }

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        create_connection(state);
                    }
                    if ui.button("Cancel").clicked() {
                        state.show_new_connection = false;
                    }
                });
            });
    }
}

/// 渲染命令历史搜索窗口
fn render_history_search_window(state: &mut AppState, ctx: &egui::Context) {
    if !state.show_history_search {
        return;
    }

    egui::Window::new("🔍 命令历史")
        .default_width(600.0)
        .default_height(400.0)
        .collapsible(false)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("搜索:");
                let resp = ui.text_edit_singleline(&mut state.history_search_query);
                
                // 自动聚焦搜索框
                if state.show_history_search {
                    resp.request_focus();
                }
                
                if ui.button("❌").clicked() {
                    state.show_history_search = false;
                }
            });

            ui.separator();

            // 统计信息
            let stats = state.command_history.stats();
            ui.horizontal(|ui| {
                ui.label(format!("总命令: {}", stats.total_commands));
                ui.separator();
                ui.label(format!("唯一命令: {}", stats.unique_commands));
                ui.separator();
                ui.label(format!("连接数: {}", stats.unique_connections));
            });

            ui.separator();

            // 历史列表
            egui::ScrollArea::vertical()
                .max_height(300.0)
                .show(ui, |ui| {
                    let results = state.command_history.search(&state.history_search_query);
                    
                    if results.is_empty() {
                        ui.label("无匹配的历史记录");
                    } else {
                        for entry in results.iter().take(50) {
                            ui.horizontal(|ui| {
                                // 命令按钮
                                if ui.selectable_label(false, &entry.command).clicked() {
                                    state.command_input = entry.command.clone();
                                    state.show_history_search = false;
                                }
                                
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(
                                        egui::RichText::new(&entry.timestamp)
                                            .small()
                                            .weak()
                                    );
                                    ui.label(
                                        egui::RichText::new(&format!("📡 {}", entry.connection))
                                            .small()
                                            .weak()
                                    );
                                });                          });
                            
                            ui.separator();
                        }
                    }
                });

            ui.separator();

            // 操作按钮
            ui.horizontal(|ui| {
                if ui.button("🗑️ 清空历史").clicked() {
                    state.command_history.clear();
                }
                
                if ui.button("💾 保存历史").clicked() {
                    // 保存到配置目录
                    if let Some(config_dir) = dirs::config_dir() {
                        let history_path = config_dir.join("ishell").join("history.json");
                        if let Err(e) = state.command_history.save(&history_path) {
                            eprintln!("Failed to save history: {}", e);
                        }
                    }
                }
            });
        });
}

