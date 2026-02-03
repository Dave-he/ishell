use crate::state::AppState;
use crate::types::{AiProviderType, SettingsPage};
use egui::Context;

/// 渲染设置窗口
pub fn render_settings_window(state: &mut AppState, ctx: &Context) {
    if !state.show_settings {
        return;
    }

    egui::Window::new("⚙️ 设置")
        .default_width(700.0)
        .default_height(500.0)
        .resizable(true)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                // 左侧菜单
                egui::SidePanel::left("settings_menu")
                    .resizable(false)
                    .exact_width(150.0)
                    .show_inside(ui, |ui| {
                        ui.selectable_value(&mut state.settings_page, SettingsPage::General, "🎨 常规");
                        ui.selectable_value(&mut state.settings_page, SettingsPage::Appearance, "🖌️ 外观");
                        ui.selectable_value(&mut state.settings_page, SettingsPage::Terminal, "💻 终端");
                        ui.selectable_value(&mut state.settings_page, SettingsPage::Ai, "🤖 AI");
                        ui.selectable_value(&mut state.settings_page, SettingsPage::History, "📜 历史");
                    });

                // 右侧内容区
                egui::CentralPanel::default().show_inside(ui, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        match state.settings_page {
                            SettingsPage::General => render_general_settings(state, ui),
                            SettingsPage::Appearance => render_appearance_settings(state, ui),
                            SettingsPage::Terminal => render_terminal_settings(state, ui),
                            SettingsPage::Ai => render_ai_settings(state, ui),
                            SettingsPage::History => render_history_settings(state, ui),
                        }
                    });
                });
            });

            ui.separator();

            // 底部按钮
            ui.horizontal(|ui| {
                if ui.button("✅ 保存").clicked() {
                    save_settings(state);
                    state.show_settings = false;
                }
                if ui.button("❌ 取消").clicked() {
                    // 重新加载配置以撤销更改
                    if let Ok(config) = state.config_manager.load_config() {
                        state.config = config;
                    }
                    state.show_settings = false;
                }
                if ui.button("🔄 恢复默认").clicked() {
                    state.config.settings = crate::types::Settings::default();
                }
            });
        });
}

/// 常规设置
fn render_general_settings(state: &mut AppState, ui: &mut egui::Ui) {
    ui.heading("常规设置");
    ui.separator();
    ui.add_space(10.0);

    ui.checkbox(
        &mut state.config.settings.auto_save_config,
        "自动保存配置"
    );
    ui.label("启用后，配置更改将自动保存");

    ui.add_space(10.0);

    ui.checkbox(
        &mut state.config.settings.confirm_before_delete,
        "删除前确认"
    );
    ui.label("在删除文件或连接时显示确认对话框");
}

/// 外观设置
fn render_appearance_settings(state: &mut AppState, ui: &mut egui::Ui) {
    ui.heading("外观设置");
    ui.separator();
    ui.add_space(10.0);

    ui.label("主题:");
    ui.horizontal(|ui| {
        ui.selectable_value(&mut state.config.settings.theme, "dark".to_string(), "🌙 深色");
        ui.selectable_value(&mut state.config.settings.theme, "light".to_string(), "☀️ 浅色");
    });

    ui.add_space(10.0);

    ui.label("字体大小:");
    ui.add(egui::Slider::new(&mut state.config.settings.font_size, 10.0..=24.0).text("pt"));

    ui.add_space(10.0);

    ui.label("终端字体大小:");
    ui.add(egui::Slider::new(&mut state.config.settings.terminal_font_size, 10.0..=24.0).text("pt"));
}

/// 终端设置
fn render_terminal_settings(state: &mut AppState, ui: &mut egui::Ui) {
    ui.heading("终端设置");
    ui.separator();
    ui.add_space(10.0);

    ui.label("回滚行数:");
    ui.add(egui::Slider::new(&mut state.config.settings.terminal_scrollback, 100..=50000).text("行"));
    ui.label("终端可以保留的历史输出行数");

    ui.add_space(10.0);

    ui.checkbox(
        &mut state.config.settings.terminal_word_wrap,
        "自动换行"
    );
    ui.label("长行是否自动换行显示");
}

/// AI 设置
fn render_ai_settings(state: &mut AppState, ui: &mut egui::Ui) {
    ui.heading("AI 设置");
    ui.separator();
    ui.add_space(10.0);

    ui.label("默认 AI 提供商:");
    ui.horizontal(|ui| {
        ui.selectable_value(&mut state.config.settings.default_ai_provider, AiProviderType::Ollama, "🦙 Ollama");
        ui.selectable_value(&mut state.config.settings.default_ai_provider, AiProviderType::OpenAI, "🤖 OpenAI");
        ui.selectable_value(&mut state.config.settings.default_ai_provider, AiProviderType::Google, "🔷 Google");
    });

    ui.add_space(10.0);

    // Ollama 配置
    ui.group(|ui| {
        ui.label(egui::RichText::new("Ollama 配置").strong());
        ui.separator();

        ui.checkbox(&mut state.config.ai.ollama.enabled, "启用 Ollama");

        ui.label("Base URL:");
        ui.text_edit_singleline(&mut state.config.ai.ollama.base_url);

        ui.label("模型:");
        ui.text_edit_singleline(&mut state.config.ai.ollama.model);
    });

    ui.add_space(10.0);

    // OpenAI 配置
    ui.group(|ui| {
        ui.label(egui::RichText::new("OpenAI 配置").strong());
        ui.separator();

        ui.checkbox(&mut state.config.ai.openai.enabled, "启用 OpenAI");

        ui.label("API Key:");
        if let Some(key) = &mut state.config.ai.openai.api_key {
            ui.add(egui::TextEdit::singleline(key).password(true));
        } else {
            let mut temp_key = String::new();
            if ui.add(egui::TextEdit::singleline(&mut temp_key).password(true).hint_text("输入 API Key")).changed() {
                if !temp_key.is_empty() {
                    state.config.ai.openai.api_key = Some(temp_key);
                }
            }
        }

        ui.label("模型:");
        ui.text_edit_singleline(&mut state.config.ai.openai.model);
    });

    ui.add_space(10.0);

    // Google 配置
    ui.group(|ui| {
        ui.label(egui::RichText::new("Google AI 配置").strong());
        ui.separator();

        ui.checkbox(&mut state.config.ai.google.enabled, "启用 Google AI");

        ui.label("API Key:");
        if let Some(key) = &mut state.config.ai.google.api_key {
            ui.add(egui::TextEdit::singleline(key).password(true));
        } else {
            let mut temp_key = String::new();
            if ui.add(egui::TextEdit::singleline(&mut temp_key).password(true).hint_text("输入 API Key")).changed() {
                if !temp_key.is_empty() {
                    state.config.ai.google.api_key = Some(temp_key);
                }
            }
        }

        ui.label("模型:");
        ui.text_edit_singleline(&mut state.config.ai.google.model);
    });
}

/// 历史设置
fn render_history_settings(state: &mut AppState, ui: &mut egui::Ui) {
    ui.heading("历史设置");
    ui.separator();
    ui.add_space(10.0);

    ui.label("最大历史记录数:");
    ui.add(egui::Slider::new(&mut state.config.settings.history_max_size, 100..=10000).text("条"));
    ui.label("保存的最大命令历史记录数量");

    ui.add_space(10.0);

    ui.checkbox(
        &mut state.config.settings.save_history_on_exit,
        "退出时保存历史"
    );
    ui.label("程序退出时自动保存命令历史");

    ui.add_space(10.0);

    // 历史统计信息
    let stats = state.command_history.stats();
    ui.group(|ui| {
        ui.label(egui::RichText::new("历史统计").strong());
        ui.separator();
        ui.label(format!("总命令数: {}", stats.total_commands));
        ui.label(format!("唯一命令数: {}", stats.unique_commands));
        ui.label(format!("连接数: {}", stats.unique_connections));
    });

    ui.add_space(10.0);

    if ui.button("🗑️ 清空所有历史").clicked() {
        state.command_history.clear();
    }
}

/// 保存设置
fn save_settings(state: &mut AppState) {
    // 更新命令历史最大大小
    state.command_history = state.command_history.clone().with_max_size(state.config.settings.history_max_size);
    
    // 保存配置
    if let Err(e) = state.config_manager.save_config(&mut state.config) {
        eprintln!("Failed to save settings: {}", e);
    }
    
    // 如果设置了自动保存历史
    if state.config.settings.save_history_on_exit {
        if let Some(config_dir) = dirs::config_dir() {
            let history_path = config_dir.join("ishell").join("history.json");
            if let Err(e) = state.command_history.save(&history_path) {
                eprintln!("Failed to save history: {}", e);
            }
        }
    }
}
