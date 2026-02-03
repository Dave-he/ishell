//! 多标签页管理模块
//! 
//! 提供多标签页功能，允许用户同时打开多个 SSH 会话。
//! 
//! # 示例
//! 
//! ```
//! use ishell::tabs::TabManager;
//! 
//! let mut manager = TabManager::new();
//! manager.create_tab("My Server".to_string());
//! manager.switch_to(1);
//! ```

pub mod manager;
pub mod tab;

pub use manager::TabManager;
pub use tab::Tab;
