use serde::{Deserialize, Serialize};

/// LLM Provider 枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LlmProvider {
    OpenAI,
    Claude,
    DeepSeek,
    Ollama,
    Custom,
}

impl LlmProvider {
    pub fn default_endpoint(&self) -> &str {
        match self {
            LlmProvider::OpenAI => "https://api.openai.com/v1",
            LlmProvider::Claude => "https://api.anthropic.com/v1",
            LlmProvider::DeepSeek => "https://api.deepseek.com/v1",
            LlmProvider::Ollama => "http://localhost:11434/v1",
            LlmProvider::Custom => "",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "openai" => LlmProvider::OpenAI,
            "claude" => LlmProvider::Claude,
            "deepseek" => LlmProvider::DeepSeek,
            "ollama" => LlmProvider::Ollama,
            _ => LlmProvider::Custom,
        }
    }
}

/// LLM 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub provider: String,
    pub endpoint: String,
    pub api_key: String,
    pub model: String,
}

/// Chat 消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Chat 请求体（OpenAI 兼容格式）
#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f64,
}

/// Chat 响应体
#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatResponseMessage,
}

#[derive(Debug, Deserialize)]
struct ChatResponseMessage {
    content: String,
}

/// 调用 LLM API（统一 OpenAI 兼容格式）
pub async fn chat_completion(
    config: &LlmConfig,
    messages: Vec<ChatMessage>,
    temperature: f64,
) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| format!("HTTP 客户端创建失败: {}", e))?;

    let endpoint = format!("{}/chat/completions", config.endpoint.trim_end_matches('/'));

    let request_body = ChatRequest {
        model: config.model.clone(),
        messages,
        temperature,
    };

    let mut req = client
        .post(&endpoint)
        .header("Content-Type", "application/json")
        .json(&request_body);

    // 添加认证头
    if !config.api_key.is_empty() {
        let provider = LlmProvider::from_str(&config.provider);
        match provider {
            LlmProvider::Claude => {
                req = req.header("x-api-key", &config.api_key);
                req = req.header("anthropic-version", "2023-06-01");
            }
            _ => {
                req = req.header("Authorization", format!("Bearer {}", config.api_key));
            }
        }
    }

    let response = req
        .send()
        .await
        .map_err(|e| format!("请求发送失败: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("LLM API 错误 ({}): {}", status, body));
    }

    let chat_response: ChatResponse = response
        .json()
        .await
        .map_err(|e| format!("响应解析失败: {}", e))?;

    chat_response
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .ok_or_else(|| "LLM 返回空响应".to_string())
}

/// 测试连接（发送简单请求验证配置）
pub async fn test_connection(config: &LlmConfig) -> Result<String, String> {
    let messages = vec![ChatMessage {
        role: "user".to_string(),
        content: "Hello, respond with just 'OK' to confirm connection.".to_string(),
    }];
    chat_completion(config, messages, 0.0).await
}
