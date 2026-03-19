//! Docker/Podman container sandbox for tool execution.
//!
//! Spawns a persistent container at startup, executes commands via `docker exec`,
//! and cleans up on drop. Uses CLI wrapper approach (no external crate dependencies).
//!
//! Inspired by Moltis's sandbox implementation.

use anyhow::{Context, Result, bail};
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use tracing::{debug, info, warn};

/// Default container image.
const DEFAULT_IMAGE: &str = "ubuntu:24.04";

/// Default memory limit.
const DEFAULT_MEMORY: &str = "512m";

/// Container configuration.
#[derive(Debug, Clone)]
pub struct DockerConfig {
    /// Container runtime: "docker" or "podman"
    pub runtime: String,
    /// Container image
    pub image: String,
    /// Memory limit (e.g., "512m", "1g")
    pub memory: String,
    /// Workspace path to mount read-only
    pub workspace: PathBuf,
    /// Maximum output bytes
    pub max_output_bytes: usize,
}

impl DockerConfig {
    pub fn new(workspace: PathBuf) -> Self {
        Self {
            runtime: "docker".to_string(),
            image: DEFAULT_IMAGE.to_string(),
            memory: DEFAULT_MEMORY.to_string(),
            workspace,
            max_output_bytes: 200_000,
        }
    }
}

/// A running Docker/Podman sandbox container.
pub struct DockerSandbox {
    container_id: String,
    runtime: String,
    _workspace: PathBuf,
}

impl DockerSandbox {
    /// Detect which container runtime is available.
    /// Returns "docker", "podman", or None.
    pub async fn detect_runtime() -> Option<String> {
        for rt in &["docker", "podman"] {
            if let Ok(output) = Command::new(rt)
                .arg("--version")
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .output()
                .await
                && output.status.success()
            {
                let version = String::from_utf8_lossy(&output.stdout);
                debug!("Detected container runtime: {} ({})", rt, version.trim());
                return Some(rt.to_string());
            }
        }
        None
    }

    /// Start a persistent sandbox container.
    pub async fn start(config: &DockerConfig) -> Result<Self> {
        let container_name = format!("localgpt-sandbox-{}", std::process::id());

        // Clean up any stale container with same name
        let _ = Command::new(&config.runtime)
            .args(["rm", "-f", &container_name])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .output()
            .await;

        let workspace_mount = format!(
            "{}:/workspace:ro",
            config.workspace.display()
        );

        let output = Command::new(&config.runtime)
            .args([
                "run", "-d",
                "--name", &container_name,
                "--cap-drop", "ALL",
                "--security-opt", "no-new-privileges",
                "--memory", &config.memory,
                "--pids-limit", "256",
                "--network", "none",
                "-v", &workspace_mount,
                "--tmpfs", "/tmp:size=100m",
                "-w", "/workspace",
                &config.image,
                "sleep", "infinity",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .context("Failed to start sandbox container")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!(
                "Failed to start sandbox container: {}",
                stderr.trim()
            );
        }

        let container_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
        info!(
            "Docker sandbox started: {} (image: {}, memory: {})",
            &container_id[..12.min(container_id.len())],
            config.image,
            config.memory
        );

        Ok(Self {
            container_id,
            runtime: config.runtime.clone(),
            _workspace: config.workspace.clone(),
        })
    }

    /// Execute a command inside the sandbox container.
    pub async fn exec(&self, command: &str, timeout: Duration) -> Result<SandboxOutput> {
        let output = tokio::time::timeout(
            timeout,
            Command::new(&self.runtime)
                .args([
                    "exec", &self.container_id,
                    "sh", "-c", command,
                ])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output(),
        )
        .await
        .map_err(|_| anyhow::anyhow!("Sandbox command timed out after {:?}", timeout))??;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(-1);

        Ok(SandboxOutput {
            stdout,
            stderr,
            exit_code,
        })
    }

    /// Stop and remove the container.
    pub async fn stop(&self) -> Result<()> {
        debug!("Stopping sandbox container: {}", &self.container_id[..12.min(self.container_id.len())]);

        let output = Command::new(&self.runtime)
            .args(["rm", "-f", &self.container_id])
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("Failed to remove sandbox container: {}", stderr.trim());
        }

        Ok(())
    }

    /// Get the container ID.
    pub fn container_id(&self) -> &str {
        &self.container_id
    }

    /// Get the runtime name.
    pub fn runtime(&self) -> &str {
        &self.runtime
    }
}

impl Drop for DockerSandbox {
    fn drop(&mut self) {
        // Best-effort synchronous cleanup
        let rt = &self.runtime;
        let id = &self.container_id;
        let _ = std::process::Command::new(rt)
            .args(["rm", "-f", id])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .output();
    }
}

/// Output from a sandbox command execution.
#[derive(Debug)]
pub struct SandboxOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

impl SandboxOutput {
    /// Combine stdout and stderr into a single result string,
    /// matching the format expected by the existing sandbox executor.
    pub fn to_result_string(&self, max_bytes: usize) -> (String, i32) {
        let mut result = String::new();

        if !self.stdout.is_empty() {
            if self.stdout.len() > max_bytes {
                result.push_str(&self.stdout[..self.stdout.floor_char_boundary(max_bytes)]);
                result.push_str(&format!(
                    "\n\n[Output truncated, {} bytes total]",
                    self.stdout.len()
                ));
            } else {
                result.push_str(&self.stdout);
            }
        }

        if !self.stderr.is_empty() {
            if !result.is_empty() {
                result.push('\n');
            }
            let remaining = max_bytes.saturating_sub(result.len());
            if self.stderr.len() > remaining {
                result.push_str(&self.stderr[..self.stderr.floor_char_boundary(remaining)]);
                result.push_str(&format!(
                    "\n\n[Stderr truncated, {} bytes total]",
                    self.stderr.len()
                ));
            } else {
                result.push_str(&self.stderr);
            }
        }

        (result, self.exit_code)
    }
}

/// Run a command using Docker sandbox, with the same interface as `run_sandboxed`.
pub async fn run_docker_sandboxed(
    command: &str,
    sandbox: &DockerSandbox,
    timeout_ms: u64,
    max_output_bytes: usize,
) -> Result<(String, i32)> {
    let timeout = Duration::from_millis(timeout_ms);
    let output = sandbox.exec(command, timeout).await?;
    Ok(output.to_result_string(max_output_bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_docker_config_defaults() {
        let config = DockerConfig::new(PathBuf::from("/workspace"));
        assert_eq!(config.runtime, "docker");
        assert_eq!(config.image, "ubuntu:24.04");
        assert_eq!(config.memory, "512m");
    }

    #[test]
    fn test_sandbox_output_combine() {
        let output = SandboxOutput {
            stdout: "hello\n".to_string(),
            stderr: "warning\n".to_string(),
            exit_code: 0,
        };
        let (result, code) = output.to_result_string(10000);
        assert!(result.contains("hello"));
        assert!(result.contains("warning"));
        assert_eq!(code, 0);
    }

    #[test]
    fn test_sandbox_output_truncation() {
        let output = SandboxOutput {
            stdout: "x".repeat(1000),
            stderr: String::new(),
            exit_code: 0,
        };
        let (result, _) = output.to_result_string(100);
        assert!(result.len() < 200); // truncated + marker
        assert!(result.contains("[Output truncated"));
    }

    #[test]
    fn test_sandbox_output_empty() {
        let output = SandboxOutput {
            stdout: String::new(),
            stderr: String::new(),
            exit_code: 42,
        };
        let (result, code) = output.to_result_string(10000);
        assert!(result.is_empty());
        assert_eq!(code, 42);
    }

    #[tokio::test]
    async fn test_detect_runtime() {
        // This test depends on Docker being installed; just verify it doesn't crash
        let runtime = DockerSandbox::detect_runtime().await;
        // runtime may be Some("docker"), Some("podman"), or None — all valid
        if let Some(rt) = &runtime {
            assert!(rt == "docker" || rt == "podman");
        }
    }
}
