// 简化的 SFTP 浏览器，仅用于测试标签绑定

use crate::state::AppState;
use crate::types::ConnectionStatus;
use eframe::egui;

pub fn render_file_browser(state: &mut AppState, ctx: &egui::Context) {
    if !state.show_file_browser {
        return;
    }

    egui::Window::new("📁 SFTP File Browser (v1.0.0 - Tab Binding)")
        .default_width(600.0)
        .default_height(400.0)
        .show(ctx, |ui| {
            ui.heading("📁 SFTP File Browser");
            ui.separator();
            
            ui.horizontal(|ui| {
                ui.label("Active tab:");
                if let Some(tab) = state.tab_manager.active_tab() {
                    ui.label(&tab.title);
                } else {
                    ui.label("None");
                }
            });

            ui.separator();

            // 检查活跃标签是否有 SFTP 状态
            if let Some(tab) = state.tab_manager.active_tab() {
                if tab.state.sftp_state.is_some() {
                    ui.label("✅ SFTP state initialized for this tab");
                } else {
                    ui.label("⚠️  No SFTP state yet (will be initialized on first use)");
                }
            }

            ui.separator();

            if ui.button("❌ Close").clicked() {
                state.show_file_browser = false;
            }
        });
}
