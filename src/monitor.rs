use sysinfo::{System, Disks, Networks};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// 系统监控管理器
pub struct SystemMonitor {
    system: Arc<Mutex<System>>,
    disks: Arc<Mutex<Disks>>,
    networks: Arc<Mutex<Networks>>,
    last_update: Arc<Mutex<Instant>>,
    update_interval: Duration,
}

impl SystemMonitor {
    /// 创建新的系统监控
    pub fn new() -> Self {
        Self {
            system: Arc::new(Mutex::new(System::new_all())),
            disks: Arc::new(Mutex::new(Disks::new_with_refreshed_list())),
            networks: Arc::new(Mutex::new(Networks::new_with_refreshed_list())),
            last_update: Arc::new(Mutex::new(Instant::now())),
            update_interval: Duration::from_secs(1),
        }
    }

    /// 更新系统信息（如果距离上次更新超过间隔时间）
    pub fn update(&self) {
        let mut last = self.last_update.lock().unwrap();
        if last.elapsed() < self.update_interval {
            return;
        }

        // 更新系统信息
        self.system.lock().unwrap().refresh_all();
        self.disks.lock().unwrap().refresh();
        self.networks.lock().unwrap().refresh();

        *last = Instant::now();
    }

    /// 强制立即更新
    pub fn force_update(&self) {
        self.system.lock().unwrap().refresh_all();
        self.disks.lock().unwrap().refresh();
        self.networks.lock().unwrap().refresh();
        *self.last_update.lock().unwrap() = Instant::now();
    }

    /// 获取 CPU 使用率（全局平均）
    pub fn cpu_usage(&self) -> f32 {
        let system = self.system.lock().unwrap();
        system.global_cpu_info().cpu_usage()
    }

    /// 获取每个 CPU 核心的使用率
    pub fn cpu_usage_per_core(&self) -> Vec<f32> {
        let system = self.system.lock().unwrap();
        system.cpus().iter().map(|cpu| cpu.cpu_usage()).collect()
    }

    /// 获取内存使用情况 (used, total) 单位：字节
    pub fn memory_usage(&self) -> (u64, u64) {
        let system = self.system.lock().unwrap();
        (system.used_memory(), system.total_memory())
    }

    /// 获取内存使用率百分比
    pub fn memory_usage_percent(&self) -> f32 {
        let (used, total) = self.memory_usage();
        if total == 0 {
            return 0.0;
        }
        (used as f64 / total as f64 * 100.0) as f32
    }

    /// 获取交换空间使用情况 (used, total)
    pub fn swap_usage(&self) -> (u64, u64) {
        let system = self.system.lock().unwrap();
        (system.used_swap(), system.total_swap())
    }

    /// 获取磁盘使用情况 Vec<(挂载点, 已用, 总量)>
    pub fn disk_usage(&self) -> Vec<(String, u64, u64)> {
        let disks = self.disks.lock().unwrap();
        disks
            .iter()
            .map(|disk| {
                let mount_point = disk.mount_point().to_string_lossy().to_string();
                let total = disk.total_space();
                let available = disk.available_space();
                let used = total.saturating_sub(available);
                (mount_point, used, total)
            })
            .collect()
    }

    /// 获取网络使用情况 (总接收, 总发送) 单位：字节
    pub fn network_usage(&self) -> (u64, u64) {
        let networks = self.networks.lock().unwrap();
        let mut total_received = 0u64;
        let mut total_transmitted = 0u64;

        for (_interface_name, data) in networks.iter() {
            total_received += data.received();
            total_transmitted += data.transmitted();
        }

        (total_received, total_transmitted)
    }

    /// 获取每个网络接口的使用情况 Vec<(接口名, 接收, 发送)>
    pub fn network_usage_per_interface(&self) -> Vec<(String, u64, u64)> {
        let networks = self.networks.lock().unwrap();
        networks
            .iter()
            .map(|(name, data)| {
                (
                    name.clone(),
                    data.received(),
                    data.transmitted(),
                )
            })
            .collect()
    }

    /// 获取系统运行时间（秒）
    pub fn uptime(&self) -> u64 {
        System::uptime()
    }

    /// 获取系统信息摘要
    pub fn summary(&self) -> SystemSummary {
        let (mem_used, mem_total) = self.memory_usage();
        let (net_rx, net_tx) = self.network_usage();
        let disks = self.disk_usage();

        SystemSummary {
            cpu_usage: self.cpu_usage(),
            memory_used: mem_used,
            memory_total: mem_total,
            memory_percent: self.memory_usage_percent(),
            network_received: net_rx,
            network_transmitted: net_tx,
            disk_info: disks,
            uptime: self.uptime(),
        }
    }
}

impl Default for SystemMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// 系统信息摘要
#[derive(Debug, Clone)]
pub struct SystemSummary {
    pub cpu_usage: f32,
    pub memory_used: u64,
    pub memory_total: u64,
    pub memory_percent: f32,
    pub network_received: u64,
    pub network_transmitted: u64,
    pub disk_info: Vec<(String, u64, u64)>, // (挂载点, 已用, 总量)
    pub uptime: u64,
}

impl SystemSummary {
    /// 格式化内存大小
    pub fn format_memory(bytes: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if bytes >= GB {
            format!("{:.2} GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.2} MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.2} KB", bytes as f64 / KB as f64)
        } else {
            format!("{} B", bytes)
        }
    }

    /// 格式化运行时间
    pub fn format_uptime(seconds: u64) -> String {
        let days = seconds / 86400;
        let hours = (seconds % 86400) / 3600;
        let minutes = (seconds % 3600) / 60;

        if days > 0 {
            format!("{}天 {}小时 {}分钟", days, hours, minutes)
        } else if hours > 0 {
            format!("{}小时 {}分钟", hours, minutes)
        } else {
            format!("{}分钟", minutes)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitor_creation() {
        let monitor = SystemMonitor::new();
        let summary = monitor.summary();

        // 基本健全性检查
        assert!(summary.cpu_usage >= 0.0);
        assert!(summary.memory_total > 0);
        println!("CPU: {:.1}%", summary.cpu_usage);
        println!("Memory: {} / {}", 
            SystemSummary::format_memory(summary.memory_used),
            SystemSummary::format_memory(summary.memory_total));
    }

    #[test]
    fn test_format_memory() {
        assert_eq!(SystemSummary::format_memory(1024), "1.00 KB");
        assert_eq!(SystemSummary::format_memory(1024 * 1024), "1.00 MB");
        assert_eq!(SystemSummary::format_memory(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_format_uptime() {
        assert_eq!(SystemSummary::format_uptime(60), "1分钟");
        assert_eq!(SystemSummary::format_uptime(3600), "1小时 0分钟");
        assert_eq!(SystemSummary::format_uptime(86400), "1天 0小时 0分钟");
    }
}
