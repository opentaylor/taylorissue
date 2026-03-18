use async_trait::async_trait;
use serde_json::Value;

use super::base::{BaseLlm, LlmConfig};

pub fn content_to_anthropic(content: &Value) -> Value {
    match content {
        Value::String(s) => Value::String(s.clone()),
        Value::Array(arr) => {
            let mut result = Vec::new();
            for part in arr {
                if let Some(obj) = part.as_object() {
                    let ptype = obj
                        .get("type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    match ptype {
                        "text" => {
                            result.push(serde_json::json!({
                                "type": "text",
                                "text": obj.get("text").and_then(|v| v.as_str()).unwrap_or("")
                            }));
                        }
                        "image_url" => {
                            let url = obj
                                .get("image_url")
                                .and_then(|v| v.get("url"))
                                .and_then(|v| v.as_str())
                                .unwrap_or("");
                            if url.starts_with("data:") {
                                if let Some((header, data)) = url.split_once(',') {
                                    let mime = header
                                        .split(':')
                                        .nth(1)
                                        .unwrap_or("")
                                        .split(';')
                                        .next()
                                        .unwrap_or("");
                                    result.push(serde_json::json!({
                                        "type": "image",
                                        "source": {
                                            "type": "base64",
                                            "media_type": mime,
                                            "data": data
                                        }
                                    }));
                                }
                            } else {
                                result.push(serde_json::json!({
                                    "type": "image",
                                    "source": {"type": "url", "url": url}
                                }));
                            }
                        }
                        _ => result.push(part.clone()),
                    }
                } else {
                    result.push(serde_json::json!({"type": "text", "text": part.to_string()}));
                }
            }
            if result.is_empty() {
                Value::String(String::new())
            } else {
                Value::Array(result)
            }
        }
        other => Value::String(other.to_string()),
    }
}

pub fn messages_to_anthropic(messages: &[Value]) -> (Option<Value>, Vec<Value>) {
    let mut system: Option<Value> = None;
    let mut result: Vec<Value> = Vec::new();
    let mut i = 0;

    while i < messages.len() {
        let msg = &messages[i];
        let role = msg.get("role").and_then(|v| v.as_str()).unwrap_or("");

        match role {
            "system" => {
                system = Some(content_to_anthropic(
                    msg.get("content").unwrap_or(&Value::String(String::new())),
                ));
                i += 1;
            }
            "user" => {
                result.push(serde_json::json!({
                    "role": "user",
                    "content": content_to_anthropic(
                        msg.get("content").unwrap_or(&Value::String(String::new()))
                    )
                }));
                i += 1;
            }
            "assistant" => {
                let mut content_parts: Vec<Value> = Vec::new();
                let raw = content_to_anthropic(
                    msg.get("content").unwrap_or(&Value::String(String::new())),
                );
                match &raw {
                    Value::String(s) if !s.is_empty() => {
                        content_parts.push(serde_json::json!({"type": "text", "text": s}));
                    }
                    Value::Array(arr) => {
                        content_parts.extend(arr.clone());
                    }
                    _ => {}
                }
                if let Some(tool_calls) = msg.get("tool_calls").and_then(|v| v.as_array()) {
                    for tc in tool_calls {
                        let func = tc.get("function").cloned().unwrap_or(Value::Null);
                        let args_raw = func
                            .get("arguments")
                            .and_then(|v| v.as_str())
                            .unwrap_or("{}");
                        let args: Value =
                            serde_json::from_str(args_raw).unwrap_or(serde_json::json!({}));
                        content_parts.push(serde_json::json!({
                            "type": "tool_use",
                            "id": tc.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                            "name": func.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                            "input": args
                        }));
                    }
                }
                let content = if content_parts.is_empty() {
                    raw
                } else {
                    Value::Array(content_parts)
                };
                result.push(serde_json::json!({"role": "assistant", "content": content}));
                i += 1;
            }
            "tool" => {
                let mut tool_results: Vec<Value> = Vec::new();
                while i < messages.len()
                    && messages[i]
                        .get("role")
                        .and_then(|v| v.as_str())
                        == Some("tool")
                {
                    let tm = &messages[i];
                    tool_results.push(serde_json::json!({
                        "type": "tool_result",
                        "tool_use_id": tm.get("tool_call_id").and_then(|v| v.as_str()).unwrap_or(""),
                        "content": content_to_anthropic(
                            tm.get("content").unwrap_or(&Value::String(String::new()))
                        )
                    }));
                    i += 1;
                }
                result.push(serde_json::json!({"role": "user", "content": tool_results}));
            }
            _ => {
                result.push(serde_json::json!({
                    "role": role,
                    "content": content_to_anthropic(
                        msg.get("content").unwrap_or(&Value::String(String::new()))
                    )
                }));
                i += 1;
            }
        }
    }
    (system, result)
}

pub fn parse_response(response: &Value) -> Vec<Value> {
    let usage_metadata = response.get("usage").map(|usage| {
        let input = usage.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
        let output = usage.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
        serde_json::json!({
            "input_tokens": input,
            "output_tokens": output,
            "total_tokens": input + output,
        })
    });

    let content_blocks = response
        .get("content")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let mut text_parts: Vec<String> = Vec::new();
    let mut tool_calls: Vec<Value> = Vec::new();

    for block in &content_blocks {
        let btype = block.get("type").and_then(|v| v.as_str()).unwrap_or("");
        match btype {
            "text" => {
                text_parts.push(
                    block
                        .get("text")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                );
            }
            "tool_use" => {
                let input = block.get("input").cloned().unwrap_or(serde_json::json!({}));
                tool_calls.push(serde_json::json!({
                    "id": block.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                    "type": "function",
                    "function": {
                        "name": block.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                        "arguments": serde_json::to_string(&input).unwrap_or_else(|_| "{}".to_string())
                    }
                }));
            }
            _ => {}
        }
    }

    let content = if text_parts.len() > 1 {
        text_parts.join("\n\n")
    } else {
        text_parts.into_iter().next().unwrap_or_default()
    };

    let mut result = serde_json::json!({"role": "assistant", "content": content});
    if let Some(um) = usage_metadata {
        result["usage_metadata"] = um;
    }
    if !tool_calls.is_empty() {
        result["tool_calls"] = Value::Array(tool_calls);
    }

    vec![result]
}

pub fn tools_to_anthropic(tools: &[Value]) -> Vec<Value> {
    tools
        .iter()
        .map(|tool| {
            serde_json::json!({
                "name": tool.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                "description": tool.get("description").and_then(|v| v.as_str()).unwrap_or(""),
                "input_schema": super::openai::tool_schema(tool),
            })
        })
        .collect()
}

pub struct AnthropicLlm {
    config: LlmConfig,
}

impl AnthropicLlm {
    pub fn new(api_key: &str, base_url: &str, model: &str) -> Self {
        let api_key = if api_key.is_empty() {
            std::env::var("ANTHROPIC_API_KEY").unwrap_or_default()
        } else {
            api_key.to_string()
        };
        let base_url = if base_url.is_empty() {
            std::env::var("ANTHROPIC_BASE_URL")
                .unwrap_or_else(|_| "https://api.anthropic.com".to_string())
        } else {
            base_url.to_string()
        };
        let model = if model.is_empty() {
            "claude-sonnet-4-20250514".to_string()
        } else {
            model.to_string()
        };
        Self {
            config: LlmConfig::new(&api_key, &base_url, &model),
        }
    }
}

#[async_trait]
impl BaseLlm for AnthropicLlm {
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
        _response_format: Option<Value>,
    ) -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
        let (system, formatted) = messages_to_anthropic(&messages);

        let mut extra = self.config.extra_params.clone();
        let max_tokens = extra
            .remove("max_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(4096);

        let mut payload = serde_json::json!({
            "model": self.config.model,
            "messages": formatted,
            "max_tokens": max_tokens,
        });

        if let Some(sys) = system {
            payload["system"] = sys;
        }
        if let Some(ref tools_val) = tools {
            if !tools_val.is_empty() {
                payload["tools"] = Value::Array(tools_to_anthropic(tools_val));
            }
        }
        for (k, v) in &extra {
            payload[k] = v.clone();
        }

        let client = reqwest::Client::new();
        let url = format!("{}/v1/messages", self.config.base_url.trim_end_matches('/'));

        let resp = client
            .post(&url)
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        let body: Value = resp.json().await?;
        Ok(parse_response(&body))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_to_anthropic_string() {
        let result = content_to_anthropic(&Value::String("hello".to_string()));
        assert_eq!(result, Value::String("hello".to_string()));
    }

    #[test]
    fn test_messages_to_anthropic_system() {
        let msgs = vec![
            serde_json::json!({"role": "system", "content": "You are helpful."}),
            serde_json::json!({"role": "user", "content": "hi"}),
        ];
        let (system, formatted) = messages_to_anthropic(&msgs);
        assert!(system.is_some());
        assert_eq!(formatted.len(), 1);
        assert_eq!(formatted[0]["role"], "user");
    }

    #[test]
    fn test_parse_response_text() {
        let response = serde_json::json!({
            "content": [{"type": "text", "text": "Hello!"}],
            "usage": {"input_tokens": 10, "output_tokens": 5}
        });
        let result = parse_response(&response);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0]["content"], "Hello!");
        assert_eq!(result[0]["usage_metadata"]["total_tokens"], 15);
    }

    #[test]
    fn test_parse_response_with_tool_use() {
        let response = serde_json::json!({
            "content": [
                {"type": "text", "text": "Let me run that."},
                {"type": "tool_use", "id": "tc1", "name": "bash", "input": {"command": "ls"}}
            ]
        });
        let result = parse_response(&response);
        assert!(result[0].get("tool_calls").is_some());
        assert_eq!(result[0]["tool_calls"][0]["function"]["name"], "bash");
    }

    #[test]
    fn test_anthropic_llm_construction() {
        let llm = AnthropicLlm::new("sk-ant-test", "https://api.anthropic.com", "claude-sonnet-4-20250514");
        assert_eq!(llm.model(), "claude-sonnet-4-20250514");
    }
}
