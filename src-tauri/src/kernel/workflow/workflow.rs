use std::collections::HashMap;
use serde_json::Value;

use crate::kernel::agent::Suspensions;

#[derive(Debug)]
pub struct WorkflowSuspensions {
    pub context: HashMap<String, Value>,
    pub node_id: String,
    pub suspensions: Suspensions,
}

impl WorkflowSuspensions {
    pub fn new(context: HashMap<String, Value>, node_id: &str, suspensions: Suspensions) -> Self {
        Self {
            context,
            node_id: node_id.to_string(),
            suspensions,
        }
    }
}

impl std::fmt::Display for WorkflowSuspensions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Workflow suspended at node '{}'", self.node_id)
    }
}

impl std::error::Error for WorkflowSuspensions {}

#[async_trait::async_trait]
pub trait Node: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    async fn run(
        &mut self,
        context: &mut HashMap<String, Value>,
        resume: Option<Value>,
    ) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>>;
}

pub struct Workflow {
    pub name: String,
    pub nodes: HashMap<String, Box<dyn Node>>,
    pub start: String,
}

impl Workflow {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            nodes: HashMap::new(),
            start: String::new(),
        }
    }

    pub fn add(&mut self, node: Box<dyn Node>) -> String {
        let id = node.id().to_string();
        if self.start.is_empty() {
            self.start = id.clone();
        }
        self.nodes.insert(id.clone(), node);
        id
    }

    pub async fn run(
        &mut self,
        context: &mut HashMap<String, Value>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let start = self.start.clone();
        self.run_loop(context, &start, None).await
    }

    pub async fn resume(
        &mut self,
        context: &mut HashMap<String, Value>,
        start_from: &str,
        resume: Option<Value>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.run_loop(context, start_from, resume).await
    }

    async fn run_loop(
        &mut self,
        context: &mut HashMap<String, Value>,
        start: &str,
        resume: Option<Value>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut current = Some(start.to_string());
        let mut resume_val = resume;
        let mut is_first = true;

        while let Some(ref node_id) = current {
            if node_id == "raise" {
                return Err("Workflow halted by 'raise' transition".into());
            }

            let node = self
                .nodes
                .get_mut(node_id)
                .ok_or_else(|| format!("Unknown node '{}'", node_id))?;

            let r = if is_first { resume_val.take() } else { None };
            is_first = false;

            match node.run(context, r).await {
                Ok(next) => {
                    current = next;
                }
                Err(e) => {
                    if let Some(suspensions) = e.downcast_ref::<Suspensions>() {
                        return Err(Box::new(WorkflowSuspensions::new(
                            context.clone(),
                            node_id,
                            suspensions.clone(),
                        )));
                    }
                    return Err(e);
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct StubNode {
        id: String,
        name: String,
        next: Option<String>,
    }

    #[async_trait::async_trait]
    impl Node for StubNode {
        fn id(&self) -> &str { &self.id }
        fn name(&self) -> &str { &self.name }
        async fn run(
            &mut self,
            context: &mut HashMap<String, Value>,
            _resume: Option<Value>,
        ) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
            context.insert(
                format!("visited_{}", self.id),
                Value::Bool(true),
            );
            Ok(self.next.clone())
        }
    }

    #[tokio::test]
    async fn test_workflow_single_node() {
        let mut wf = Workflow::new("test");
        wf.add(Box::new(StubNode {
            id: "n1".to_string(),
            name: "Node 1".to_string(),
            next: None,
        }));

        let mut ctx = HashMap::new();
        wf.run(&mut ctx).await.unwrap();
        assert_eq!(ctx.get("visited_n1"), Some(&Value::Bool(true)));
    }

    #[tokio::test]
    async fn test_workflow_chain() {
        let mut wf = Workflow::new("test");
        wf.add(Box::new(StubNode {
            id: "n1".to_string(),
            name: "Node 1".to_string(),
            next: Some("n2".to_string()),
        }));
        wf.add(Box::new(StubNode {
            id: "n2".to_string(),
            name: "Node 2".to_string(),
            next: None,
        }));

        let mut ctx = HashMap::new();
        wf.run(&mut ctx).await.unwrap();
        assert_eq!(ctx.get("visited_n1"), Some(&Value::Bool(true)));
        assert_eq!(ctx.get("visited_n2"), Some(&Value::Bool(true)));
    }

    #[tokio::test]
    async fn test_workflow_raise() {
        let mut wf = Workflow::new("test");
        wf.add(Box::new(StubNode {
            id: "n1".to_string(),
            name: "Node 1".to_string(),
            next: Some("raise".to_string()),
        }));

        let mut ctx = HashMap::new();
        let result = wf.run(&mut ctx).await;
        assert!(result.is_err());
    }
}
