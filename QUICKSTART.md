# iShell 快速入门指南

10 分钟上手 iShell！

---

## 📦 安装

### 前置要求

- Rust 1.70+ ([安装指南](https://www.rust-lang.org/tools/install))
- macOS/Linux/Windows 系统

### 克隆并运行

```bash
# 1. 克隆仓库
git clone <repository-url>
cd ishell

# 2. 运行（自动编译）
./run.sh
```

或者手动编译：

```bash
cargo run --release
```

**首次编译**: 约 2 分钟  
**启动时间**: 约 2 秒

---

## 🚀 第一次使用

### Step 1: 创建 SSH 连接

1. 点击左侧面板的 **"➕ New Connection"**
2. 填写连接信息：
   ```
   Name: My Server
   Host: 192.168.1.100  (你的服务器IP)
   Port: 22
   Username: admin  (你的用户名)
   ```

3. 选择认证方式：

   **选项 A: 密码认证**
   - Password: 输入你的密码
   - 密码会自动加密存储

   **选项 B: 私钥认证**
   - Use Private Key: 勾选
   - Key Path: `/Users/你的用户名/.ssh/id_rsa`
   - Key Passphrase: 如果密钥有密码，输入密码

4. 点击 **"✅ Create"**

### Step 2: 连接服务器

1. 在左侧连接列表选择刚创建的连接
2. 点击 **"🔗 Connect"**
3. 等待状态变为 🟢 **Connected**

### Step 3: 执行命令

在终端面板输入命令：

```bash
# 查看当前目录
pwd

# 列出文件
ls -la

# 查看磁盘使用
df -h

# 查看内存
free -h
```

按 **Enter** 或点击 **"▶ Run"** 执行。

---

## 🤖 使用 AI 助手

### 快速开始

1. 在右侧 AI 面板输入你的问题：
   ```
   How do I find files larger than 100MB?
   ```

2. 点击 **"📤 Send"**

3. AI 会生成命令：
   ```bash
   find / -type f -size +100M -exec ls -lh {} \;
   ```

4. 复制命令到终端执行

### AI 示例对话

**你**: find large log files

**AI**:
```bash
find /var/log -type f -size +10M -exec ls -lh {} \;
```

**你**: backup mysql database

**AI**:
```bash
mysqldump -u root -p database_name > backup_$(date +%Y%m%d).sql
```

**你**: fix permission denied error

**AI**:
```bash
# 检查文件权限
ls -l filename

# 修改权限
chmod 644 filename

# 或添加执行权限
chmod +x script.sh
```

---

## ⚙️ 配置 AI 提供商

iShell 支持 3 种 AI 提供商：

### 1. Ollama (推荐，免费本地运行)

```bash
# macOS 安装
brew install ollama

# 启动服务
ollama serve

# 下载模型
ollama pull llama3.2
```

iShell 会自动检测并使用本地 Ollama。

### 2. OpenAI

编辑配置文件 `~/.ishell/config.toml`:

```toml
[ai.openai]
enabled = true
api_key_encrypted = "sk-your-api-key-here"  # 首次保存自动加密
model = "gpt-4o-mini"
```

重启 iShell，切换到 OpenAI 图标即可使用。

### 3. Google Gemini

编辑配置文件:

```toml
[ai.google]
enabled = true
api_key_encrypted = "AIza-your-api-key-here"
model = "gemini-1.5-flash"
```

---

## 📁 配置文件详解

配置文件位置: `~/.ishell/config.toml`

### 查看配置

```bash
cat ~/.ishell/config.toml
```

### 配置示例

```toml
version = "0.2.0"

# SSH 连接配置
[[connections]]
name = "Production Server"
host = "192.168.1.100"
port = 22
username = "admin"
password_encrypted = "base64..."  # 自动加密

[[connections]]
name = "Dev Server"
host = "dev.example.com"
port = 22
username = "developer"
key_path = "/Users/me/.ssh/id_rsa"

# Ollama 配置（本地免费）
[ai.ollama]
enabled = true
base_url = "http://localhost:11model = "llama3.2"

# OpenAI 配置
[ai.openai]
enabled = false
api_key_encrypted = ""
model = "gpt-4o-mini"

# Google Gemini 配置
[ai.google]
enabled = false
api_key_encrypted = ""
model = "gemini-1.5-flash"

# 应用设置
[settings]
default_ai_provider = "Ollama"
theme = "dark"
terminal_font_size = 14.0
```

### 手动编辑配置

```bash
# 编辑配置
vim ~/.ishell/config.toml

# 或使用其他编辑器
nano ~/.ishell/config.toml
```

**注意**: 密码和 API 密钥首次保存时会自动加密。

---

## 🔐 安全最佳实践

### 1. 保护配置文件

```bash
# 设置配置文件权限（仅所有者可读写）
chmod 600 ~/.ishell/config.toml
```

### 2. 使用私钥认证（推荐）

比密码更安全：

```bash
# 生成 SSH 密钥对
ssh-keygen -t rsa -b 4096 -C "your_email@example.com"

# 复制公钥到服务器
ssh-copy-id user@server

# 在 iShell 中使用私钥连接
```

### 3. 定期备份配置

```bash
# 备份配置
cp ~/.ishell/config.toml ~/ishell_backup_$(date +%Y%m%d).toml

# 安全导出（不含密码）
# 使用 iShell 的导出功能
```

---

## 🎯 常见任务

### 任务 1: 批量执行命令

```bash
# 连接到服务器
# 执行：
for server in server1 server2 server3; do
  echo "Updating $server"
  ssh $server 'sudo apt update && sudo apt upgrade -y'
done
```

### 任务 2: 监控日志

```bash
# 实时查看日志
tail -f /var/log/syslog

# 搜索错误
grep -i error /var/log/application.log
```

### 任务 3: 文件搜索

```bash
# 查找大文件
find / -type f -size +100M 2>/dev/null

# 查找最近修改的文件
find /var/log -type f -mtime -1
```

---

## 🐛 常见问题

### Q: 连接失败怎么办？

**A**: 检查以下几点：

1. 服务器地址和端口正确吗？
   ```bash
   # 测试连接
   ssh -p 22 user@host
   ```

2. 防火墙是否阻止？
   ```bash
   # 检查端口
   telnet host 22
   ```

3. 密码/密钥正确吗？

### Q: AI 没有响应？

**A**: 

**Ollama**:
```bash
# 检查服务
curl http://localhost:11434/api/tags

# 重启服务
ollama serve
```

**OpenAI/Gemini**:
- 检查 API key 是否正确
- 检查网络连接
- 查看配置文件 `enabled = true`

### Q: 配置文件在哪里？

**A**:
```bash
# 查看配置文件位置
ls -la ~/.ishell/config.toml

# 编辑配置
vim ~/.ishell/config.toml
```

### Q: 如何重置配置？

**A**:
```bash
# 备份当前配置
cp ~/.ishell/config.toml ~/config.backup

# 删除配置（重启 iShell 会创建默认配置）
rm ~/.ishell/config.toml
```

---

## 📚 进阶使用

### 1. 多连接管理

```toml
# config.toml

[[connections]]
name = "Web Server"
host = "web.example.com"
port = 22
username = "deploy"

[[connections]]
name = "Database Server"
host = "db.example.com"
port = 22
username = "postgres"

[[connections]]
name = "Backup Server"
host = "backup.local"
port = 2222  # 自定义端口
username = "backup"
```

### 2. 自定义 AI 模型

```toml
[ai.ollama]
enabled = true
base_url = "http://localhost:11434"
model = "codellama:latest"  # 使用代码专用模型

[ai.openai]
enabled = true
model = "gpt-4-turbo"  # 使用更强大的模型
```

### 3. 快捷操作

- **快速连接**: 双击连接名称
- **快速命令**: 使用命令历史（上/下箭头）
- **复制输出**: 选择文本后自动复制
- **AI 快捷操作**: 使用右侧快捷按钮

---

## 🚀 下一步

### 学习更多

- 📖 [完整文档](README.md)
- 🏗️ [架构设计](docs/agent/architecture.md)
- 🧪 [测试指南](docs/TESTING.md)
- 👨‍💻 [开发指南](docs/DEV_GUIDE_v0.2.0.md)

### 贡献

欢迎贡献代码和反馈！

```bash
# Fork 并克隆
git clone your-fork-url
cd ishell

# 创建功能分支
git checkout -b feature/my-feature

# 提交更改
git commit -m "Add my feature"
git push origin feature/my-feature
```

---

## 💡 提示与技巧

### 提示 1: 使用 AI 生成复杂命令

不知道怎么写命令？问 AI！

```
你: "How to compress all log files older than 7 days?"

AI: "find /var/log -name '*.log' -mtime +7 -exec gzip {} \;"
```

### 提示 2: 保存常用命令

在配置文件中保存：

```toml
[settings]
common_commands = [
  "sudo systemctl status nginx",
  "df -h",
  "free -m"
]
```

### 提示 3: 使用别名

在服务器 `~/.bashrc` 添加：

```bash
alias ll='ls -lah'
alias update='sudo apt update && sudo apt upgrade -y'
```

---

**开始使用 iShell，让 SSH 管理更智能！** 🎉

**最后更新**: 2026-02-02  
**版本**: v0.2.0
