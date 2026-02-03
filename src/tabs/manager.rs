use super::tab::Tab;

/// 标签页管理器
#[derive(Debug)]
pub struct TabManager {
    /// 所有标签页
    pub tabs: Vec<Tab>,
    
    /// 当前活跃的标签页索引
    pub active_tab_index: usize,
    
    /// 下一个标签 ID（自增）
    next_tab_id: usize,
    
    /// 最大标签页数量限制
    max_tabs: usize,
}

impl TabManager {
    /// 创建新的标签管理器（自动创建第一个标签）
    pub fn new() -> Self {
        let mut manager = Self {
            tabs: Vec::new(),
            active_tab_index: 0,
            next_tab_id: 1,
            max_tabs: 50,
        };
        
        // 默认创建一个标签页
        manager.create_tab("Tab 1".to_string());
        manager
    }
    
    /// 创建新标签页
    pub fn create_tab(&mut self, title: String) -> usize {
        if self.tabs.len() >= self.max_tabs {
            eprintln!("⚠️ 已达到最大标签页数量限制 ({})", self.max_tabs);
            return self.active_tab_index;
        }
        
        let tab_id = self.next_tab_id;
        self.next_tab_id += 1;
        
        let tab = Tab::new(tab_id, title);
        self.tabs.push(tab);
        
        // 切换到新创建的标签
        self.active_tab_index = self.tabs.len() - 1;
        
        tab_id
    }
    
    /// 关闭指定标签页
    pub fn close_tab(&mut self, index: usize) -> bool {
        // 至少保留一个标签页
        if self.tabs.len() <= 1 {
            return false;
        }
        
        if index >= self.tabs.len() {
            return false;
        }
        
        self.tabs.remove(index);
        
        // 调整活跃标签索引
        if self.active_tab_index >= self.tabs.len() {
            self.active_tab_index = self.tabs.len() - 1;
        } else if self.active_tab_index > index {
            self.active_tab_index -= 1;
        }
        
        true
    }
    
    /// 切换到指定标签
    pub fn switch_to(&mut self, index: usize) {
        if index < self.tabs.len() {
            if let Some(tab) = self.tabs.get_mut(self.active_tab_index) {
                tab.mark_active();
            }
            self.active_tab_index = index;
        }
    }
    
    /// 切换到下一个标签（循环）
    pub fn next_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.active_tab_index = (self.active_tab_index + 1) % self.tabs.len();
            if let Some(tab) = self.tabs.get_mut(self.active_tab_index) {
                tab.mark_active();
            }
        }
    }
    
    /// 切换到上一个标签（循环）
    pub fn previous_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.active_tab_index = if self.active_tab_index == 0 {
                self.tabs.len() - 1
            } else {
                self.active_tab_index - 1
            };
            if let Some(tab) = self.tabs.get_mut(self.active_tab_index) {
                tab.mark_active();
            }
        }
    }
    
    /// 获取当前活跃标签（不可变引用）
    pub fn active_tab(&self) -> Option<&Tab> {
        self.tabs.get(self.active_tab_index)
    }
    
    /// 获取当前活跃标签（可变引用）
    pub fn active_tab_mut(&mut self) -> Option<&mut Tab> {
        self.tabs.get_mut(self.active_tab_index)
    }
    
    /// 获取标签总数
    pub fn count(&self) -> usize {
        self.tabs.len()
    }
    
    /// 设置最大标签数
    pub fn set_max_tabs(&mut self, max: usize) {
        self.max_tabs = max.max(1); // 至少允许 1 个标签
    }
}

impl Default for TabManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tab_manager_creation() {
        let manager = TabManager::new();
        assert_eq!(manager.tabs.len(), 1);
        assert_eq!(manager.active_tab_index, 0);
        assert_eq!(manager.tabs[0].title, "Tab 1");
    }
    
    #[test]
    fn test_create_tab() {
        let mut manager = TabManager::new();
        assert_eq!(manager.tabs.len(), 1);
        
        let new_id = manager.create_tab("Test Tab".to_string());
        assert_eq!(new_id, 2);
        assert_eq!(manager.tabs.len(), 2);
        assert_eq!(manager.active_tab_index, 1);
        assert_eq!(manager.tabs[1].title, "Test Tab");
    }
    
    #[test]
    fn test_close_tab() {
        let mut manager = TabManager::new();
        manager.create_tab("Tab 2".to_string());
        manager.create_tab("Tab 3".to_string());
        
        assert_eq!(manager.tabs.len(), 3);
        assert!(manager.close_tab(1));
        assert_eq!(manager.tabs.len(), 2);
    }
    
    #[test]
    fn test_cannot_close_last_tab() {
        let mut manager = TabManager::new();
        assert!(!manager.close_tab(0));
        assert_eq!(manager.tabs.len(), 1);
    }
    
    #[test]
    fn test_switch_to_tab() {
        let mut manager = TabManager::new();
        manager.create_tab("Tab 2".to_string());
        manager.create_tab("Tab 3".to_string());
        
        manager.switch_to(0);
        assert_eq!(manager.active_tab_index, 0);
        
        manager.switch_to(2);
        assert_eq!(manager.active_tab_index, 2);
    }
    
    #[test]
    fn test_next_tab() {
        let mut manager = TabManager::new();
        manager.create_tab("Tab 2".to_string());
        manager.create_tab("Tab 3".to_string());
        
        manager.switch_to(0);
        manager.next_tab();
        assert_eq!(manager.active_tab_index, 1);
        
        manager.next_tab();
        assert_eq!(manager.active_tab_index, 2);
        
        // 循环到第一个
        manager.next_tab();
        assert_eq!(manager.active_tab_index, 0);
    }
    
    #[test]
    fn test_previous_tab() {
        let mut manager = TabManager::new();
        manager.create_tab("Tab 2".to_string());
        manager.create_tab("Tab 3".to_string());
        
        manager.switch_to(2);
        manager.previous_tab();
        assert_eq!(manager.active_tab_index, 1);
        
        manager.previous_tab();
        assert_eq!(manager.active_tab_index, 0);
        
        // 循环到最后一个
        manager.previous_tab();
        assert_eq!(manager.active_tab_index, 2);
    }
    
    #[test]
    fn test_active_tab() {
        let mut manager = TabManager::new();
        manager.create_tab("Tab 2".to_string());
        
        manager.switch_to(1);
        let active = manager.active_tab();
        assert!(active.is_some());
        assert_eq!(active.unwrap().title, "Tab 2");
    }
    
    #[test]
    fn test_max_tabs_limit() {
        let mut manager = TabManager::new();
        manager.set_max_tabs(3);
        
        manager.create_tab("Tab 2".to_string());
        manager.create_tab("Tab 3".to_string());
        
        // 应该达到限制
        assert_eq!(manager.tabs.len(), 3);
        
        // 尝试创建第 4 个标签应该失败
        manager.create_tab("Tab 4".to_string());
        assert_eq!(manager.tabs.len(), 3);
    }
    
    #[test]
    fn test_close_active_tab_adjustment() {
        let mut manager = TabManager::new();
        manager.create_tab("Tab 2".to_string());
        manager.create_tab("Tab 3".to_string());
        
        manager.switch_to(2); // 激活最后一个标签
        assert!(manager.close_tab(2));
        
        // 活跃索引应该调整到新的最后一个
        assert_eq!(manager.active_tab_index, 1);
    }
}
