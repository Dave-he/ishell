# iShell v0.2.0 开发文档

## 📋 项目概述

本文档记录了 iShell v0.2.0 的开发进展和使用指南。

### 版本信息
- **版本**: 0.2.0  
- **状态**: 核心模块开发完成 ✅
- **日期**: 2026-02-02

---

## ✅ 已完成功能

### 1. 核心基础模块

#### 📦 依赖配置 (`Cargo.toml`)

已添加所有必要的依赖：

```toml
# SSH 支持
ssh2 = "0.9"
tokio = { version = "1", features = ["full", "rt-multi-thread"] }

# AI API 集成
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
async-openai = "0.23"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
async-trait = "0.1"

# 配置持久化
toml = "0.8"
dirs = "5.0"

# 密码加密
aes-gcm = "0.10"
base64 = "0.22"
rand = "0.8"
whoami = "1.5"

# 异步运行时
futures = "0.3"
```

---

### 2. 类型系统 (`src/types.rs`)

定义了完整的类型系统：

#### 核心类型

```rust
// SSH 认证方法
pub enum AuthMethod {
    Password(String),
    PrivateKey {
        key_path: PathBuf,
        passphrase: Option<String>,
    },
}

// SSH 连接配置
pub struct SshConfig {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth: Option<AuthMethod>,
    pub password_encrypted: Option<String>,
    pub key_path: Option<String>,
    pub key_passphrase_encrypted: Option<String>,
}

// AI 提供商类型
#[derive(Hash)]
pub enum AiProviderType {
    Ollama,
    OpenAI,
    Google,
}

// AI 消息
pub struct AiMessage {
    pub role: String,
    pub content: String,
}

// 应用完整配置
pub struct AppConfig {
    pub version: String,
    pub connections: Vec<SshConfig>,
    pub ai: AiConfig,
    pub settings: Settings,
}
```

---

### 3. 加密模块 (`src/crypto.rs`)

实现了 AES-256-GCM 加密：

#### 功能

- ✅ 密码加密/解密
- ✅ 基于机器的密钥生成（确保同一台机器密钥一致）
- ✅ Base64 编码存储
- ✅ Unicode 支持

#### 使用示例

```rust
use crate::crypto::PasswordEncryptor;

let encryptor = PasswordEncryptor::new()?;

// 加密
let encrypted = encryptor.encrypt("my_password")?;
// 返回: "base64_encoded_string"

// 解密
let decrypted = encryptor.decrypt(&encrypted)?;
// 返回: "my_password"
```

#### 安全特性

- 使用 AES-256-GCM（AEAD 加密）
- 每次加密使用随机 nonce
- 密钥基于用户名和主机名派生
- 自动 padding 和认证

---

### 4. SSH 模块 (`src/ssh.rs`)

实现了真实的 SSH 连接功能：

#### 功能

- ✅ 密码认证
- ✅ 私钥认证（支持密码保护的密钥）
- ✅ 单命令执行
- ✅ 交互式 Shell
- ✅ 连接状态管理
- ✅ 错误处理与重连

#### 使用示例

**密码认证连接**:

```rust
use crate::ssh::SshSession;
use crate::types::AuthMethod;

let session = SshSession::new(
    "192.168.1.100".to_string(),
    22,
    "admin".to_string(),
);

let auth = AuthMethod::Password("secret".to_string());
session.connect(&auth)?;

// 执行命令
let output = session.execute_command("ls -la")?;
println!("{}", output);

session.disconnect()?;
```

**密钥认证连接**:

```rust
let auth = AuthMethod::PrivateKey {
    key_path: PathBuf::from("/Users/user/.ssh/id_rsa"),
    passphrase: Some("key_password".to_string()),
};

session.connect(&auth)?;
```

**交互式 Shell**:

```rust
let mut shell = session.start_shell()?;

// 发送命令
shell.send_command("cd /var/log")?;
shell.send_command("tail -f syslog")?;

// 读取输出
let output = shell.read_output()?;
println!("{}", output);

shell.close()?;
```

---

### 5. AI 集成模块 (`src/ai.rs`)

实现了三个 AI 提供商的集成：

#### 支持的 AI 服务

1. **Ollama** (本地运行)
2. **OpenAI GPT**
3. **Google Gemini**

#### 统一接口

```rust
#[async_trait]
pub trait AiProvider {
    async fn chat(&self, messages: &[AiMessage]) -> Result<String>;
    fn provider_type(&self) -> AiProviderType;
}
```

#### 使用示例

**Ollama**:

```rust
use crate::ai::OllamaProvider;

let provider = OllamaProvider::new(
    "http://localhost:11434".to_string(),
    "llama3.2".to_string(),
);

let messages = vec![
    AiMessage::user("find large files".to_string()),
];

let response = provider.chat(&messages).await?;
```

**OpenAI**:

```rust
use crate::ai::OpenAiProvider;

let provider = OpenAiProvider::new(
    "sk-...".to_string(),
    "gpt-4o-mini".to_string(),
);

let response = provider.chat(&messages).await?;
```

**Google Gemini**:

```rust
use crate::ai::GoogleProvider;

let provider = GoogleProvider::new(
    "AIza...".to_string(),
    "gemini-1.5-flash".to_string(),
);

let response = provider.chat(&messages).await?;
```

#### AI 管理器（统一管理多个提供商）

```rust
use crate::ai::AiManager;

let mut manager = AiManager::new();

// 注册提供商
manager.register_provider(Box::new(ollama_provider));
manager.register_provider(Box::new(openai_provider));
manager.register_provider(Box::new(google_provider));

// 切换提供商
manager.set_current_provider(AiProviderType::Ollama);

// 发送消息
let response = manager.chat(&messages).await?;
```

---

### 6. 配置管理模块 (`src/config.rs`)

实现了配置持久化：

#### 功能

- ✅ TOML 格式配置文件
- ✅ 自动加密敏感信息
- ✅ 存储在 `~/.ishell/config.toml`
- ✅ 配置导出（不含敏感信息）
- ✅ 配置备份

#### 配置文件格式

```toml
version = "0.2.0"

[[connections]]
name = "Production Server"
host = "192.168.1.100"
port = 22
username = "admin"
password_encrypted = "..." # Base64 加密
key_path = "~/.ssh/id_rsa"
key_passphrase_encrypted = "..."

[ai.ollama]
enabled = true
base_url = "http://localhost:11434"
model = "llama3.2"

[ai.openai]
enabled = false
api_key_encrypted = "..."
model = "gpt-4o-mini"

[ai.google]
enabled = false
api_key_encrypted = "..."
model = "gemini-1.5-flash"

[settings]
default_ai_provider = "Ollama"
theme = "dark"
terminal_font_size = 14.0
```

#### 使用示例

```rust
use crate::config::ConfigManager;

let manager = ConfigManager::new()?;

// 加载配置
let config = manager.load_config()?;

// 添加连接
let conn = SshConfig::new(
    "My Server".to_string(),
    "192.168.1.100".to_string(),
    22,
    "user".to_string(),
);
manager.add_connection(&mut config, conn)?;

// 保存配置
manager.save_config(&mut config)?;

// 备份配置
let backup_path = manager.backup_config()?;
println!("Backup saved to: {:?}", backup_path);
```

---

## 🏗️ 项目结构

```
ishell/
├── Cargo.toml          # 依赖配置 ✅
├── src/
│   ├── main.rs         # 程序入口 ✅
│   ├── types.rs        # 类型定义 ✅
│   ├── crypto.rs       # 加密模块 ✅
│   ├── ssh.rs          # SSH 模块 ✅
│   ├── ai.rs           # AI 集成 ✅
│   ├── config.rs       # 配置管理 ✅
│   └── app.rs          # UI 应用 (待集成)
└── assets/
    └── icon.png        # 应用图标 ✅
```

---

## 🚧 下一步工作

### 重构 `app.rs` - 集成新模块

#### 需要实现的功能

1. **SSH 集成**
   - 使用真实 SSH 连接替换模拟
   - 支持密码和密钥认证
   - 在后台线程执行 SSH 操作（避免 UI 阻塞）

2. **AI 集成**
   - 使用 AiManager 替换模拟响应
   - 异步调用 AI API
   - 显示 Loading 状态

3. **配置持久化**
   - 启动时加载配置
   - 保存连接信息
   - 保存 AI 配置

#### 实现要点

**异步处理** - 由于 egui 是同步的，需要使用 channel 与异步任务通信：

```rust
// 在 App 结构中添加
use tokio::sync::mpsc;

struct App {
    // SSH 相关
    ssh_rx: mpsc::Receiver<SshMessage>,
    ssh_tx: mpsc::Sender<SshCommand>,
    
    // AI 相关
    ai_rx: mpsc::Receiver<String>,
    ai_tx: mpsc::Sender<Vec<AiMessage>>,
    
    // 配置
    config_manager: ConfigManager,
    config: AppConfig,
}
```

**后台任务示例**:

```rust
// SSH 任务
tokio::spawn(async move {
    while let Some(cmd) = ssh_cmd_rx.recv().await {
        match cmd {
            SshCommand::Connect(config) => {
                let session = SshSession::new(...);
                // 连接并发送结果
            }
            SshCommand::Execute(command) => {
                let output = session.execute_command(&command);
                // 发送输出
            }
        }
    }
});

// AI 任务
tokio::spawn(async move {
    while let Some(messages) = ai_input_rx.recv().await {
        let response = ai_manager.chat(&messages).await;
        ai_output_tx.send(response).await;
    }
});
```

---

## 📊 当前状态总结

### ✅ 已完成 (约70%)

- [x] 所有核心模块代码编写完成
- [x] 编译通过（无错误）
- [x] 类型系统完整
- [x] SSH 连接功能实现
- [x] AI 三个提供商集成
- [x] 配置持久化和加密

### 🚧 进行中 (约20%)

- [ ] 集成到 app.rs
- [ ] 异步通信架构
- [ ] UI 更新

### 📋 待完成 (约10%)

- [ ] 端到端测试
- [ ] 文档更新
- [ ] 发布准备

---

## 🎯 验收标准

### SSH 功能

- [ ] 能够使用密码连接真实 SSH 服务器
- [ ] 能够使用密钥连接真实 SSH 服务器
- [ ] 终端可以执行任意命令并正确显示输出
- [ ] 支持交互式命令（如 vi、top）
- [ ] 连接断开后能够正确重连

### AI 功能

- [ ] Ollama 本地 API 能够正常调用并返回结果
- [ ] OpenAI API 能够正常调用（需有效 API key）
- [ ] Google Gemini API 能够正常调用（需有效 API key）
- [ ] 能够在三种 AI 之间无缝切换
- [ ] AI 响应能够正确显示在 UI 中

### 配置功能

- [ ] 配置文件能够正确保存到 `~/.ishell/config.toml`
- [ ] 应用重启后能够加载保存的连接
- [ ] 密码能够安全加密存储
- [ ] AI 密钥能够安全存储

---

## 📚 API 文档

### SSH API

#### `SshSession::connect(auth: &AuthMethod)`

连接到 SSH 服务器。

**参数**:
- `auth`: 认证方法（密码或密钥）

**返回**: `Result<()>`

**示例**:
```rust
let auth = AuthMethod::Password("password".to_string());
session.connect(&auth)?;
```

#### `SshSession::execute_command(command: &str)`

执行单个命令。

**参数**:
- `command`: 要执行的命令

**返回**: `Result<String>` - 命令输出

**示例**:
```rust
let output = session.execute_command("ls -la")?;
```

### AI API

#### `AiProvider::chat(messages: &[AiMessage])`

发送聊天消息。

**参数**:
- `messages`: 消息历史

**返回**: `Result<String>` - AI 响应

**示例**:
```rust
let messages = vec![AiMessage::user("Hello".to_string())];
let response = provider.chat(&messages).await?;
```

### 配置 API

#### `ConfigManager::load_config()`

加载配置文件。

**返回**: `Result<AppConfig>`

#### `ConfigManager::save_config(config: &mut AppConfig)`

保存配置文件（自动加密敏感信息）。

**参数**:
- `config`: 要保存的配置

**返回**: `Result<()>`

---

## 🔒 安全性

### 密码加密

- 使用 AES-256-GCM
- 密钥基于机器标识（用户名 + 主机名）
- 每次加密使用随机 nonce
- Base64 编码存储

### 配置文件

- 敏感信息自动加密
- 配置文件权限应设置为 600
- 支持安全导出（不含敏感信息）

---

## 🐛 已知问题

1. ✅ **已修复**: `AiProviderType` 缺少 Hash derive
2. ✅ **已修复**: whoami::hostname() 使用了已弃用的 API
3. ✅ **已修复**: ssh2::Channel 没有 set_blocking 方法
4. ℹ️ **待处理**: 未使用代码警告（集成到 app.rs 后会消失）

---

## 📝 下一步开发计划

### Phase 1: 集成 (2-3 天)

1. 重构 `app.rs` 结构
2. 添加异步运行时支持
3. 集成 SSH 模块
4. 集成 AI 模块
5. 集成配置管理

### Phase 2: 测试 (2 天)

1. 单元测试
2. 集成测试
3. 端到端测试
4. 性能测试

### Phase 3: 文档与发布 (1 天)

1. 更新 README.md
2. 编写使用指南
3. 创建示例配置
4. 打 v0.2.0 tag

---

## 🎉 总结

v0.2.0 的核心功能开发已经完成！

- ✅ 6 个核心模块全部实现
- ✅ 编译通过无错误
- ✅ 完整的类型系统
- ✅ SSH + AI + 配置管理全部就绪
- 🚧 下一步：集成到 UI 层

**工作量统计**:
- 代码行数: ~1500+ 行
- 新增文件: 5 个
- 新增依赖: 15+ 个
- 进度: 约 70% 完成

---

**最后更新**: 2026-02-02  
**作者**: iShell Development Team
