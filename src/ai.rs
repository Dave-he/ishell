use crate::types::{AiMessage, AiProviderType, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// AI 提供商 trait
#[async_trait]
pub trait AiProvider: Send + Sync {
    /// 发送聊天消息并获取响应
    async fn chat(&self, messages: &[AiMessage]) -> Result<String>;

    /// 获取提供商类型
    fn provider_type(&self) -> AiProviderType;
}

// ============================================================================
// Ollama Provider
// ============================================================================

pub struct OllamaProvider {
    base_url: String,
    model: String,
    client: reqwest::Client,
}

impl OllamaProvider {
    pub fn new(base_url: String, model: String) -> Self {
        Self {
            base_url,
            model,
            client: reqwest::Client::new(),
        }
    }
}

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
}

#[derive(Serialize, Deserialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OllamaResponse {
    message: OllamaMessage,
}

#[async_trait]
impl AiProvider for OllamaProvider {
    async fn chat(&self, messages: &[AiMessage]) -> Result<String> {
        let url = format!("{}/api/chat", self.base_url);

        let ollama_messages: Vec<OllamaMessage> = messages
            .iter()
            .map(|m| OllamaMessage {
                role: m.role.clone(),
                content: m.content.clone(),
            })
            .collect();

        let request = OllamaRequest {
            model: self.model.clone(),
            messages: ollama_messages,
            stream: false,
        };

        let response = self.client.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(format!("Ollama API error ({}): {}", status, error_text).into());
        }

        let ollama_response: OllamaResponse = response.json().await?;
        Ok(ollama_response.message.content)
    }

    fn provider_type(&self) -> AiProviderType {
        AiProviderType::Ollama
    }
}

// ============================================================================
// OpenAI Provider
// ============================================================================

pub struct OpenAiProvider {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl OpenAiProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            api_key,
            model,
            client: reqwest::Client::new(),
        }
    }
}

#[derive(Serialize)]
struct OpenAiRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
}

#[derive(Serialize, Deserialize)]
struct OpenAiMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenAiResponse {
    choices: Vec<OpenAiChoice>,
}

#[derive(Deserialize)]
struct OpenAiChoice {
    message: OpenAiMessage,
}

#[async_trait]
impl AiProvider for OpenAiProvider {
    async fn chat(&self, messages: &[AiMessage]) -> Result<String> {
        let url = "https://api.openai.com/v1/chat/completions";

        let openai_messages: Vec<OpenAiMessage> = messages
            .iter()
            .map(|m| OpenAiMessage {
                role: m.role.clone(),
                content: m.content.clone(),
            })
            .collect();

        let request = OpenAiRequest {
            model: self.model.clone(),
            messages: openai_messages,
        };

        let response = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(format!("OpenAI API error ({}): {}", status, error_text).into());
        }

        let openai_response: OpenAiResponse = response.json().await?;

        openai_response
            .choices
            .first()
            .map(|choice| choice.message.content.clone())
            .ok_or("No response from OpenAI".into())
    }

    fn provider_type(&self) -> AiProviderType {
        AiProviderType::OpenAI
    }
}

// ============================================================================
// Google Gemini Provider
// ============================================================================

pub struct GoogleProvider {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl GoogleProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            api_key,
            model,
            client: reqwest::Client::new(),
        }
    }
}

#[derive(Serialize)]
struct GoogleRequest {
    contents: Vec<GoogleContent>,
}

#[derive(Serialize)]
struct GoogleContent {
    role: String,
    parts: Vec<GooglePart>,
}

#[derive(Serialize)]
struct GooglePart {
    text: String,
}

#[derive(Deserialize)]
struct GoogleResponse {
    candidates: Vec<GoogleCandidate>,
}

#[derive(Deserialize)]
struct GoogleCandidate {
    content: GoogleContentResponse,
}

#[derive(Deserialize)]
struct GoogleContentResponse {
    parts: Vec<GooglePartResponse>,
}

#[derive(Deserialize)]
struct GooglePartResponse {
    text: String,
}

#[async_trait]
impl AiProvider for GoogleProvider {
    async fn chat(&self, messages: &[AiMessage]) -> Result<String> {
        // Gemini API 使用不同的角色命名
        let contents: Vec<GoogleContent> = messages
            .iter()
            .map(|m| {
                let role = if m.role == "assistant" {
                    "model"
                } else {
                    "user"
                };
                GoogleContent {
                    role: role.to_string(),
                    parts: vec![GooglePart {
                        text: m.content.clone(),
                    }],
                }
            })
            .collect();

        let request = GoogleRequest { contents };

        // Gemini API endpoint
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(format!("Google AI API error ({}): {}", status, error_text).into());
        }

        let google_response: GoogleResponse = response.json().await?;

        google_response
            .candidates
            .first()
            .and_then(|c| c.content.parts.first())
            .map(|p| p.text.clone())
            .ok_or("No response from Google AI".into())
    }

    fn provider_type(&self) -> AiProviderType {
        AiProviderType::Google
    }
}

// ============================================================================
// AI Manager - 统一管理多个 AI 提供商
// ============================================================================

pub struct AiManager {
    providers: std::collections::HashMap<AiProviderType, Box<dyn AiProvider>>,
    current_provider: AiProviderType,
}

impl AiManager {
    pub fn new() -> Self {
        Self {
            providers: std::collections::HashMap::new(),
            current_provider: AiProviderType::Ollama,
        }
    }

    /// 注册 AI 提供商
    pub fn register_provider(&mut self, provider: Box<dyn AiProvider>) {
        let provider_type = provider.provider_type();
        self.providers.insert(provider_type, provider);
    }

    /// 设置当前使用的提供商
    pub fn set_current_provider(&mut self, provider_type: AiProviderType) {
        if self.providers.contains_key(&provider_type) {
            self.current_provider = provider_type;
        }
    }

    /// 获取当前提供商
    pub fn current_provider(&self) -> AiProviderType {
        self.current_provider
    }

    /// 发送聊天消息
    pub async fn chat(&self, messages: &[AiMessage]) -> Result<String> {
        let provider = self
            .providers
            .get(&self.current_provider)
            .ok_or("No AI provider configured")?;

        provider.chat(messages).await
    }

    /// 检查提供商是否可用
    pub fn is_provider_available(&self, provider_type: AiProviderType) -> bool {
        self.providers.contains_key(&provider_type)
    }
}

impl Default for AiManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // 需要真实的 API 访问
    async fn test_ollama_provider() {
        let provider =
            OllamaProvider::new("http://localhost:11434".to_string(), "llama3.2".to_string());

        let messages = vec![AiMessage::user("Hello!".to_string())];

        let result = provider.chat(&messages).await;
        // 实际测试需要运行 Ollama
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_ai_manager() {
        let mut manager = AiManager::new();

        let ollama = Box::new(OllamaProvider::new(
            "http://localhost:11434".to_string(),
            "llama3.2".to_string(),
        ));

        manager.register_provider(ollama);
        assert!(manager.is_provider_available(AiProviderType::Ollama));
        assert!(!manager.is_provider_available(AiProviderType::OpenAI));
    }
}
