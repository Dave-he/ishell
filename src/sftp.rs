use ssh2::Sftp as Ssh2Sftp;
use std::path::Path;
use std::io::{Read, Write};
use crate::types::{FileEntry, Result};

/// SFTP 客户端
pub struct SftpClient {
    sftp: Ssh2Sftp,
}

impl SftpClient {
    /// 创建 SFTP 客户端
    pub fn new(sftp: Ssh2Sftp) -> Self {
        Self { sftp }
    }
    
    /// 列出目录内容
    pub fn list_dir(&self, path: &str) -> Result<Vec<FileEntry>> {
        let path_buf = Path::new(path);
        let entries = self.sftp.readdir(path_buf)?;
        
        let mut file_entries = Vec::new();
        for (path, stat) in entries {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("?")
                .to_string();
            
            let path_str = path.to_string_lossy().to_string();
            let is_dir = stat.is_dir();
            let size = stat.size.unwrap_or(0);
            
            let modified = stat.mtime.map(|mtime| {
                std::time::UNIX_EPOCH + std::time::Duration::from_secs(mtime)
            });
            
            let permissions = stat.perm.map(|perm| format!("{:o}", perm));
            
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
    
    /// 上传文件
    pub fn upload_file<F>(&self, local: &Path, remote: &str, mut progress_callback: F) -> Result<()>
    where
        F: FnMut(f32),
    {
        let mut local_file = std::fs::File::open(local)?;
        let file_size = local_file.metadata()?.len();
        
        let mut remote_file = self.sftp.create(Path::new(remote))?;
        
        let mut buffer = vec![0u8; 8192]; // 8KB buffer
        let mut total_written = 0u64;
        
        loop {
            let bytes_read = local_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            
            remote_file.write_all(&buffer[..bytes_read])?;
            total_written += bytes_read as u64;
            
            let progress = (total_written as f32 / file_size as f32) * 100.0;
            progress_callback(progress);
        }
        
        Ok(())
    }
    
    /// 下载文件
    pub fn download_file<F>(&self, remote: &str, local: &Path, mut progress_callback: F) -> Result<()>
    where
        F: FnMut(f32),
    {
        let mut remote_file = self.sftp.open(Path::new(remote))?;
        let file_size = remote_file.stat()?.size.unwrap_or(0);
        
        let mut local_file = std::fs::File::create(local)?;
        
        let mut buffer = vec![0u8; 8192];
        let mut total_read = 0u64;
        
        loop {
            let bytes_read = remote_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            
            local_file.write_all(&buffer[..bytes_read])?;
            total_read += bytes_read as u64;
            
            let progress = if file_size > 0 {
                (total_read as f32 / file_size as f32) * 100.0
            } else {
                0.0
            };
            progress_callback(progress);
        }
        
        Ok(())
    }
    
    /// 删除文件
    pub fn delete_file(&self, path: &str) -> Result<()> {
        self.sftp.unlink(Path::new(path))?;
        Ok(())
    }
    
    /// 删除目录
    pub fn delete_dir(&self, path: &str) -> Result<()> {
        self.sftp.rmdir(Path::new(path))?;
        Ok(())
    }
    
    /// 创建目录
    pub fn create_dir(&self, path: &str) -> Result<()> {
        self.sftp.mkdir(Path::new(path), 0o755)?;
        Ok(())
    }
    
    /// 重命名/移动文件
    pub fn rename(&self, old_path: &str, new_path: &str) -> Result<()> {
        self.sftp.rename(Path::new(old_path), Path::new(new_path), None)?;
        Ok(())
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sftp_client_creation() {
        // 这个测试需要真实的 SSH 连接，通常会被忽略
        // 或者使用 mock
    }
    
    // 注意：完整的 SFTP 测试需要真实的 SSH 服务器
    // 可以使用 Docker 容器进行集成测试
}
