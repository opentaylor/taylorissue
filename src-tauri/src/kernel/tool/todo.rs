use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::{Arc, Mutex};

use super::base::{schema_for, BaseTool};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TodoStatus {
    Pending,
    InProgress,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TodoItem {
    pub id: String,
    pub content: String,
    pub status: TodoStatus,
}

pub type TodoState = Arc<Mutex<Vec<TodoItem>>>;

pub fn new_todo_state() -> TodoState {
    Arc::new(Mutex::new(Vec::new()))
}

pub fn format_todo_list(todos: &[TodoItem]) -> String {
    if todos.is_empty() {
        return "No todos.".to_string();
    }
    todos
        .iter()
        .map(|t| {
            let status = match t.status {
                TodoStatus::Pending => "pending",
                TodoStatus::InProgress => "in_progress",
                TodoStatus::Completed => "completed",
                TodoStatus::Cancelled => "cancelled",
            };
            format!("- [{}] {} ({})", t.id, t.content, status)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[derive(Deserialize, JsonSchema)]
struct WriteTodosArgs {
    /// Array of todo items to create or update
    todos: Vec<TodoItem>,
    /// If true, merge with existing todos by id. If false, replace all.
    #[serde(default)]
    merge: Option<bool>,
}

pub struct WriteTodosTool {
    state: TodoState,
}

impl WriteTodosTool {
    pub fn new(state: TodoState) -> Self {
        Self { state }
    }
}

#[async_trait]
impl BaseTool for WriteTodosTool {
    fn name(&self) -> &str { "write_todos" }

    fn description(&self) -> &str {
        "Create or update a structured todo list for task planning and progress tracking."
    }

    fn params_schema(&self) -> Value { schema_for::<WriteTodosArgs>() }

    async fn run(&self, args: Value) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let args: WriteTodosArgs = serde_json::from_value(args)?;
        let merge = args.merge.unwrap_or(false);

        let mut state = self.state.lock().unwrap();

        if merge {
            for new_item in args.todos {
                if let Some(existing) = state.iter_mut().find(|t| t.id == new_item.id) {
                    existing.content = new_item.content;
                    existing.status = new_item.status;
                } else {
                    state.push(new_item);
                }
            }
        } else {
            *state = args.todos;
        }

        Ok(format_todo_list(&state))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_write_todos_replace() {
        let state = new_todo_state();
        let tool = WriteTodosTool::new(state.clone());

        tool.run(serde_json::json!({
            "todos": [
                {"id": "1", "content": "First task", "status": "pending"},
                {"id": "2", "content": "Second task", "status": "in_progress"}
            ]
        })).await.unwrap();

        let todos = state.lock().unwrap();
        assert_eq!(todos.len(), 2);
        assert_eq!(todos[0].id, "1");
        assert_eq!(todos[1].status, TodoStatus::InProgress);
    }

    #[tokio::test]
    async fn test_write_todos_merge() {
        let state = new_todo_state();
        let tool = WriteTodosTool::new(state.clone());

        tool.run(serde_json::json!({
            "todos": [
                {"id": "1", "content": "Task A", "status": "pending"}
            ]
        })).await.unwrap();

        tool.run(serde_json::json!({
            "todos": [
                {"id": "1", "content": "Task A", "status": "completed"},
                {"id": "2", "content": "Task B", "status": "pending"}
            ],
            "merge": true
        })).await.unwrap();

        let todos = state.lock().unwrap();
        assert_eq!(todos.len(), 2);
        assert_eq!(todos[0].status, TodoStatus::Completed);
    }

    #[test]
    fn test_format_todo_list_empty() {
        assert_eq!(format_todo_list(&[]), "No todos.");
    }

    #[test]
    fn test_format_todo_list_all_statuses() {
        let todos = vec![
            TodoItem { id: "1".into(), content: "A".into(), status: TodoStatus::Pending },
            TodoItem { id: "2".into(), content: "B".into(), status: TodoStatus::InProgress },
            TodoItem { id: "3".into(), content: "C".into(), status: TodoStatus::Completed },
            TodoItem { id: "4".into(), content: "D".into(), status: TodoStatus::Cancelled },
        ];
        let output = format_todo_list(&todos);
        assert!(output.contains("(pending)"));
        assert!(output.contains("(in_progress)"));
        assert!(output.contains("(completed)"));
        assert!(output.contains("(cancelled)"));
    }

    #[test]
    fn test_write_todos_schema() {
        let state = new_todo_state();
        let tool = WriteTodosTool::new(state);
        let schema = tool.params_schema();
        assert!(schema.get("properties").is_some());
        let props = schema["properties"].as_object().unwrap();
        assert!(props.contains_key("todos"));
        assert!(props.contains_key("merge"));
    }

    #[tokio::test]
    async fn test_write_todos_returns_formatted_list() {
        let state = new_todo_state();
        let tool = WriteTodosTool::new(state);
        let result = tool.run(serde_json::json!({
            "todos": [{"id": "x", "content": "Do X", "status": "in_progress"}]
        })).await.unwrap();
        assert!(result.contains("[x] Do X (in_progress)"));
    }
}
