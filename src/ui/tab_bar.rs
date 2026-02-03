use crate::tabs::TabManager;
use egui::Context;

/// 渲染顶部标签栏
pub fn render_tab_bar(tab_manager: &mut TabManager, ctx: &Context) {
    egui::TopBottomPanel::top("tab_bar")
        .exact_height(32.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(4.0);
                
                // 记录需要执行的操作
                let mut tab_to_close: Option<usize> = None;
                let mut tab_to_switch: Option<usize> = None;
                
                // 渲染所有标签
                for (index, tab) in tab_manager.tabs.iter().enumerate() {
                    let is_active = index == tab_manager.active_tab_index;
                    
                    // 标签样式
                    let bg_color = if is_active {
                        egui::Color32::from_rgb(100, 149, 237) // 活跃标签：蓝色
                    } else {
                        egui::Color32::from_rgb(60, 60, 70) // 非活跃：深灰
                    };
                    
                    let text_color = if is_active {
                        egui::Color32::WHITE
                    } else {
                        egui::Color32::from_rgb(200, 200, 200)
                    };
                    
                    // 标签按钮组
                    ui.group(|ui| {
                        ui.visuals_mut().widgets.inactive.weak_bg_fill = bg_color;
                        ui.visuals_mut().widgets.hovered.weak_bg_fill = 
                            egui::Color32::from_rgb(80, 120, 200);
                        
                        ui.horizontal(|ui| {
                            // 标签标题按钮
                            let title = if tab.title.len() > 20 {
                                format!("{}...", &tab.title[..17])
                            } else {
                                tab.title.clone()
                            };
                            
                            let tab_button = ui.add(
                                egui::Button::new(
                                    egui::RichText::new(&title)
                                        .color(text_color)
                                        .size(13.0)
                                )
                                .fill(bg_color)
                            );
                            
                            if tab_button.clicked() {
                                tab_to_switch = Some(index);
                            }
                            
                            // 鼠标悬停时显示完整标题和右键菜单
                            let mut response = if !tab.title.is_empty() {
                                tab_button.on_hover_text(&tab.title)
                            } else {
                                tab_button
                            };
                            
                            // 右键菜单
                            response.context_menu(|ui| {
                                if ui.button("✏️ 重命名").clicked() {
                                    // TODO: 实现重命名对话框
                                    ui.close_menu();
                                }
                                if ui.button("📋 复制标签").clicked() {
                                    // TODO: 实现复制功能
                                    ui.close_menu();
                                }
                                ui.separator();
                                if ui.button("❌ 关闭标签").clicked() {
                                    tab_to_close = Some(index);
                                    ui.close_menu();
                                }
                                if ui.button("❌ 关闭其他标签").clicked() {
                                    // TODO: 实现关闭其他标签
                                    ui.close_menu();
                                }
                            });
                            
                            // 关闭按钮（只有多于 1 个标签时显示）
                            if tab_manager.count() > 1 {
                                let close_button = ui.add(
                                    egui::Button::new(
                                        egui::RichText::new("×")
                                            .color(text_color)
                                            .size(16.0)
                                    )
                                    .fill(bg_color)
                                    .frame(false)
                                    .min_size(egui::vec2(16.0, 16.0))
                                );
                                
                                if close_button.clicked() {
                                    tab_to_close = Some(index);
                                }
                                
                                close_button.on_hover_text("关闭标签");
                            }
                        });
                    });
                    
                    ui.add_space(2.0);
                }
                
                // 新建标签按钮
                let new_tab_button = ui.add(
                    egui::Button::new(
                        egui::RichText::new("➕")
                            .size(14.0)
                    )
                    .min_size(egui::vec2(28.0, 24.0))
                );
                
                if new_tab_button.clicked() {
                    let next_id = tab_manager.tabs.len() + 1;
                    tab_manager.create_tab(format!("Tab {}", next_id));
                }
                
                new_tab_button.on_hover_text("新建标签 (Ctrl+T)");
                
                // 右侧信息区域
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(format!("{}/{}", 
                            tab_manager.active_tab_index + 1,
                            tab_manager.count()
                        ))
                        .size(11.0)
                        .weak()
                    );
                });
                
                // 执行延迟操作
                if let Some(index) = tab_to_close {
                    tab_manager.close_tab(index);
                }
                if let Some(index) = tab_to_switch {
                    tab_manager.switch_to(index);
                }
            });
        });
}

/// 处理标签页键盘快捷键
pub fn handle_tab_shortcuts(tab_manager: &mut TabManager, ctx: &Context) {
    ctx.input(|i| {
        let ctrl = i.modifiers.ctrl || i.modifiers.command;
        let shift = i.modifiers.shift;
        
        // Ctrl+T: 新建标签
        if ctrl && i.key_pressed(egui::Key::T) {
            let next_id = tab_manager.tabs.len() + 1;
            tab_manager.create_tab(format!("Tab {}", next_id));
        }
        
        // Ctrl+W: 关闭当前标签
        if ctrl && i.key_pressed(egui::Key::W) {
            tab_manager.close_tab(tab_manager.active_tab_index);
        }
        
        // Ctrl+Tab: 下一个标签
        if ctrl && i.key_pressed(egui::Key::Tab) && !shift {
            tab_manager.next_tab();
        }
        
        // Ctrl+Shift+Tab: 上一个标签
        if ctrl && shift && i.key_pressed(egui::Key::Tab) {
            tab_manager.previous_tab();
        }
        
        // Ctrl+1-9: 快速切换到第 N 个标签
        for (n, key) in [
            (1, egui::Key::Num1), (2, egui::Key::Num2), (3, egui::Key::Num3),
            (4, egui::Key::Num4), (5, egui::Key::Num5), (6, egui::Key::Num6),
            (7, egui::Key::Num7), (8, egui::Key::Num8), (9, egui::Key::Num9),
        ] {
            if ctrl && i.key_pressed(key) {
                if n <= tab_manager.count() {
                    tab_manager.switch_to(n - 1);
                }
            }
        }
    });
}
