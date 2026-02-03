use std::time::SystemTime;
use crate::state::TabState;

/// 单个标签页
#[derive(Debug)]
pub struct Tab {
    /// 唯一标识符
    pub id: usize,
    
    /// 标签标题
    pub title: String,
    
    /// 关联的连接 ID（如果已连接）
    pub connection_id: Option<usize>,
    
    /// 标签页的状态（终端输出、SSH 会话等）
    pub state: TabState,
    
    /// 创建时间
    pub created_at: SystemTime,
    
    /// 最后活跃时间
    pub last_active: SystemTime,
}

impl Tab {
    /// 创建新标签页
    pub fn new(id: usize, title: String) -> Self {
        Self {
            id,
            title,
            connection_id: None,
            state: TabState::new(),
            created_at: SystemTime::now(),
            last_active: SystemTime::now(),
        }
    }
    
    /// 标记为活跃（更新最后活跃时间）
    pub fn mark_active(&mut self) {
        self.last_active = SystemTime::now();
    }
    
    /// 更新标题
    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }
    
    /// 连接到服务器
    pub fn connect(&mut self, connection_id: usize, connection_name: &str) {
        self.connection_id = Some(connection_id);
        self.title = format!("📡 {}", connection_name);
    }
    
    /// 断开连接
    pub fn disconnect(&mut self) {
        self.connection_id = None;
        self.title = format!("Tab {}", self.id);
    }
    
    /// 检查是否已连接
    pub fn is_connected(&self) -> bool {
        self.connection_id.is_some()
    }
    
    /// 获取不活跃时长（秒）
    pub fn inactive_duration(&self) -> u64 {
        self.last_active
            .elapsed()
            .unwrap_or_default()
            .as_secs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tab_creation() {
        let tab = Tab::new(1, "Test Tab".to_string());
        assert_eq!(tab.id, 1);
        assert_eq!(tab.title, "Test Tab");
        assert_eq!(tab.connection_id, None);
        assert!(!tab.is_connected());
    }
    
    #[test]
    fn test_tab_connect() {
        let mut tab = Tab::new(1, "Tab 1".to_string());
        tab.connect(5, "Production Server");
        
        assert_eq!(tab.connection_id, Some(5));
        assert!(tab.is_connected());
        assert_eq!(tab.title, "📡 Production Server");
    }
    
    #[test]
    fn test_tab_disconnect() {
        let mut tab = Tab::new(1, "Tab 1".to_string());
        tab.connect(5, "Production Server");
        tab.disconnect();
        
        assert_eq!(tab.connection_id, None);
        assert!(!tab.is_connected());
        assert_eq!(tab.title, "Tab 1");
    }
    
    #[test]
    fn test_mark_active() {
        let mut tab = Tab::new(1, "Tab 1".to_string());
        let initial_time = tab.last_active;
        
        std::thread::sleep(std::time::Duration::from_millis(10));
        tab.mark_active();
        
        assert!(tab.last_active > initial_time);
    }
    
    #[test]
    fn test_set_title() {
        let mut tab = Tab::new(1, "Old Title".to_string());
        tab.set_title("New Title".to_string());
        
        assert_eq!(tab.title, "New Title");
    }
}
