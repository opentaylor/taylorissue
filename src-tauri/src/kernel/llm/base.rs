use async_trait::async_trait;
use serde_json::Value;

#[async_trait]
pub trait BaseLlm: Send + Sync {
    fn id(&self) -> &str;
    fn api_key(&self) -> &str;
    fn base_url(&self) -> &str;
    fn model(&self) -> &str;

    async fn run(
        &self,
        messages: Vec<Value>,
        tools: Option<Vec<Value>>,
        response_format: Option<Value>,
    ) -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>>;
}

pub struct LlmConfig {
    pub id: String,
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub extra_params: serde_json::Map<String, Value>,
}

impl LlmConfig {
    pub fn new(api_key: &str, base_url: &str, model: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            api_key: api_key.to_string(),
            base_url: base_url.to_string(),
            model: model.to_string(),
            extra_params: serde_json::Map::new(),
        }
    }
}

pub fn make_llm(base_url: &str, api_key: &str, model: &str) -> super::OpenAiLlm {
    super::OpenAiLlm::new(api_key, base_url, model)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_config_defaults() {
        let config = LlmConfig::new("key", "url", "model");
        assert_eq!(config.api_key, "key");
        assert_eq!(config.base_url, "url");
        assert_eq!(config.model, "model");
        assert!(!config.id.is_empty());
    }
}
