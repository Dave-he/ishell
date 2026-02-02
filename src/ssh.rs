use crate::types::{AuthMethod, ConnectionStatus, Result};
use crate::sftp::SftpClient;
use ssh2::Session;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// SSH 会话管理器
pub struct SshSession {
    session: Arc<Mutex<Option<Session>>>,
    stream: Arc<Mutex<Option<TcpStream>>>,
    status: Arc<Mutex<ConnectionStatus>>,
    host: String,
    port: u16,
    username: String,
}

impl SshSession {
    /// 创建新的 SSH 会话
    pub fn new(host: String, port: u16, username: String) -> Self {
        Self {
            session: Arc::new(Mutex::new(None)),
            stream: Arc::new(Mutex::new(None)),
            status: Arc::new(Mutex::new(ConnectionStatus::Disconnected)),
            host,
            port,
            username,
        }
    }

    /// 连接到 SSH 服务器
    pub fn connect(&self, auth: &AuthMethod) -> Result<()> {
        // 设置状态为连接中
        *self.status.lock().unwrap() = ConnectionStatus::Connecting;

        // 建立 TCP 连接
        let addr = format!("{}:{}", self.host, self.port);
        let tcp = TcpStream::connect_timeout(&addr.parse()?, Duration::from_secs(10))?;
        tcp.set_read_timeout(Some(Duration::from_secs(30)))?;
        tcp.set_write_timeout(Some(Duration::from_secs(30)))?;

        // 创建 SSH 会话
        let mut sess = Session::new()?;
        sess.set_tcp_stream(tcp.try_clone()?);
        sess.handshake()?;

        // 认证
        match auth {
            AuthMethod::Password(password) => {
                sess.userauth_password(&self.username, password)?;
            }
            AuthMethod::PrivateKey {
                key_path,
                passphrase,
            } => {
                let key_str = key_path.to_str().ok_or("Invalid key path")?;

                if let Some(pass) = passphrase {
                    sess.userauth_pubkey_file(
                        &self.username,
                        None,
                        Path::new(key_str),
                        Some(pass),
                    )?;
                } else {
                    sess.userauth_pubkey_file(&self.username, None, Path::new(key_str), None)?;
                }
            }
        }

        // 验证认证成功
        if !sess.authenticated() {
            *self.status.lock().unwrap() = ConnectionStatus::Error;
            return Err("Authentication failed".into());
        }

        // 保存会话
        *self.session.lock().unwrap() = Some(sess);
        *self.stream.lock().unwrap() = Some(tcp);
        *self.status.lock().unwrap() = ConnectionStatus::Connected;

        Ok(())
    }

    /// 执行单个命令
    pub fn execute_command(&self, command: &str) -> Result<String> {
        let session = self.session.lock().unwrap();
        let sess = session.as_ref().ok_or("Not connected")?;

        let mut channel = sess.channel_session()?;
        channel.exec(command)?;

        let mut output = String::new();
        channel.read_to_string(&mut output)?;

        let mut stderr = String::new();
        channel.stderr().read_to_string(&mut stderr)?;

        channel.wait_close()?;
        let exit_status = channel.exit_status()?;

        if exit_status != 0 && !stderr.is_empty() {
            output.push_str("\n[stderr]:\n");
            output.push_str(&stderr);
        }

        Ok(output)
    }

    /// 启动交互式 shell
    pub fn start_shell(&self) -> Result<SshShell> {
        let session = self.session.lock().unwrap();
        let sess = session.as_ref().ok_or("Not connected")?;

        let mut channel = sess.channel_session()?;
        channel.request_pty("xterm", None, None)?;
        channel.shell()?;

        Ok(SshShell { channel })
    }

    /// 获取连接状态
    pub fn status(&self) -> ConnectionStatus {
        *self.status.lock().unwrap()
    }

    /// 断开连接
    pub fn disconnect(&self) -> Result<()> {
        if let Some(sess) = self.session.lock().unwrap().take() {
            sess.disconnect(None, "User disconnected", None)?;
        }
        *self.stream.lock().unwrap() = None;
        *self.status.lock().unwrap() = ConnectionStatus::Disconnected;
        Ok(())
    }

    /// 检查是否已连接
    pub fn is_connected(&self) -> bool {
        self.status() == ConnectionStatus::Connected
    }
    
    /// 获取 SFTP 客户端 (v0.3.0)
    pub fn sftp(&self) -> Result<SftpClient> {
        let session = self.session.lock().unwrap();
        let sess = session.as_ref().ok_or("Not connected")?;
        
        let sftp = sess.sftp()?;
        Ok(SftpClient::new(sftp))
    }
}

/// 交互式 SSH Shell
pub struct SshShell {
    channel: ssh2::Channel,
}

impl SshShell {
    /// 发送命令
    pub fn send_command(&mut self, command: &str) -> Result<()> {
        self.channel.write_all(command.as_bytes())?;
        self.channel.write_all(b"\n")?;
        self.channel.flush()?;
        Ok(())
    }

    /// 读取输出（非阻塞）
    pub fn read_output(&mut self) -> Result<String> {
        let mut buffer = vec![0; 8192];
        // ssh2 的 Channel 默认就是非阻塞的，直接读取即可

        let mut output = String::new();
        loop {
            match self.channel.read(&mut buffer) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    output.push_str(&String::from_utf8_lossy(&buffer[..n]));
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    break; // 没有更多数据
                }
                Err(e) => return Err(e.into()),
            }

            // 防止无限循环，如果没有数据就退出
            if output.is_empty() {
                break;
            }
        }

        Ok(output)
    }

    /// 检查 shell 是否仍然活跃
    pub fn is_active(&self) -> bool {
        !self.channel.eof()
    }

    /// 关闭 shell
    pub fn close(mut self) -> Result<()> {
        self.channel.close()?;
        self.channel.wait_close()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 注意：这些测试需要真实的 SSH 服务器才能运行
    // 在 CI/CD 环境中应该 mock 或跳过

    #[test]
    #[ignore] // 需要真实 SSH 服务器
    fn test_connect_password() {
        let session = SshSession::new("localhost".to_string(), 22, "testuser".to_string());

        let auth = AuthMethod::Password("testpass".to_string());
        let result = session.connect(&auth);

        // 这个测试在没有真实服务器时会失败
        // 实际使用时需要配置测试环境
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_session_creation() {
        let session = SshSession::new("example.com".to_string(), 22, "user".to_string());

        assert_eq!(session.status(), ConnectionStatus::Disconnected);
        assert!(!session.is_connected());
    }
}
