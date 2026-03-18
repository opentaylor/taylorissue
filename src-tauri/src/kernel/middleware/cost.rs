use async_trait::async_trait;
use serde_json::Value;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::kernel::agent::Agent;
use crate::kernel::util::store::JsonlFile;
use super::base::Middleware;

#[derive(Debug, Clone, Default)]
pub struct CostRecord {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
    pub model: String,
}

pub struct CostMiddleware {
    pub records: Arc<Mutex<Vec<CostRecord>>>,
}

impl CostMiddleware {
    pub fn new() -> Self {
        Self {
            records: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn total_tokens(&self) -> u64 {
        self.records
            .lock()
            .unwrap()
            .iter()
            .map(|r| r.total_tokens)
            .sum()
    }
}

impl Default for CostMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Middleware for CostMiddleware {}

pub struct SessionCostMiddleware {
    pub base_dir: PathBuf,
}

impl SessionCostMiddleware {
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }
}

#[async_trait]
impl Middleware for SessionCostMiddleware {}

pub fn load_cost_jsonl(path: &PathBuf) -> Vec<CostRecord> {
    let file = JsonlFile::new(path.clone());
    file.read_all()
        .into_iter()
        .filter_map(|v| {
            Some(CostRecord {
                input_tokens: v.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
                output_tokens: v.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
                total_tokens: v.get("total_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
                model: v
                    .get("model")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_middleware_total() {
        let mw = CostMiddleware::new();
        {
            let mut records = mw.records.lock().unwrap();
            records.push(CostRecord {
                input_tokens: 10,
                output_tokens: 5,
                total_tokens: 15,
                model: "gpt-4".to_string(),
            });
            records.push(CostRecord {
                input_tokens: 20,
                output_tokens: 10,
                total_tokens: 30,
                model: "gpt-4".to_string(),
            });
        }
        assert_eq!(mw.total_tokens(), 45);
    }
}
