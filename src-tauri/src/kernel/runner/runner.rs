use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::kernel::agent::{Agent, Session};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize)]
pub struct Job {
    pub id: String,
    pub agent_name: String,
    pub thread_id: String,
    pub status: JobStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Job {
    pub fn new(id: &str, agent_name: &str, thread_id: &str) -> Self {
        Self {
            id: id.to_string(),
            agent_name: agent_name.to_string(),
            thread_id: thread_id.to_string(),
            status: JobStatus::Pending,
            created_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
        }
    }
}

pub struct Runner {
    sessions: Arc<Mutex<HashMap<String, Session>>>,
    jobs: Arc<Mutex<Vec<Job>>>,
}

impl Runner {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            jobs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn get_or_create_session(&self, thread_id: &str) -> Session {
        let mut sessions = self.sessions.lock().unwrap();
        sessions
            .entry(thread_id.to_string())
            .or_insert_with(Session::new)
            .clone()
    }

    pub fn save_session(&self, thread_id: &str, session: Session) {
        let mut sessions = self.sessions.lock().unwrap();
        sessions.insert(thread_id.to_string(), session);
    }

    pub fn list_jobs(&self) -> Vec<Job> {
        self.jobs.lock().unwrap().clone()
    }
}

impl Default for Runner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_status() {
        let job = Job::new("j1", "agent", "thread");
        assert_eq!(job.status, JobStatus::Pending);
        assert!(job.started_at.is_none());
    }

    #[test]
    fn test_runner_sessions() {
        let runner = Runner::new();
        let s1 = runner.get_or_create_session("t1");
        assert!(s1.messages.is_empty());

        let mut s2 = runner.get_or_create_session("t1");
        s2.messages.push(serde_json::json!({"role": "user", "content": "hi"}));
        runner.save_session("t1", s2);

        let s3 = runner.get_or_create_session("t1");
        assert_eq!(s3.messages.len(), 1);
    }
}
