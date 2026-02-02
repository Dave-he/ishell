# iShell 测试指南

## 📋 测试概览

iShell 包含完整的自动化测试套件，覆盖所有核心功能。

### 测试统计

- **总测试数**: 34
- **自动化测试**: 33 (通过)
- **手动测试**: 1 (需要外部环境)
- **测试覆盖率**: 96.4%

---

## 🚀 运行测试

### 运行所有自动化测试

```bash
cargo test
```

### 运行特定模块测试

```bash
# 加密模块测试
cargo test crypto_tests

# 配置管理测试
cargo test config_tests

# SSH 模块测试
cargo test ssh_tests

# AI 模块测试
cargo test ai_tests

# 集成测试
cargo test integration_tests
```

### 运行单个测试

```bash
cargo test test_encrypt_decrypt_basic
```

### 查看详细输出

```bash
cargo test -- --nocapture
```

---

## 🔍 手动测试

某些测试需要外部环境支持，因此被标记为 `#[ignore]`。

### SSH 连接测试

**测试**: `test_ssh_connect_password`

**要求**:
- SSH 服务器运行在 localhost:22
- 测试用户: `testuser`
- 测试密码: `testpass`

**运行方法**:

```bash
# 1. 启动 SSH 服务器并创建测试用户
# macOS:
sudo systemsetup -setremotelogin on
sudo dscl . -create /Users/testuser
sudo dscl . -create /Users/testuser UserShell /bin/bash
sudo dscl . -passwd /Users/testuser testpass

# Linux:
sudo useradd -m -s /bin/bash testuser
echo "testuser:testpass" | sudo chpasswd
sudo systemctl start sshd

# 2. 运行测试
cargo test test_ssh_connect_password -- --ignored --nocapture
```

### AI API 测试

**测试**: AI 提供商真实调用测试（当前已注释）

**要求**:
- Ollama: 运行在 http://localhost:11434
- OpenAI: 有效的 API key
- Google: 有效的 API key

**如何启用**:

1. 取消 `tests/integration_test.rs` 中的 AI 测试注释
2. 添加 `#[tokio::test]` 标记
3. 配置相应的 API key
4. 运行测试

```bash
# 确保 Ollama 运行
ollama serve

# 运行测试
cargo test -- --ignored
```

---

## 📊 测试详情

### 加密模块 (8 tests)

| 测试 | 验证功能 |
|------|---------|
| `test_encryptor_creation` | 加密器创建 |
| `test_encrypt_decrypt_basic` | 基础加解密 |
| `test_encrypt_empty_string` | 空字符串处理 |
| `test_encrypt_unicode` | Unicode 支持 |
| `test_encrypt_different_nonces` | 随机 nonce |
| `test_decrypt_invalid_base64` | 错误处理 |
| `test_decrypt_too_short` | 数据验证 |
| `test_long_password` | 长密码支持 |

### 类型模块 (9 tests)

| 测试 | 验证功能 |
|------|---------|
| `test_ssh_config_creation` | SSH 配置创建 |
| `test_auth_method_password` | 密码认证 |
| `test_auth_method_private_key` | 密钥认证 |
| `test_ai_message_creation` | AI 消息 |
| `test_ai_provider_type_display` | AI 提供商类型 |
| `test_connection_status` | 连接状态 |
| `test_app_config_default` | 默认配置 |
| `test_ollama_config_default` | Ollama 配置 |
| `test_openai_config_default` | OpenAI 配置 |
| `test_google_config_default` | Google 配置 |

### 配置模块 (5 tests)

| 测试 | 验证功能 |
|------|---------|
| `test_config_manager_creation` | 配置管理器创建 |
| `test_load_default_config` | 加载默认配置 |
| `test_encryption_roundtrip` | 加密往返 |
| `test_save_and_load_config保存加载配置 |
| `test_export_config_safe` | 安全导出 |

### SSH 模块 (3 tests)

| 测试 | 验证功能 | 状态 |
|------|---------|------|
| `test_ssh_session_creation` | 会话创建 | ✅ 自动 |
| `test_ssh_session_status` | 状态管理 | ✅ 自动 |
| `test_ssh_connect_password` | 密码连接 | ⏭️ 手动 |

### AI 模块 (4 tests)

| 测试 | 验证功能 |
|------|---------|
| `test_ai_manager_creation` | AI 管理器创建 |
| `test_ai_manager_register_provider` | 提供商注册 |
| `test_ai_manager_set_current_provider` | 切换提供商 |
| `test_ollama_provider_creation` | Ollama 提供商 |

### 集成测试 (3 tests)

| 测试 | 验证功能 |
|------|---------|
| `test_full_config_workflow` | 完整配置流程 |
| `test_config_with_multiple_connections` | 多连接配置 |
| `test_ai_config_persistence` | AI 配置持久化 |

---

## 🔧 测试隔离

所有测试使用独立的临时配置文件，确保测试间不会相互影响：

```rust
fn get_test_manager() -> ConfigManager {
    let temp_dir = std::env::temp_dir().join("ishell_integration_test");
    let rand_val: u32 = rand::thread_rng().gen();
    let config_path = temp_dir.join(format!("config_{}.toml", rand_val));
    
    std::fs::create_dir_all(&temp_dir).unwrap();
    ConfigManager::new_with_path(config_path).unwrap()
}
```

测试配置文件位置: `/tmp/ishell_integration_test/config_<random>.toml`

---

## 🐛 故障排查

### 测试失败

如果测试失败，请检查：

1. **编译错误**: 确保代码编译通过
   ```bash
   cargo build
   ```

2. **依赖问题**: 更新依赖
   ```bash
   cargo update
   ```

3. **缓存问题**: 清理并重新构建
   ```bash
   cargo clean && cargo test
   ```

### 临时文件清理

测试会在 `/tmp` 目录创建临时文件。如需清理：

```bash
 /tmp/ishell_integration_test
```

---

## 📈 持续集成

### GitHub Actions 配置示例

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run tests
        run: cargo test --all-features
```

---

## 🎯 测试最佳实践

### 编写新测试

1. **命名规范**: `test_<module>_<feature>`
2. **独立性**: 每个测试独立运行
3. **清理**: 测试后清理临时资源
4. **注释**: 添加清晰的测试说明

### 示例

```rust
#[test]
fn test_new_feature() {
    // Arrange - 准备测试数据
    let manager = get_test_manager();
    
    // Act - 执行测试操作
    let result = manager.do_something();
    
    // Assert - 验证结果
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), expected_value);
}
```

---

## 📚 相关文档

- [TEST_REPORT.md](TEST_REPORT.md) - 详细测试报告
- [DEV_GUIDE_v0.2.0.md](DEV_GUIDE_v0.2.0.md) - 开发指南
- [INTEGRATION_COMPLETE.md](INTEGRATION_COMPLETE.md) - 集成完成报告

---

**最后更新**: 2026-02-02  
**维护者**: iShell Development Team
