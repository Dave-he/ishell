use crate::types::{FileEntry, Result};
use ssh2::Sftp as Ssh2Sftp;
use std::io::{Read, Write};
use std::path::Path;

/// SFTP 客户端封装
pub struct SftpClient {
    sftp: Ssh2Sftp,
}

impl SftpClient {
    /// 创建新的 SFTP 客户端
    pub fn new(sftp: Ssh2Sftp) -> Self {
        Self { sftp }
    }

    /// 列出目录内容
    pub fn list_dir(&self, path: &str) -> Result<Vec<FileEntry>> {
        let path = if path.is_empty() { "." } else { path };
        
        let entries = self.sftp.readdir(std::path::Path::new(path))?;
        
        let mut file_entries = Vec::new();
        for (path, stat) in entries {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            
            let path_str = path.to_string_lossy().to_string();
            let is_dir = stat.is_dir();
            let size = stat.size.unwrap_or(0);
            
            // 转换时间戳
            let modified = stat.mtime.and_then(|mtime| {
                std::time::UNIX_EPOCH.checked_add(std::time::Duration::from_secs(mtime))
            });
            
            // 转换权限
            let permissions = stat.perm.map(|p| format!("{:o}", p));
            
            file_entries.push(FileEntry {
                name,
                path: path_str,
                is_dir,
                size,
                modified,
                permissions,
            });
        }
        
        // 排序：目录在前，然后按名称
        file_entries.sort_by(|a, b| {
            match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            }
        });
        
        Ok(file_entries)
    }

    /// 上传文件（带进度回调）
    pub fn upload_file<F>(
        &self,
        local: &Path,
        remote: &str,
        mut progress_callback: F,
    ) -> Result<()>
    where
        F: FnMut(f32),
    {
        // 打开本地文件
        let mut local_file = std::fs::File::open(local)?;
        let file_size = local_file.metadata()?.len();
        
        // 创建远程文件
        let mut remote_file = self.sftp.create(std::path::Path::new(remote))?;
        
        // 分块传输
        let mut buffer = vec![0u8; 8192]; // 8KB 缓冲区
        let mut total_sent = 0u64;
        
        loop {
            let bytes_read = local_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            
            remote_file.write_all(&buffer[..bytes_read])?;
            total_sent += bytes_read as u64;
            
            // 报告进度
            let progress = if file_size > 0 {
                (total_sent as f32 / file_size as f32).min(1.0)
            } else {
                1.0
            };
            progress_callback(progress);
        }
        
        remote_file.flush()?;
        Ok(())
    }

    /// 下载文件（带进度回调）
    pub fn download_file<F>(
        &self,
        remote: &str,
        local: &Path,
        mut progress_callback: F,
    ) -> Result<()>
    where
        F: FnMut(f32),
    {
        // 打开远程文件
        let mut remote_file = self.sftp.open(std::path::Path::new(remote))?;
        let file_size = remote_file.stat()?.size.unwrap_or(0);
        
        // 创建本地文件
        let mut local_file = std::fs::File::create(local)?;
        
        // 分块传输
        let mut buffer = vec![0u8; 8192]; // 8KB 缓冲区
        let mut total_received = 0u64;
        
        loop {
            let bytes_read = remote_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            
            local_file.write_all(&buffer[..bytes_read])?;
            total_received += bytes_read as u64;
            
            // 报告进度
            let progress = if file_size > 0 {
                (total_received as f32 / file_size as f32).min(1.0)
            } else {
                1.0
            };
            progress_callback(progress);
        }
        
        local_file.flush()?;
        Ok(())
    }

    /// 删除文件或目录
    pub fn delete(&self, path: &str) -> Result<()> {
        let path_obj = std::path::Path::new(path);
        let stat = self.sftp.stat(path_obj)?;
        
        if stat.is_dir() {
            // 递归删除目录
            self.delete_dir_recursive(path)?;
        } else {
            // 删除文件
            self.sftp.unlink(path_obj)?;
        }
        
        Ok(())
    }

    /// 递归删除目录
    fn delete_dir_recursive(&self, path: &str) -> Result<()> {
        // 列出目录内容
        let entries = self.sftp.readdir(std::path::Path::new(path))?;
        
        // 删除所有子项
        for (entry_path, stat) in entries {
            let entry_path_str = entry_path.to_string_lossy();
            
            // 跳过 . 和 ..
            if entry_path_str.ends_with("/.") || entry_path_str.ends_with("/..") {
                continue;
            }
            
            if stat.is_dir() {
                self.delete_dir_recursive(&entry_path_str)?;
            } else {
                self.sftp.unlink(&entry_path)?;
            }
        }
        
        // 删除空目录
        self.sftp.rmdir(std::path::Path::new(path))?;
        Ok(())
    }

    /// 创建目录
    pub fn create_dir(&self, path: &str) -> Result<()> {
        self.sftp.mkdir(
            std::path::Path::new(path),
            0o755, // rwxr-xr-x
        )?;
        Ok(())
    }

    /// 获取文件/目录状态
    pub fn stat(&self, path: &str) -> Result<FileEntry> {
        let stat = self.sftp.stat(std::path::Path::new(path))?;
        
        let name = std::path::Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        
        let is_dir = stat.is_dir();
        let size = stat.size.unwrap_or(0);
        let modified = stat.mtime.and_then(|mtime| {
            std::time::UNIX_EPOCH.checked_add(std::time::Duration::from_secs(mtime))
        });
        let permissions = stat.perm.map(|p| format!("{:o}", p));
        
        Ok(FileEntry {
            name,
            path: path.to_string(),
            is_dir,
            size,
            modified,
            permissions,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 注意：这些测试需要真实的 SSH 连接，通常在集成测试中运行
    // 这里提供测试框架结构

    #[test]
    fn test_sftp_client_creation() {
        // 测试 SFTP 客户端创建
        // 需要模拟或真实 SSH 会话
    }

    #[test]
    fn test_list_dir() {
        // 测试目录列表
    }

    #[test]
    fn test_file_operations() {
        // 测试文件上传/下载/删除
    }
}
