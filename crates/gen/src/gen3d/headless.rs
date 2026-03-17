//! Headless Bevy generation — windowless 3D world creation.
//!
//! Provides the completion flag and detection system for headless mode.
//! In headless mode, the Bevy app runs without a window and exits
//! automatically when the agent finishes generation.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use bevy::prelude::*;

/// Shared flag between agent thread and Bevy main thread.
///
/// The agent sets `done = true` after generation completes (or fails/times out).
/// The `headless_completion_detector` system checks this each frame and
/// triggers `AppExit` when set.
#[derive(Resource, Clone)]
pub struct HeadlessCompletionFlag {
    pub done: Arc<AtomicBool>,
    pub success: Arc<AtomicBool>,
}

impl Default for HeadlessCompletionFlag {
    fn default() -> Self {
        Self {
            done: Arc::new(AtomicBool::new(false)),
            success: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl HeadlessCompletionFlag {
    /// Signal that generation has completed successfully.
    pub fn complete_success(&self) {
        self.success.store(true, Ordering::Release);
        self.done.store(true, Ordering::Release);
    }

    /// Signal that generation has failed.
    pub fn complete_failure(&self) {
        self.success.store(false, Ordering::Release);
        self.done.store(true, Ordering::Release);
    }

    /// Check if generation is done.
    pub fn is_done(&self) -> bool {
        self.done.load(Ordering::Acquire)
    }

    /// Check if generation succeeded (only meaningful if `is_done()` is true).
    pub fn is_success(&self) -> bool {
        self.success.load(Ordering::Acquire)
    }
}

/// System that checks the completion flag and triggers Bevy exit.
///
/// Added to the Update schedule in headless mode only.
/// Uses `MessageWriter<AppExit>` (Bevy 0.18 API).
pub fn headless_completion_detector(
    flag: Res<HeadlessCompletionFlag>,
    mut exit: MessageWriter<AppExit>,
) {
    if flag.is_done() {
        tracing::info!(
            "Headless generation complete (success={}), shutting down Bevy",
            flag.is_success()
        );
        exit.write(AppExit::Success);
    }
}

/// Configuration for headless generation passed from CLI.
#[derive(Debug, Clone)]
pub struct HeadlessConfig {
    /// The generation prompt.
    pub prompt: String,
    /// Output directory for the world skill.
    pub output: Option<String>,
    /// Whether to capture a screenshot after generation.
    pub screenshot: bool,
    /// Screenshot width in pixels.
    pub screenshot_width: u32,
    /// Screenshot height in pixels.
    pub screenshot_height: u32,
    /// Maximum generation time before abort.
    pub timeout_secs: u64,
    /// Agent ID for memory isolation.
    pub agent_id: String,
    /// Optional model override.
    pub model: Option<String>,
    /// Optional style hint prepended to prompt.
    pub style: Option<String>,
}

impl Default for HeadlessConfig {
    fn default() -> Self {
        Self {
            prompt: String::new(),
            output: None,
            screenshot: true,
            screenshot_width: 1280,
            screenshot_height: 720,
            timeout_secs: 300, // 5 minutes
            agent_id: "gen-headless".to_string(),
            model: None,
            style: None,
        }
    }
}

impl HeadlessConfig {
    /// Build the effective prompt, prepending style if provided.
    pub fn effective_prompt(&self) -> String {
        if let Some(ref style) = self.style {
            format!("Style: {}. {}", style, self.prompt)
        } else {
            self.prompt.clone()
        }
    }
}
