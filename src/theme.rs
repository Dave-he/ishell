use egui::{Color32, Context, Visuals};
use serde::{Deserialize, Serialize};

/// 主题类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Theme {
    Dark,
    Light,
    Custom,
}

impl Default for Theme {
    fn default() -> Self {
        Theme::Dark
    }
}

impl std::fmt::Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Theme::Dark => write!(f, "深色"),
            Theme::Light => write!(f, "浅色"),
            Theme::Custom => write!(f, "自定义"),
        }
    }
}

/// 主题管理器
pub struct ThemeManager;

impl ThemeManager {
    /// 应用主题到上下文
    pub fn apply(ctx: &Context, theme_name: &str) {
        let theme = match theme_name {
            "light" => Theme::Light,
            "dark" => Theme::Dark,
            "custom" => Theme::Custom,
            _ => Theme::Dark,
        };

        let visuals = match theme {
            Theme::Dark => Self::dark_theme(),
            Theme::Light => Self::light_theme(),
            Theme::Custom => Self::custom_theme(),
        };

        ctx.set_visuals(visuals);
    }

    /// 深色主题
    fn dark_theme() -> Visuals {
        Visuals::dark()
    }

    /// 浅色主题
    fn light_theme() -> Visuals {
        Visuals::light()
    }

    /// 自定义主题
    fn custom_theme() -> Visuals {
        let mut visuals = Visuals::dark();

        // 背景颜色
        visuals.window_fill = Color32::from_rgb(25, 25, 35);
        visuals.panel_fill = Color32::from_rgb(30, 30, 40);

        // 组件颜色
        visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(35, 35, 45);
        visuals.widgets.noninteractive.weak_bg_fill = Color32::from_rgb(30, 30, 40);
        visuals.widgets.noninteractive.bg_stroke.color = Color32::from_rgb(50, 50, 60);

        visuals.widgets.inactive.bg_fill = Color32::from_rgb(40, 40, 50);
        visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(35, 35, 45);
        visuals.widgets.inactive.bg_stroke.color = Color32::from_rgb(60, 60, 70);

        visuals.widgets.hovered.bg_fill = Color32::from_rgb(50, 50, 70);
        visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(45, 45, 60);
        visuals.widgets.hovered.bg_stroke.color = Color32::from_rgb(80, 80, 100);

        visuals.widgets.active.bg_fill = Color32::from_rgb(60, 60, 90);
        visuals.widgets.active.weak_bg_fill = Color32::from_rgb(55, 55, 80);
        visuals.widgets.active.bg_stroke.color = Color32::from_rgb(100, 100, 140);

        // 选中颜色
        visuals.selection.bg_fill = Color32::from_rgb(100, 149, 237); // Cornflower Blue
        visuals.selection.stroke.color = Color32::from_rgb(120, 169, 255);

        // 超链接颜色
        visuals.hyperlink_color = Color32::from_rgb(100, 149, 237);

        // 文本颜色
        visuals.widgets.noninteractive.fg_stroke.color = Color32::from_rgb(220, 220, 230);
        visuals.widgets.inactive.fg_stroke.color = Color32::from_rgb(200, 200, 210);
        visuals.widgets.hovered.fg_stroke.color = Color32::from_rgb(240, 240, 250);
        visuals.widgets.active.fg_stroke.color = Color32::WHITE;

        // 窗口边框
        visuals.window_stroke.color = Color32::from_rgb(60, 60, 70);
        visuals.window_rounding = 8.0.into();
        visuals.window_shadow.spread = 16.0;
        visuals.window_shadow.blur = 24.0;

        visuals
    }

    /// 获取主题名称列表
    pub fn theme_names() -> Vec<&'static str> {
        vec!["dark", "light", "custom"]
    }

    /// 应用字体大小
    pub fn apply_font_size(ctx: &Context, font_size: f32) {
        let mut style = (*ctx.style()).clone();
        
        // 更新所有文本样式的字体大小
        for (_, font_id) in style.text_styles.iter_mut() {
            font_id.size = font_size;
        }

        ctx.set_style(style);
    }

    /// 应用终端字体大小
    pub fn apply_terminal_font_size(_ctx: &Context, _font_size: f32) {
        // 终端字体大小通常通过 TextEdit 的单独配置来设置
        // 这里预留接口供将来扩展
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_display() {
        assert_eq!(format!("{}", Theme::Dark), "深色");
        assert_eq!(format!("{}", Theme::Light), "浅色");
        assert_eq!(format!("{}", Theme::Custom), "自定义");
    }

    #[test]
    fn test_theme_names() {
        let names = ThemeManager::theme_names();
        assert_eq!(names.len(), 3);
        assert!(names.contains(&"dark"));
        assert!(names.contains(&"light"));
        assert!(names.contains(&"custom"));
    }
}
