use ishell::*;

#[cfg(test)]
mod crypto_tests {
    use ishell::crypto::PasswordEncryptor;

    #[test]
    fn test_encryptor_creation() {
        let result = PasswordEncryptor::new();
        assert!(result.is_ok(), "Failed to create password encryptor");
    }

    #[test]
    fn test_encrypt_decrypt_basic() {
        let encryptor = PasswordEncryptor::new().unwrap();

        let original = "test_password_123";
        let encrypted = encryptor.encrypt(original).unwrap();
        let decrypted = encryptor.decrypt(&encrypted).unwrap();

        assert_eq!(
            original, decrypted,
            "Decrypted password doesn't match original"
        );
        assert_ne!(
            original, encrypted,
            "Encrypted should be different from original"
        );
    }

    #[test]
    fn test_encrypt_empty_string() {
        let encryptor = PasswordEncryptor::new().unwrap();

        let encrypted = encryptor.encrypt("").unwrap();
        assert_eq!(encrypted, "", "Empty string should encrypt to empty string");

        let decrypted = encryptor.decrypt("").unwrap();
        assert_eq!(decrypted, "", "Empty string should decrypt to empty string");
    }

    #[test]
    fn test_encrypt_unicode() {
        let encryptor = PasswordEncryptor::new().unwrap();

        let original = "密码测试🔐";
        let encrypted = encryptor.encrypt(original).unwrap();
        let decrypted = encryptor.decrypt(&encrypted).unwrap();

        assert_eq!(original, decrypted, "Unicode password not preserved");
    }

    #[test]
    fn test_encrypt_different_nonces() {
        let encryptor = PasswordEncryptor::new().unwrap();

        let password = "same_password";
        let encrypted1 = encryptor.encrypt(password).unwrap();
        let encrypted2 = encryptor.encrypt(password).unwrap();

        // Different nonces mean different ciphertexts
        assert_ne!(
            encrypted1, encrypted2,
            "Encryptions should be different due to random nonce"
        );

        // But both should decrypt to the same value
        assert_eq!(encryptor.decrypt(&encrypted1).unwrap(), password);
        assert_eq!(encryptor.decrypt(&encrypted2).unwrap(), password);
    }

    #[test]
    fn test_decrypt_invalid_base64() {
        let encryptor = PasswordEncryptor::new().unwrap();

        let result = encryptor.decrypt("not-valid-base64!!!");
        assert!(result.is_err(), "Should fail on invalid base64");
    }

    #[test]
    fn test_decrypt_too_short() {
        let encryptor = PasswordEncryptor::new().unwrap();

        // Valid base64 but too short to contain nonce
        let result = encryptor.decrypt("YWJj"); // "abc" in base64
        assert!(result.is_err(), "Should fail on too short data");
    }

    #[test]
    fn test_long_password() {
        let encryptor = PasswordEncryptor::new().unwrap();

        let long_password = "a".repeat(1000);
        let encrypted = encryptor.encrypt(&long_password).unwrap();
        let decrypted = encryptor.decrypt(&encrypted).unwrap();

        assert_eq!(long_password, decrypted, "Long password not preserved");
    }
}

#[cfg(test)]
mod types_tests {
    use super::*;

    #[test]
    fn test_ssh_config_creation() {
        let config = SshConfig::new(
            "test".to_string(),
            "localhost".to_string(),
            22,
            "user".to_string(),
        );

        assert_eq!(config.name, "test");
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 22);
        assert_eq!(config.username, "user");
        assert!(config.auth.is_none());
    }

    #[test]
    fn test_auth_method_password() {
        let auth = AuthMethod::Password("secret".to_string());

        match auth {
            AuthMethod::Password(ref pass) => {
                assert_eq!(pass, "secret");
            }
            _ => panic!("Wrong auth method type"),
        }
    }

    #[test]
    fn test_auth_method_private_key() {
        use std::path::PathBuf;

        let auth = AuthMethod::PrivateKey {
            key_path: PathBuf::from("/path/to/key"),
            passphrase: Some("passphrase".to_string()),
        };

        match auth {
            AuthMethod::PrivateKey {
                key_path,
                passphrase,
            } => {
                assert_eq!(key_path, PathBuf::from("/path/to/key"));
                assert_eq!(passphrase, Some("passphrase".to_string()));
            }
            _ => panic!("Wrong auth method type"),
        }
    }

    #[test]
    fn test_ai_message_creation() {
        let user_msg = AiMessage::user("Hello".to_string());
        assert_eq!(user_msg.role, "user");
        assert_eq!(user_msg.content, "Hello");

        let assistant_msg = AiMessage::assistant("Hi there".to_string());
        assert_eq!(assistant_msg.role, "assistant");
        assert_eq!(assistant_msg.content, "Hi there");
    }

    #[test]
    fn test_ai_provider_type_display() {
        assert_eq!(AiProviderType::Ollama.to_string(), "Ollama");
        assert_eq!(AiProviderType::OpenAI.to_string(), "OpenAI");
        assert_eq!(AiProviderType::Google.to_string(), "Google");
    }

    #[test]
    fn test_connection_status() {
        assert_eq!(
            ConnectionStatus::Disconnected,
            ConnectionStatus::Disconnected
        );
        assert_ne!(ConnectionStatus::Connected, ConnectionStatus::Disconnected);
    }

    #[test]
    fn test_app_config_default() {
        let config = AppConfig::default();

        assert_eq!(config.version, "0.2.0");
        assert_eq!(config.connections.len(), 0);
        assert_eq!(config.settings.default_ai_provider, AiProviderType::Ollama);
    }

    #[test]
    fn test_ollama_config_default() {
        let config = OllamaConfig::default();

        assert!(config.enabled);
        assert_eq!(config.base_url, "http://localhost:11434");
        assert_eq!(config.model, "llama3.2");
    }

    #[test]
    fn test_openai_config_default() {
        let config = OpenAiConfig::default();

        assert!(!config.enabled);
        assert!(config.api_key.is_none());
        assert_eq!(config.model, "gpt-4o-mini");
    }

    #[test]
    fn test_google_config_default() {
        let config = GoogleConfig::default();

        assert!(!config.enabled);
        assert!(config.api_key.is_none());
        assert_eq!(config.model, "gemini-1.5-flash");
    }
}

#[cfg(test)]
mod config_tests {
    use super::*;
    use ishell::config::ConfigManager;
    use rand::Rng;

    fn get_test_manager() -> ConfigManager {
        let temp_dir = std::env::temp_dir().join("ishell_integration_test");
        let rand_val: u32 = rand::thread_rng().gen();
        let config_path = temp_dir.join(format!("config_{}.toml", rand_val));

        // Ensure parent exists
        std::fs::create_dir_all(&temp_dir).unwrap();

        ConfigManager::new_with_path(config_path).unwrap()
    }

    #[test]
    fn test_config_dir() {
        let _manager = get_test_manager();
    }

    #[test]
    fn test_config_manager_creation() {
        let manager = get_test_manager();
        assert!(true);
    }

    #[test]
    fn test_load_default_config() {
        let manager = get_test_manager();
        let config = manager.load_config();
        assert!(config.is_ok());
        if let Ok(cfg) = config {
            assert_eq!(cfg.version, "0.2.0");
        }
    }

    #[test]
    fn test_encryption_roundtrip_in_config() {
        let manager = get_test_manager();
        let mut conn = SshConfig::new(
            "test".to_string(),
            "localhost".to_string(),
            22,
            "user".to_string(),
        );
        conn.auth = Some(AuthMethod::Password("test_password".to_string()));

        // Encrypt
        let encrypt_result = manager.encrypt_connection(&mut conn);
        assert!(encrypt_result.is_ok());
        assert!(conn.password_encrypted.is_some());

        // Decrypt
        let decrypt_result = manager.decrypt_connection(&mut conn);
        assert!(decrypt_result.is_ok());

        if let Some(AuthMethod::Password(pass)) = &conn.auth {
            assert_eq!(pass, "test_password");
        } else {
            panic!("Auth method not properly restored");
        }
    }

    #[test]
    fn test_save_and_load_config() {
        let manager = get_test_manager();

        let mut config = AppConfig::default();
        config.connections.push(SshConfig::new(
            "Test Server".to_string(),
            "test.example.com".to_string(),
            22,
            "testuser".to_string(),
        ));

        // Save
        let save_result = manager.save_config(&mut config);
        assert!(
            save_result.is_ok(),
            "Failed to save config: {:?}",
            save_result.err()
        );

        // Load
        let loaded_config = manager.load_config();
        assert!(
            loaded_config.is_ok(),
            "Failed to load config: {:?}",
            loaded_config.err()
        );

        if let Ok(loaded) = loaded_config {
            assert_eq!(loaded.connections.len(), config.connections.len());
            if !loaded.connections.is_empty() {
                assert_eq!(loaded.connections[0].name, "Test Server");
            }
        }
    }

    #[test]
    fn test_export_config_safe() {
        let manager = get_test_manager();

        let mut config = AppConfig::default();
        let mut conn = SshConfig::new(
            "Secure Server".to_string(),
            "secure.example.com".to_string(),
            22,
            "admin".to_string(),
        );
        conn.password_encrypted = Some("encrypted_password".to_string());
        config.connections.push(conn);

        let exported = manager.export_config_safe(&config);
        assert!(exported.is_ok());

        if let Ok(export_str) = exported {
            assert!(!export_str.contains("encrypted_password"));
        }
    }
}

#[cfg(test)]
mod ssh_tests {
    use super::*;
    use ishell::ssh::SshSession;

    #[test]
    fn test_ssh_session_creation() {
        let session = SshSession::new("localhost".to_string(), 22, "testuser".to_string());

        assert_eq!(session.status(), ConnectionStatus::Disconnected);
        assert!(!session.is_connected());
    }

    #[test]
    fn test_ssh_session_status() {
        let session = SshSession::new("example.com".to_string(), 22, "user".to_string());

        assert_eq!(session.status(), ConnectionStatus::Disconnected);
    }

    // Manual Test: Real SSH connection test
    // This test requires a running SSH server. To run it:
    // 1. Ensure you have an SSH server running on localhost:22
    // 2. Create a test user: testuser/testpass
    // 3. Run: cargo test test_ssh_connect_password -- --ignored --nocapture
    #[test]
    #[ignore]
    fn test_ssh_connect_password() {
        let session = SshSession::new("localhost".to_string(), 22, "testuser".to_string());

        let auth = AuthMethod::Password("testpass".to_string());
        let _result = session.connect(&auth);
    }
}

#[cfg(test)]
mod ai_tests {
    use super::*;
    use ishell::ai::{AiManager, AiProvider, OllamaProvider};

    #[test]
    fn test_ai_manager_creation() {
        let manager = AiManager::new();

        // Initially no providers
        assert!(!manager.is_provider_available(AiProviderType::Ollama));
        assert!(!manager.is_provider_available(AiProviderType::OpenAI));
        assert!(!manager.is_provider_available(AiProviderType::Google));
    }

    #[test]
    fn test_ai_manager_register_provider() {
        let mut manager = AiManager::new();

        let provider = Box::new(OllamaProvider::new(
            "http://localhost:11434".to_string(),
            "llama3.2".to_string(),
        ));

        manager.register_provider(provider);

        assert!(manager.is_provider_available(AiProviderType::Ollama));
        assert!(!manager.is_provider_available(AiProviderType::OpenAI));
    }

    #[test]
    fn test_ai_manager_set_current_provider() {
        let mut manager = AiManager::new();

        let ollama = Box::new(OllamaProvider::new(
            "http://localhost:11434".to_string(),
            "llama3.2".to_string(),
        ));

        manager.register_provider(ollama);
        manager.set_current_provider(AiProviderType::Ollama);

        assert_eq!(manager.current_provider(), AiProviderType::Ollama);
    }

    #[test]
    fn test_ollama_provider_creation() {
        let provider =
            OllamaProvider::new("http://localhost:11434".to_string(), "llama3.2".to_string());

        assert_eq!(provider.provider_type(), AiProviderType::Ollama);
    }

    // Note: Real AI API tests require running services
    // Async test not supported in standard test framework, would need tokio::test
    // #[test]
    // #[ignore]
    // async fn test_ollama_chat() {
    //     let provider = OllamaProvider::new(
    //         "http://localhost:11434".to_string(),
    //         "llama3.2".to_string(),
    //     );
    //
    //     let messages = vec![AiMessage::user("Hello".to_string())];
    //     let result = provider.chat(&messages).await;
    // }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use ishell::config::ConfigManager;
    use ishell::crypto::PasswordEncryptor;
    use rand::Rng;

    fn get_test_manager() -> ConfigManager {
        let temp_dir = std::env::temp_dir().join("ishell_integration_test");
        let rand_val: u32 = rand::thread_rng().gen();
        let config_path = temp_dir.join(format!("config_{}.toml", rand_val));

        // Ensure parent exists
        std::fs::create_dir_all(&temp_dir).unwrap();

        ConfigManager::new_with_path(config_path).unwrap()
    }

    #[test]
    fn test_full_config_workflow() {
        // Create config manager
        let manager = get_test_manager();
        let _encryptor = PasswordEncryptor::new().unwrap();

        // Create a connection with password
        let mut conn = SshConfig::new(
            "Integration Test Server".to_string(),
            "test.local".to_string(),
            2222,
            "testuser".to_string(),
        );

        let original_password = "super_secret123";
        conn.auth = Some(AuthMethod::Password(original_password.to_string()));

        // Encrypt the connection
        manager.encrypt_connection(&mut conn).unwrap();
        assert!(conn.password_encrypted.is_some());

        // Password should be encrypted
        let encrypted = conn.password_encrypted.clone().unwrap();
        assert_ne!(encrypted, original_password);

        // Decrypt the connection
        manager.decrypt_connection(&mut conn).unwrap();

        // Verify password is correctly decrypted
        if let Some(AuthMethod::Password(pass)) = &conn.auth {
            assert_eq!(pass, original_password);
        } else {
            panic!("Password not properly decrypted");
        }
    }

    #[test]
    fn test_config_with_multiple_connections() {
        let manager = get_test_manager();

        let mut config = AppConfig::default();

        // Add password-based connection
        let mut conn1 = SshConfig::new(
            "Server 1".to_string(),
            "server1.com".to_string(),
            22,
            "user1".to_string(),
        );
        conn1.auth = Some(AuthMethod::Password("pass1".to_string()));

        // Add key-based connection
        let mut conn2 = SshConfig::new(
            "Server 2".to_string(),
            "server2.com".to_string(),
            22,
            "user2".to_string(),
        );
        conn2.auth = Some(AuthMethod::PrivateKey {
            key_path: std::path::PathBuf::from("/home/user/.ssh/id_rsa"),
            passphrase: Some("keypass".to_string()),
        });

        config.connections.push(conn1);
        config.connections.push(conn2);

        // Save
        let save_result = manager.save_config(&mut config);
        assert!(
            save_result.is_ok(),
            "Failed to save config: {:?}",
            save_result.err()
        );

        // Load
        let loaded_result = manager.load_config();
        assert!(
            loaded_result.is_ok(),
            "Failed to load config: {:?}",
            loaded_result.err()
        );
        let loaded = loaded_result.unwrap();

        assert_eq!(loaded.connections.len(), 2);
        assert_eq!(loaded.connections[0].name, "Server 1");
        assert_eq!(loaded.connections[1].name, "Server 2");
    }

    #[test]
    fn test_ai_config_persistence() {
        let manager = get_test_manager();

        let mut config = AppConfig::default();

        // Configure Ollama
        config.ai.ollama.enabled = true;
        config.ai.ollama.model = "custom-model".to_string();

        // Configure OpenAI with API key
        config.ai.openai.enabled = true;
        config.ai.openai.api_key = Some("sk-test-key".to_string());

        // Save
        let save_result = manager.save_config(&mut config);
        assert!(
            save_result.is_ok(),
            "Failed to save config: {:?}",
            save_result.err()
        );

        // Load
        let loaded_result = manager.load_config();
        assert!(
            loaded_result.is_ok(),
            "Failed to load config: {:?}",
            loaded_result.err()
        );
        let loaded = loaded_result.unwrap();

        assert!(loaded.ai.ollama.enabled);
        assert_eq!(loaded.ai.ollama.model, "custom-model");
        assert!(loaded.ai.openai.enabled);
        // API key should be decrypted
        assert!(loaded.ai.openai.api_key.is_some());
    }
}

#[cfg(test)]
mod history_tests {
    use ishell::history::CommandHistory;

    #[test]
    fn test_command_history_creation() {
        let history = CommandHistory::new();
        assert_eq!(history.commands.len(), 0);
    }

    #[test]
    fn test_add_command() {
        let mut history = CommandHistory::new();
        history.add("ls -la".to_string(), "server1".to_string());
        
        assert_eq!(history.commands.len(), 1);
        assert_eq!(history.commands[0].command, "ls -la");
        assert_eq!(history.commands[0].connection, "server1");
    }

    #[test]
    fn test_search_commands() {
        let mut history = CommandHistory::new();
        history.add("ls -la".to_string(), "server1".to_string());
        history.add("cd /home".to_string(), "server1".to_string());
        history.add("cat file.txt".to_string(), "server2".to_string());
        
        let results = history.search("ls");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].command, "ls -la");
        
        let all_results = history.search("");
        assert_eq!(all_results.len(), 3);
    }

    #[test]
    fn test_history_stats() {
        let mut history = CommandHistory::new();
        history.add("ls".to_string(), "server1".to_string());
        history.add("ls".to_string(), "server1".to_string());
        history.add("cd /home".to_string(), "server2".to_string());
        
        let stats = history.stats();
        assert_eq!(stats.total_commands, 3);
        assert_eq!(stats.unique_commands, 2);
        assert_eq!(stats.unique_connections, 2);
    }

    #[test]
    fn test_save_and_load_history() {
        let temp_dir = std::env::temp_dir();
        let rand_val: u32 = rand::Rng::gen(&mut rand::thread_rng());
        let history_path = temp_dir.join(format!("test_history_{}.json", rand_val));

        // Create and save history
        let mut history = CommandHistory::new();
        history.add("echo hello".to_string(), "server1".to_string());
        history.add("ls -la".to_string(), "server2".to_string());
        
        let save_result = history.save(&history_path);
        assert!(save_result.is_ok(), "Failed to save history");

        // Load history
        let loaded = CommandHistory::load(&history_path);
        assert!(loaded.is_ok(), "Failed to load history");
        
        let loaded_history = loaded.unwrap();
        assert_eq!(loaded_history.commands.len(), 2);
        assert_eq!(loaded_history.commands[0].command, "echo hello");
        assert_eq!(loaded_history.commands[1].command, "ls -la");

        // Cleanup
        let _ = std::fs::remove_file(history_path);
    }

    #[test]
    fn test_clear_history() {
        let mut history = CommandHistory::new();
        history.add("ls".to_string(), "server1".to_string());
        history.add("pwd".to_string(), "server1".to_string());
        
        assert_eq!(history.commands.len(), 2);
        
        history.clear();
        assert_eq!(history.commands.len(), 0);
    }

    #[test]
    fn test_history_max_size() {
        let mut history = CommandHistory::new().with_max_size(3);
        
        history.add("cmd1".to_string(), "server".to_string());
        history.add("cmd2".to_string(), "server".to_string());
        history.add("cmd3".to_string(), "server".to_string());
        assert_eq!(history.commands.len(), 3);
        
        // Adding 4th command should remove the first
        history.add("cmd4".to_string(), "server".to_string());
        assert_eq!(history.commands.len(), 3);
        assert_eq!(history.commands[0].command, "cmd2");
        assert_eq!(history.commands[2].command, "cmd4");
    }

    #[test]
    fn test_ignore_empty_commands() {
        let mut history = CommandHistory::new();
        history.add("".to_string(), "server".to_string());
        history.add("   ".to_string(), "server".to_string());
        
        assert_eq!(history.commands.len(), 0);
    }
}
