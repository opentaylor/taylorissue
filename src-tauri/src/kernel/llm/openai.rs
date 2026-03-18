use async_trait::async_trait;
use serde_json::Value;

use super::base::{BaseLlm, LlmConfig};
use super::response_format::{parse_json_content, to_response_format};

pub fn tool_schema(tool: &Value) -> Value {
    if let Some(params) = tool.get("params_schema") {
        return params.clone();
    }
    serde_json::json!({"type": "object", "properties": {}})
}

pub fn tools_to_openai(tools: &[Value]) -> Vec<Value> {
    tools
        .iter()
        .map(|tool| {
            serde_json::json!({
                "type": "function",
                "function": {
                    "name": tool.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                    "description": tool.get("description").and_then(|v| v.as_str()).unwrap_or(""),
                    "parameters": tool_schema(tool),
                }
            })
        })
        .collect()
}

pub fn messages_to_openai(messages: &[Value]) -> Vec<Value> {
    messages
        .iter()
        .map(|msg| {
            let role = msg.get("role").and_then(|v| v.as_str()).unwrap_or("");
            match role {
                "assistant" => {
                    let mut d = serde_json::json!({
                        "role": "assistant",
                        "content": msg.get("content").cloned().unwrap_or(Value::Null),
                    });
                    if let Some(tool_calls) = msg.get("tool_calls") {
                        d["tool_calls"] = tool_calls.clone();
                        if d["content"].is_null() || d["content"] == "" {
                            d["content"] = Value::Null;
                        }
                    }
                    d
                }
                "tool" => {
                    serde_json::json!({
                        "role": "tool",
                        "content": msg.get("content").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        "tool_call_id": msg.get("tool_call_id").and_then(|v| v.as_str()).unwrap_or(""),
                    })
                }
                _ => {
                    serde_json::json!({
                        "role": role,
                        "content": msg.get("content").and_then(|v| v.as_str()).unwrap_or(""),
                    })
                }
            }
        })
        .collect()
}

pub fn parse_response(response: &Value, schema: Option<&Value>) -> Vec<Value> {
    let mut results = Vec::new();

    let usage_metadata = response.get("usage").map(|usage| {
        serde_json::json!({
            "input_tokens": usage.get("prompt_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
            "output_tokens": usage.get("completion_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
            "total_tokens": usage.get("total_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
        })
    });

    let choices = response
        .get("choices")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    for choice in choices {
        let msg = choice.get("message").cloned().unwrap_or(Value::Object(serde_json::Map::new()));
        let role = msg.get("role").and_then(|v| v.as_str()).unwrap_or("");
        let content = msg
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        if role == "assistant" {
            let mut result = serde_json::json!({
                "role": "assistant",
                "content": content,
            });

            if let Some(ref um) = usage_metadata {
                result["usage_metadata"] = um.clone();
            }

            if let Some(raw_tcs) = msg.get("tool_calls").and_then(|v| v.as_array()) {
                let tool_calls: Vec<Value> = raw_tcs
                    .iter()
                    .map(|tc| {
                        serde_json::json!({
                            "id": tc.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                            "type": "function",
                            "function": {
                                "name": tc.get("function").and_then(|f| f.get("name")).and_then(|v| v.as_str()).unwrap_or(""),
                                "arguments": tc.get("function").and_then(|f| f.get("arguments")).and_then(|v| v.as_str()).unwrap_or("{}"),
                            }
                        })
                    })
                    .collect();
                result["tool_calls"] = Value::Array(tool_calls);
            }

            if let Some(s) = schema {
                if !content.is_empty()
                    && result.get("tool_calls").is_none()
                {
                    if let Some(parsed) = parse_json_content(&content, s) {
                        result["parsed"] = parsed;
                    }
                }
            }

            results.push(result);
        } else {
            results.push(serde_json::json!({"role": role, "content": content}));
        }
    }
    results
}

pub struct OpenAiLlm {
    config: LlmConfig,
}

impl OpenAiLlm {
    pub fn new(api_key: &str, base_url: &str, model: &str) -> Self {
        Self {
            config: LlmConfig::new(api_key, base_url, model),
        }
    }
}

#[async_trait]
impl BaseLlm for OpenAiLlm {
    fn id(&self) -> &str {
        &self.config.id
    }
    fn api_key(&self) -> &str {
        &self.config.api_key
    }
    fn base_url(&self) -> &str {
        &self.config.base_url
    }
    fn model(&self) -> &str {
        &self.config.model
    }

    async fn run(
        &self,
        messages: Vec<Value>,
        tools: Option<Vec<Value>>,
        response_format: Option<Value>,
    ) -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
        let openai_messages = messages_to_openai(&messages);

        let mut payload = serde_json::json!({
            "model": self.config.model,
            "messages": openai_messages,
        });

        if let Some(ref tools_val) = tools {
            if !tools_val.is_empty() {
                payload["tools"] = Value::Array(tools_val.clone());
            }
        }

        if let Some(ref rf) = response_format {
            payload["response_format"] = to_response_format(rf);
        }

        for (k, v) in &self.config.extra_params {
            payload[k] = v.clone();
        }

        let client = reqwest::Client::new();
        let url = format!("{}/chat/completions", self.config.base_url.trim_end_matches('/'));

        log::debug!("LLM request to {} model={}", url, self.config.model);

        let resp = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        let status = resp.status();
        let body: Value = resp.json().await?;

        if !status.is_success() {
            let error_msg = body
                .get("error")
                .and_then(|e| e.get("message"))
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown API error");
            return Err(format!("API error {}: {}", status.as_u16(), error_msg).into());
        }

        let result = parse_response(&body, response_format.as_ref());
        if result.is_empty() {
            log::warn!("LLM returned empty choices: {:?}", body);
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_messages_to_openai_user() {
        let msgs = vec![serde_json::json!({"role": "user", "content": "hello"})];
        let result = messages_to_openai(&msgs);
        assert_eq!(result[0]["role"], "user");
        assert_eq!(result[0]["content"], "hello");
    }

    #[test]
    fn test_messages_to_openai_assistant_with_tool_calls() {
        let msgs = vec![serde_json::json!({
            "role": "assistant",
            "content": "",
            "tool_calls": [{"id": "tc1", "function": {"name": "test", "arguments": "{}"}}]
        })];
        let result = messages_to_openai(&msgs);
        assert!(result[0]["tool_calls"].is_array());
        assert!(result[0]["content"].is_null());
    }

    #[test]
    fn test_messages_to_openai_tool() {
        let msgs = vec![serde_json::json!({
            "role": "tool",
            "content": "result",
            "tool_call_id": "tc1"
        })];
        let result = messages_to_openai(&msgs);
        assert_eq!(result[0]["role"], "tool");
        assert_eq!(result[0]["tool_call_id"], "tc1");
    }

    #[test]
    fn test_parse_response_basic() {
        let response = serde_json::json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": "Hello!"
                }
            }],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 5,
                "total_tokens": 15
            }
        });
        let result = parse_response(&response, None);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0]["role"], "assistant");
        assert_eq!(result[0]["content"], "Hello!");
        assert_eq!(result[0]["usage_metadata"]["input_tokens"], 10);
    }

    #[test]
    fn test_parse_response_with_tool_calls() {
        let response = serde_json::json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": null,
                    "tool_calls": [{
                        "id": "tc1",
                        "function": {"name": "bash", "arguments": "{\"command\": \"ls\"}"}
                    }]
                }
            }]
        });
        let result = parse_response(&response, None);
        assert!(result[0].get("tool_calls").is_some());
        assert_eq!(result[0]["tool_calls"][0]["function"]["name"], "bash");
    }

    #[test]
    fn test_openai_llm_construction() {
        let llm = OpenAiLlm::new("sk-test", "https://api.openai.com/v1", "gpt-4");
        assert_eq!(llm.model(), "gpt-4");
        assert_eq!(llm.api_key(), "sk-test");
    }
}
