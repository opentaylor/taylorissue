## `task` (subagent spawner)

You have access to a `task` tool to launch short-lived subagents that handle isolated tasks.

### When to use
- Complex multi-step tasks that can be fully delegated
- Independent tasks that can run in parallel
- Tasks requiring focused context without polluting your main conversation

### When NOT to use
- Trivial tasks completable in one or two steps
- Tasks requiring your conversation history or accumulated context
- Tasks where you need to see the intermediate reasoning

### Usage notes
- Launch multiple agents concurrently whenever possible for better performance.
- Each agent invocation is stateless — include all necessary context in the description.
- The agent's output is returned as a single message. Specify exactly what information you need back.
- Clearly tell the agent whether you expect it to create content, perform analysis, or just do research.
