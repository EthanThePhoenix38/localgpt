//! Lifecycle hook system for LocalGPT
//!
//! Hooks are shell commands that fire at key points in the agent pipeline:
//! - before_tool_call: Before a tool executes (can block)
//! - after_tool_call: After a tool completes
//! - on_message: When a user message is received
//! - on_session_start: When a session is created
//! - on_session_end: When a session ends
//!
//! Hook definitions are JSON files in:
//! - workspace/hooks/*.hook.json
//! - ~/.local/share/localgpt/hooks/*.hook.json (global, XDG_DATA_HOME)
//!
//! Example hook file:
//! ```json
//! {
//!   "name": "audit-log",
//!   "event": "after_tool_call",
//!   "command": "/usr/local/bin/log-tool-call",
//!   "timeout_ms": 5000,
//!   "enabled": true
//! }
//! ```
//!
//! The event JSON is piped to the hook command's stdin.
//! Exit code 0 = allow, non-zero = block (for modifying hooks only).

mod discovery;
mod event;
mod runner;

pub use discovery::{HookDef, discover_hooks};
pub use event::HookEvent;
pub use runner::{HookDecision, HookEngine};
