use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    pub signal_type: String,
    pub agent: String,
    pub thread_id: String,
    pub data: Value,
    pub timestamp: DateTime<Utc>,
}

impl Signal {
    pub fn new(signal_type: &str, agent: &str, thread_id: &str) -> Self {
        Self {
            signal_type: signal_type.to_string(),
            agent: agent.to_string(),
            thread_id: thread_id.to_string(),
            data: Value::Null,
            timestamp: Utc::now(),
        }
    }

    pub fn with_data(mut self, data: Value) -> Self {
        self.data = data;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_fields() {
        let sig = Signal::new("started", "agent1", "thread1");
        assert_eq!(sig.signal_type, "started");
        assert_eq!(sig.agent, "agent1");
        assert_eq!(sig.thread_id, "thread1");
        assert!(sig.data.is_null());
    }

    #[test]
    fn test_signal_with_data() {
        let sig = Signal::new("done", "a", "t").with_data(serde_json::json!({"result": "ok"}));
        assert_eq!(sig.data["result"], "ok");
    }

    #[test]
    fn test_signal_timestamp() {
        let before = Utc::now();
        let sig = Signal::new("test", "", "");
        let after = Utc::now();
        assert!(sig.timestamp >= before && sig.timestamp <= after);
    }

    #[test]
    fn test_signal_data_independence() {
        let s1 = Signal::new("a", "b", "c").with_data(serde_json::json!({"x": 1}));
        let s2 = Signal::new("a", "b", "c").with_data(serde_json::json!({"y": 2}));
        assert_ne!(s1.data, s2.data);
    }
}
