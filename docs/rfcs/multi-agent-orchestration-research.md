# Multi-Agent Orchestration Research

**Date:** 2026-02-23
**Purpose:** Research multi-agent/subagent patterns for LocalGPT architecture decisions

---

## Implementation Status

| Phase | Status | Date |
|-------|--------|------|
| Phase 1: Single-Level Delegation | ✅ **Implemented** | 2026-02-23 |
| Phase 2: Actor Infrastructure | 📋 Planned | Next quarter |
| Phase 3: Peer Agent Routing | 📋 Planned | Future |

### Phase 1 Implementation Details

- **File:** `localgpt/crates/core/src/agent/tools/spawn_agent.rs`
- **Config options:** `agent.max_spawn_depth` (default: 1), `agent.subagent_model` (optional)
- **Tool name:** `spawn_agent`
- **Task types:** `explore`, `plan`, `implement`, `analyze`
- **Depth tracking:** Via `SpawnContext` struct passed to tool

---

## Executive Summary

LocalGPT should implement a **phased hybrid approach** for multi-agent orchestration:

1. **Phase 1 (Now):** Single-level hierarchical delegation via `spawn_agent` tool
2. **Phase 2 (Next quarter):** Actor infrastructure for daemon reliability
3. **Phase 3 (Future):** Peer agent routing for multi-persona scenarios

This prioritizes task decomposition capability over complex multi-agent collaboration, following patterns proven by Claude Code and Moltis.

---

## Research Sources

| Source | Type | Key Contribution |
|--------|------|------------------|
| **OpenClaw** | TypeScript reference | Hub-and-spoke gateway, peer agents, allowlist-based communication |
| **Moltis** | Rust implementation | SpawnAgentTool, depth limits, filtered tool registries |
| **Claude Code** | Production CLI | Single-level hierarchy, Task tool, constrained spawning |
| **Industry Survey** | Multi-framework analysis | Six orchestration patterns, Cognition's critique of swarms |

---

## Three Architectural Approaches

### Approach 1: Single-Level Hierarchical Delegation

**Used by:** Claude Code, Moltis

```
Main Agent (Orchestrator)
├── SubAgent A (Explore, read-only)
├── SubAgent B (Plan, architect)
└── SubAgent C (Implement, code write)
    └── Return results via oneshot channel
    └── CANNOT spawn more subagents
```

**Implementation (Moltis):**

```rust
// crates/tools/src/spawn_agent.rs
pub struct SpawnAgentTool {
    provider_registry: Arc<RwLock<ProviderRegistry>>,
    default_provider: Arc<dyn LLMProvider>,
    tool_registry: Arc<ToolRegistry>,
    on_event: Option<OnSpawnEvent>,
}

impl SpawnAgentTool {
    async fn execute(&self, args: Value) -> Result<Value> {
        // 1. Check depth limit (max 3 levels)
        // 2. Filter tools (exclude spawn_agent)
        // 3. Build focused system prompt
        // 4. Run subagent synchronously
        // 5. Return structured result
    }
}
```

**Key constraints:**
- Maximum nesting depth: 1-3 levels
- Filtered tool registry (subagents get subset)
- Focused system prompts per task
- Synchronous execution within parent context

**Pros:**
- ✅ Simple to implement and debug
- ✅ No context fragmentation
- ✅ Clear responsibility boundaries
- ✅ Predictable resource usage
- ✅ Mobile-friendly (bounded execution)
- ✅ Works with existing Agent struct

**Cons:**
- ❌ Limited parallelism (sequential by default)
- ❌ Can't decompose subtasks further
- ❌ May need larger subagents for complex tasks

**Best for:** Code exploration, planning, implementation tasks, Gen scene generation

---

### Approach 2: Hub-and-Spoke Peer Agents

**Used by:** OpenClaw

```
Gateway / Router
├── Agent A (default)
│   ├── Workspace A
│   ├── Session A
│   └── Tools A
├── Agent B (coding specialist)
│   ├── Workspace B
│   └── Tools B
└── Agent C (gen mode)
    └── Tools C
    └── Can call Agent A via allowlist
```

**Implementation (OpenClaw):**

```typescript
// Agent configuration
type AgentConfig = {
  id: string;
  workspace?: string;
  skills?: string[];
  subagents?: {
    allowAgents?: string[];  // Which agents can be spawned
    model?: AgentModelConfig;
  };
};

// Session key hierarchy
// Main: `agent:default:main`
// Subagent: `agent:default:subagent-<uuid>`

// Routing priority
// peer match > guild > team > account > fallback
```

**Key features:**
- Each agent is fully isolated persona
- Gateway routes messages by config
- Agent-to-agent via explicit allowlist
- Serial execution by default (Lane Queue)
- Different agents per channel (Telegram, Discord)

**Pros:**
- ✅ True isolation between agents
- ✅ Completely different personas/tools
- ✅ Multi-channel routing support
- ✅ Long-running specialists

**Cons:**
- ❌ Complex state management
- ❌ Requires gateway process
- ❌ Higher resource usage
- ❌ Not ideal for mobile (overhead)
- ❌ Overkill for single-developer use

**Best for:** Multi-persona bots, multi-channel messaging, enterprise routing

---

### Approach 3: Actor-Based Supervision Tree

**Used by:** Kameo framework, recommended for Phase 2

```
Supervisor Actor (restarts failed children)
├── Agent Actor A
│   ├── mailbox: mpsc::Receiver
│   └── panic isolation
├── Agent Actor B
│   └── Can run in parallel
└── Agent Actor C
    └── Async message passing
```

**Implementation (Tokio-native):**

```rust
// Zero new dependencies - uses existing tokio
pub struct AgentActor {
    agent: Agent,
    receiver: mpsc::Receiver<AgentMessage>,
}

pub enum AgentMessage {
    Chat { input: String, reply: oneshot::Sender<Result<String>> },
    Spawn { task: String, reply: oneshot::Sender<Result<SubAgentResult>> },
    Stop,
}

impl AgentActor {
    pub fn spawn(config: Config, agent_id: String) -> (ActorRef, JoinHandle<()>) {
        let (sender, receiver) = mpsc::channel(100);

        let handle = tokio::spawn(async move {
            let agent = Agent::new(config, agent_id).unwrap();
            let mut actor = Self { agent, receiver };

            while let Some(msg) = actor.receiver.recv().await {
                actor.handle_message(msg).await;
            }
        });

        (ActorRef { sender }, handle)
    }
}
```

**With Kameo (adds ~200-400KB, supervision):**

```rust
#[actor]
pub struct AgentActor {
    agent: Agent,
}

impl AgentActor {
    #[handler]
    async fn chat(&mut self, input: String) -> Result<String> {
        self.agent.chat(&input).await
    }
}

// Supervisor handles panics automatically
let supervisor = Supervisor::new(|_: ActorRef<AgentActor>| {
    AgentActor::new(config)
});
```

**Pros:**
- ✅ Crash isolation (panic doesn't kill system)
- ✅ Natural parallelism
- ✅ Supervision for reliability
- ✅ Scales to many agents
- ✅ Mobile-compatible (tokio-based)

**Cons:**
- ❌ More complex mental model
- ❌ Debugging async issues harder
- ❌ State synchronization challenges
- ❌ Overkill for simple delegation

**Best for:** Long-running daemon, parallel tasks, Gen mode coordination

---

## Industry Patterns Analysis

From survey of AutoGPT, CrewAI, LangGraph, Microsoft AutoGen, OpenAI Agents SDK, Google A2A:

| Pattern | Used By | Key Trait | LocalGPT Fit |
|---------|---------|-----------|--------------|
| **Supervisor/Worker** | Claude Code, Amazon | Central orchestrator, result synthesis | ✅ Excellent |
| **Handoff/Delegation** | OpenAI Agents SDK | Transfer conversation between agents | ⚠️ Medium |
| **Graph-based** | LangGraph | DAG with conditional routing | ❌ Overkill |
| **Role-based Crews** | CrewAI | Organizational roles, team metaphor | ❌ Rigid |
| **Conversational** | Microsoft AutoGen | LLM-mediated chat between agents | ❌ Fragile |
| **Protocol-based** | Google A2A, MCP | JSON-RPC standardization | ✅ Future |

**Critical insight from Cognition (Devin makers):**
> "Running multiple agents in collaboration only results in fragile systems" - decision-making becomes too dispersed, context cannot be shared effectively.

**Implication:** LocalGPT should use constrained hierarchies, not open agent swarms.

---

## Recommendations for LocalGPT

### Phase 1: Single-Level Delegation (Now)

Implement Moltis-style `spawn_agent` tool:

```rust
// localgpt/crates/core/src/agent/tools/spawn_agent.rs
pub struct SpawnAgentTool {
    provider: Arc<dyn LLMProvider>,
    tool_registry: Arc<ToolRegistry>,
    max_depth: u8,  // Default: 1
}

impl Tool for SpawnAgentTool {
    async fn execute(&self, args: &str) -> Result<String> {
        let params: SpawnParams = serde_json::from_str(args)?;

        // Depth limit check
        if params.depth.unwrap_or(0) >= self.max_depth {
            return Err("Maximum spawn depth reached".into());
        }

        // Filter tools (exclude spawn_agent)
        let sub_tools = self.tool_registry.clone_without(&["spawn_agent"]);

        // Focused prompt
        let prompt = format!(
            "You are a specialist. Task: {}\n{}",
            params.task,
            "Complete and return results. Do not spawn more agents."
        );

        run_agent_loop(&*self.provider, &sub_tools, &prompt, &params.input).await
    }
}
```

**Why Phase 1:**
- Solves 80% of use cases (exploration, planning)
- Minimal code (reuses existing Agent)
- Mobile-safe (bounded execution)
- No new dependencies

---

### Phase 2: Actor Infrastructure (Next Quarter)

Add actor layer for daemon mode reliability:

```rust
// localgpt/crates/core/src/agent/actor.rs
pub struct AgentActor { /* ... */ }

impl AgentActor {
    pub fn spawn_pool(config: Config, count: usize) -> Vec<ActorRef> {
        (0..count).map(|_| Self::spawn(config.clone())).collect()
    }
}
```

**Why Phase 2:**
- Enables parallel Gen operations
- Crash isolation for daemon
- Foundation for bridge coordination

---

### Phase 3: Peer Agent Routing (Future)

Multi-persona routing for messaging:

```rust
// localgpt/crates/bridge/src/router.rs
pub struct AgentRouter {
    agents: HashMap<String, AgentActor>,
    routing_rules: Vec<RoutingRule>,
}

// Priority: peer > guild > account > default
```

**Why Phase 3:**
- Only needed for multi-persona
- Lower ROI for single-developer use

---

## Integration Points

### Gen Mode

```rust
// Gen mode uses spawn_agent for complex scene operations
let tools = vec![
    memory_search, web_fetch,          // Safe tools
    spawn_entity, modify_entity,       // Gen tools
    SpawnAgentTool::new_gen(&config),  // For complex generation
];

// Use cases:
// - Explore scene requirements
// - Generate asset descriptions
// - Coordinate multi-step creation
```

### Messaging (Bridge Daemons)

```rust
// Bridge → Agent Actor → (optional) SubAgent → Result
let result = agent_actor.chat(BridgeMessage::Incoming {
    channel: "telegram",
    text: "Create a forest scene",
}).await?;
```

### Mobile (Core Crate)

```rust
// spawn_agent available but limited:
// - max_depth = 1
// - Only safe tools
// - No bash, no file writes
```

---

## Key Files to Modify

| File | Phase | Changes |
|------|-------|---------|
| `crates/core/src/agent/tools/mod.rs` | 1 | Add spawn_agent registration |
| `crates/core/src/agent/tools/spawn_agent.rs` | 1 | New tool implementation |
| `crates/core/src/agent/session.rs` | 1 | Track parent/child sessions |
| `crates/core/src/agent/actor.rs` | 2 | New actor infrastructure |
| `crates/bridge/src/router.rs` | 3 | Agent routing logic |

---

## Binary Size Budget

| Component | Size |
|-----------|------|
| Tokio runtime | 2-3 MB |
| Current LocalGPT | ~27 MB |
| spawn_agent tool | +50-100 KB |
| Kameo (Phase 2) | +200-400 KB |
| **Phase 1 total** | ~27.1 MB |
| **Phase 2 total** | ~27.5 MB |

All phases fit within acceptable binary size.

---

## References

- **OpenClaw source:** `external/openclaw/src/agents/subagent-*.ts`
- **Moltis source:** `external/moltis/crates/tools/src/spawn_agent.rs`
- **Existing analysis:** `docs/agents/Multi-agent before multi-user.md`
- **Messaging architecture:** `docs/messaging-architecture.md`
- **Moltis survey:** `docs/moltis-survey.md`
