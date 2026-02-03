use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 命令历史条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub command: String,
    pub timestamp: String,
    pub connection: String,
}

/// 命令历史管理器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandHistory {
    pub commands: Vec<HistoryEntry>,
    max_size: usize,
}

impl CommandHistory {
    /// 创建新的命令历史
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            max_size: 1000,
        }
    }

    /// 设置最大历史记录数
    pub fn with_max_size(mut self, max_size: usize) -> Self {
        self.max_size = max_size;
        self
    }

    /// 添加命令到历史
    pub fn add(&mut self, command: String, connection: String) {
        // 忽略空命令
        if command.trim().is_empty() {
            return;
        }

        let entry = HistoryEntry {
            command,
            timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            connection,
        };

        self.commands.push(entry);

        // 保持最大大小
        if self.commands.len() > self.max_size {
            self.commands.remove(0);
        }
    }

    /// 获取所有历史记录（最新的在前）
    pub fn get_all(&self) -> Vec<&HistoryEntry> {
        self.commands.iter().rev().collect()
    }

    /// 模糊搜索命令
    pub fn search(&self, query: &str) -> Vec<&HistoryEntry> {
        if query.is_empty() {
            return self.get_all();
        }

        use fuzzy_matcher::skim::SkimMatcherV2;
        use fuzzy_matcher::FuzzyMatcher;

        let matcher = SkimMatcherV2::default();
        let mut results: Vec<(&HistoryEntry, i64)> = self
            .commands
            .iter()
            .filter_map(|entry| {
                matcher
                    .fuzzy_match(&entry.command, query)
                    .map(|score| (entry, score))
            })
            .collect();

        // 按匹配分数降序排序
        results.sort_by(|a, b| b.1.cmp(&a.1));
        results.into_iter().map(|(entry, _)| entry).collect()
    }

    /// 按连接过滤历史
    pub fn filter_by_connection(&self, connection: &str) -> Vec<&HistoryEntry> {
        self.commands
            .iter()
            .filter(|entry| entry.connection == connection)
            .rev()
            .collect()
    }

    /// 获取最近的 n 条命令
    pub fn get_recent(&self, n: usize) -> Vec<&HistoryEntry> {
        self.commands.iter().rev().take(n).collect()
    }

    /// 清空历史
    pub fn clear(&mut self) {
        self.commands.clear();
    }

    /// 保存历史到文件
    pub fn save(&self, path: &PathBuf) -> Result<(), std::io::Error> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)
    }

    /// 从文件加载历史
    pub fn load(path: &PathBuf) -> Result<Self, std::io::Error> {
        if !path.exists() {
            return Ok(Self::new());
        }

        let content = std::fs::read_to_string(path)?;
        serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    /// 获取统计信息
    pub fn stats(&self) -> HistoryStats {
        let total = self.commands.len();
        let unique_commands = self
            .commands
            .iter()
            .map(|e| &e.command)
            .collect::<std::collections::HashSet<_>>()
            .len();
        let unique_connections = self
            .commands
            .iter()
            .map(|e| &e.connection)
            .collect::<std::collections::HashSet<_>>()
            .len();

        HistoryStats {
            total_commands: total,
            unique_commands,
            unique_connections,
        }
    }
}

impl Default for CommandHistory {
    fn default() -> Self {
        Self::new()
    }
}

/// 历史统计信息
#[derive(Debug, Clone)]
pub struct HistoryStats {
    pub total_commands: usize,
    pub unique_commands: usize,
    pub unique_connections: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_command() {
        let mut history = CommandHistory::new();
        history.add("ls -la".to_string(), "server1".to_string());
        history.add("pwd".to_string(), "server1".to_string());

        assert_eq!(history.commands.len(), 2);
    }

    #[test]
    fn test_search() {
        let mut history = CommandHistory::new();
        history.add("ls -la".to_string(), "server1".to_string());
        history.add("pwd".to_string(), "server1".to_string());
        history.add("cd /home".to_string(), "server1".to_string());

        let results = history.search("ls");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].command, "ls -la");
    }

    #[test]
    fn test_max_size() {
        let mut history = CommandHistory::new().with_max_size(3);
        history.add("cmd1".to_string(), "s1".to_string());
        history.add("cmd2".to_string(), "s1".to_string());
        history.add("cmd3".to_string(), "s1".to_string());
        history.add("cmd4".to_string(), "s1".to_string());

        assert_eq!(history.commands.len(), 3);
        assert_eq!(history.commands[0].command, "cmd2");
    }

    #[test]
    fn test_filter_by_connection() {
        let mut history = CommandHistory::new();
        history.add("cmd1".to_string(), "server1".to_string());
        history.add("cmd2".to_string(), "server2".to_string());
        history.add("cmd3".to_string(), "server1".to_string());

        let filtered = history.filter_by_connection("server1");
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_stats() {
        let mut history = CommandHistory::new();
        history.add("ls".to_string(), "server1".to_string());
        history.add("ls".to_string(), "server2".to_string());
        history.add("pwd".to_string(), "server1".to_string());

        let stats = history.stats();
        assert_eq!(stats.total_commands, 3);
        assert_eq!(stats.unique_commands, 2);
        assert_eq!(stats.unique_connections, 2);
    }
}
