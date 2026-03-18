use tiktoken_rs::CoreBPE;

pub struct TiktokenCounter {
    bpe: CoreBPE,
}

impl TiktokenCounter {
    pub fn new(model: &str) -> Self {
        let bpe = tiktoken_rs::get_bpe_from_model(model)
            .unwrap_or_else(|_| tiktoken_rs::cl100k_base().unwrap());
        Self { bpe }
    }

    pub fn count_text(&self, text: &str) -> usize {
        self.bpe.encode_with_special_tokens(text).len()
    }

    pub fn count_message(&self, message: &serde_json::Value) -> usize {
        let mut tokens = 4; // <im_start>, role, \n, <im_end>
        if let Some(content) = message.get("content").and_then(|v| v.as_str()) {
            tokens += self.count_text(content);
        }
        if let Some(role) = message.get("role").and_then(|v| v.as_str()) {
            tokens += self.count_text(role);
        }
        if let Some(tool_calls) = message.get("tool_calls").and_then(|v| v.as_array()) {
            for tc in tool_calls {
                if let Some(func) = tc.get("function") {
                    if let Some(name) = func.get("name").and_then(|v| v.as_str()) {
                        tokens += self.count_text(name);
                    }
                    if let Some(args) = func.get("arguments").and_then(|v| v.as_str()) {
                        tokens += self.count_text(args);
                    }
                }
            }
        }
        tokens
    }

    pub fn count_messages(&self, messages: &[serde_json::Value]) -> usize {
        let mut total = 3; // priming tokens
        for msg in messages {
            total += self.count_message(msg);
        }
        total
    }
}

pub struct CharEstimateCounter;

impl CharEstimateCounter {
    pub fn count_text(text: &str) -> usize {
        text.len() / 4
    }

    pub fn count_message(message: &serde_json::Value) -> usize {
        let content = message
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        Self::count_text(content) + 4
    }

    pub fn count_messages(messages: &[serde_json::Value]) -> usize {
        messages.iter().map(Self::count_message).sum::<usize>() + 3
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tiktoken_counter_construction() {
        let counter = TiktokenCounter::new("gpt-4");
        assert!(counter.count_text("hello world") > 0);
    }

    #[test]
    fn test_tiktoken_count_message() {
        let counter = TiktokenCounter::new("gpt-4");
        let msg = serde_json::json!({"role": "user", "content": "hello"});
        let count = counter.count_message(&msg);
        assert!(count > 4);
    }

    #[test]
    fn test_tiktoken_count_messages() {
        let counter = TiktokenCounter::new("gpt-4");
        let msgs = vec![
            serde_json::json!({"role": "user", "content": "hello"}),
            serde_json::json!({"role": "assistant", "content": "hi there"}),
        ];
        let count = counter.count_messages(&msgs);
        assert!(count > 3);
    }

    #[test]
    fn test_char_estimate_counter() {
        let msgs = vec![serde_json::json!({"role": "user", "content": "hello world"})];
        let count = CharEstimateCounter::count_messages(&msgs);
        assert!(count > 0);
    }
}
