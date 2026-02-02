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
    // 使用模拟数据（v0.3.0 之前）
    // v0.3.0 将实现真实系统监控
    
    egui::TopBottomPanel::bottom("monitor")
        .default_height(100.0)
        .show(ctx, |ui| {
            ui.heading("📊 系统监控");
            ui.separator();

            ui.columns(4, |columns| {
                // CPU (模拟数据)
                columns[0].vertical(|ui| {
                    ui.label(egui::RichText::new("🔥 CPU").strong());
                    ui.add(
                        egui::ProgressBar::new(state.cpu_usage / 100.0)
                            .text(format!("{:.1}%", state.cpu_usage))
                    );
                });

                // 内存 (模拟数据)
                columns[1].vertical(|ui| {
                    ui.label(egui::RichText::new("💾 Memory").strong());
                    ui.add(
                        egui::ProgressBar::new(state.mem_usage / 100.0)
                            .text(format!("{:.1}%", state.mem_usage))
                    );
                });

                // 磁盘 (模拟)
                columns[2].vertical(|ui| {
                    ui.label(egui::RichText::new("💿 Disk").strong());
                    ui.add(
                        egui::ProgressBar::new(0.65)
                            .text("65.0%")
                    );
                });

                // 网络 (模拟)
                columns[3].vertical(|ui| {
                    ui.label(egui::RichText::new("🌐 Network").strong());
                    ui.label("↓ 1.2 MB/s");
                    ui.label("↑ 256 KB/s");
                });
            });
        });
}

pub fn render_terminal_panel(state: &mut AppState, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("💻 Terminal");
        ui.separator();

        egui::ScrollArea::vertical()
            .min_scrolled_height(ui.available_height() - 60.0)
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut state.terminal_output)
                        .code_editor()
                        .desired_rows(10)
                        .lock_focus(true),
                );
            });

        ui.separator();
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut state.command_input).hint_text("Enter command..."),
            );
                if (ui.button("Execute").clicked() || ui.input(|i| i.key_pressed(egui::Key::Enter)))
                    && !state.command_input.trim().is_empty() {
                        execute_ssh_command(state, state.command_input.clone());
                        state.command_input.clear();
                    }

            let connected = state
                .selected_connection
                .and_then(|i| state.connection_status.get(i))
                .map(|s| *s == ConnectionStatus::Connected)
                .unwrap_or(false);

            ui.label(if connected {
                "🟢 Connected"
            } else {
                "🔴 Disconnected"
            });
        });
    });
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
