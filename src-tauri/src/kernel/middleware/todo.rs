use async_trait::async_trait;

use crate::kernel::agent::Agent;
use crate::kernel::tool::todo::{format_todo_list, new_todo_state, TodoState, WriteTodosTool};
use super::base::{AgentError, Middleware, Next};

const TODO_PROMPT: &str = include_str!("../../prompts/kernel/write_todos.md");

pub struct TodoMiddleware {
    state: TodoState,
}

impl TodoMiddleware {
    pub fn new() -> (Self, WriteTodosTool) {
        let state = new_todo_state();
        let middleware = Self { state: state.clone() };
        let tool = WriteTodosTool::new(state);
        (middleware, tool)
    }

    pub fn with_state(state: TodoState) -> Self {
        Self { state }
    }
}

#[async_trait]
impl Middleware for TodoMiddleware {
    async fn wrap_llm(&self, ctx: &mut Agent, next: Next<'_>) -> Result<(), AgentError> {
        let todos = self.state.lock().unwrap().clone();

        let mut prompt_parts = vec![TODO_PROMPT.to_string()];

        if !todos.is_empty() {
            prompt_parts.push(format!(
                "\n### Current Todo List\n\n{}",
                format_todo_list(&todos)
            ));
        }

        ctx.prefix_messages.push(serde_json::json!({
            "role": "system",
            "content": prompt_parts.join("\n")
        }));

        next.call(ctx).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::tool::base::BaseTool;
    use crate::kernel::tool::todo::{TodoItem, TodoStatus};

    #[test]
    fn test_todo_middleware_creates_pair() {
        let (mw, tool) = TodoMiddleware::new();
        assert!(mw.state.lock().unwrap().is_empty());
        assert_eq!(tool.name(), "write_todos");
    }

    #[test]
    fn test_shared_state() {
        let (mw, _tool) = TodoMiddleware::new();
        {
            let mut todos = mw.state.lock().unwrap();
            todos.push(TodoItem {
                id: "1".to_string(),
                content: "Test".to_string(),
                status: TodoStatus::Pending,
            });
        }
        assert_eq!(mw.state.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_wrap_llm_injects_system_message() {
        use crate::kernel::agent::Agent;
        use crate::kernel::middleware::base::{Next, Phase};

        let (mw, _tool) = TodoMiddleware::new();
        let mut agent = Agent::new();
        let mws: Vec<Box<dyn crate::kernel::middleware::base::Middleware>> = vec![];
        let next = Next { remaining: &mws, phase: Phase::Start };

        mw.wrap_llm(&mut agent, next).await.unwrap();

        assert_eq!(agent.prefix_messages.len(), 1);
        assert_eq!(agent.prefix_messages[0]["role"], "system");
        let content = agent.prefix_messages[0]["content"].as_str().unwrap();
        assert!(content.contains("write_todos"));
    }

    #[tokio::test]
    async fn test_wrap_llm_includes_todo_state() {
        use crate::kernel::agent::Agent;
        use crate::kernel::middleware::base::{Next, Phase};

        let (mw, _tool) = TodoMiddleware::new();
        {
            let mut todos = mw.state.lock().unwrap();
            todos.push(TodoItem {
                id: "t1".to_string(),
                content: "Do the thing".to_string(),
                status: TodoStatus::InProgress,
            });
        }

        let mut agent = Agent::new();
        let mws: Vec<Box<dyn crate::kernel::middleware::base::Middleware>> = vec![];
        let next = Next { remaining: &mws, phase: Phase::Start };

        mw.wrap_llm(&mut agent, next).await.unwrap();

        let content = agent.prefix_messages[0]["content"].as_str().unwrap();
        assert!(content.contains("Do the thing"));
        assert!(content.contains("in_progress"));
    }

    #[test]
    fn test_with_state() {
        let state = crate::kernel::tool::todo::new_todo_state();
        {
            let mut todos = state.lock().unwrap();
            todos.push(TodoItem {
                id: "pre".to_string(),
                content: "Pre-existing".to_string(),
                status: TodoStatus::Completed,
            });
        }
        let mw = TodoMiddleware::with_state(state.clone());
        assert_eq!(mw.state.lock().unwrap().len(), 1);
    }
}
