//! Append-only, hash-chained security audit log.
//!
//! Stored at `~/.local/state/localgpt/localgpt.audit.jsonl` (outside the workspace,
//! in the state directory). Each entry contains a SHA-256 hash of the
//! previous entry, forming a tamper-evident chain.
//!
//! # Format
//!
//! One JSON object per line (JSONL). Each entry includes:
//!
//! | Field | Description |
//! |-------|-------------|
//! | `ts` | ISO 8601 timestamp |
//! | `action` | What happened: `signed`, `verified`, `tamper_detected`, etc. |
//! | `content_sha256` | SHA-256 of the policy content at the time |
//! | `prev_entry_sha256` | SHA-256 of the previous JSONL line (chain link) |
//! | `source` | Who triggered it: `cli`, `gui`, or `session_start` |
//!
//! # Chain Integrity
//!
//! The first entry uses `000...000` (64 zeros) as `prev_entry_sha256`.
//! Every subsequent entry hashes the raw bytes of the previous line.
//! A broken chain indicates the log file was tampered with.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

const AUDIT_FILENAME: &str = "localgpt.audit.jsonl";

/// The hash used for the first entry in the chain (no predecessor).
const GENESIS_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// Security audit log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// ISO 8601 timestamp of the event.
    pub ts: String,
    /// What security action occurred.
    pub action: AuditAction,
    /// SHA-256 of the policy content at the time (hex-encoded).
    pub content_sha256: String,
    /// SHA-256 of the previous JSONL line (chain link, hex-encoded).
    pub prev_entry_sha256: String,
    /// Who triggered the action: `"cli"`, `"session_start"`, `"tool:{name}"`, etc.
    pub source: String,
    /// Optional context. Tool name and path for `WriteBlocked`, patterns for `SuspiciousContent`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// Security actions recorded in the audit log.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    /// Policy template was created during workspace init.
    Created,
    /// Policy was signed via CLI or GUI.
    Signed,
    /// Policy was verified at session start.
    Verified,
    /// Tamper detected (HMAC mismatch or manifest corruption).
    TamperDetected,
    /// Policy file is missing from the workspace.
    Missing,
    /// Policy exists but has no manifest (not yet signed).
    Unsigned,
    /// Manifest JSON parse failure.
    ManifestCorrupted,
    /// Policy contains suspicious prompt injection patterns.
    SuspiciousContent,
    /// File watcher detected LocalGPT.md modification mid-session.
    FileChanged,
    /// Agent tool attempted to write a protected file.
    WriteBlocked,
    /// Tool attempted to access a path outside allowed directories.
    PathDenied,
    /// Previous audit entry corrupted, new chain segment started.
    ChainRecovery,
    /// Session context window was compacted (messages truncated + summarized).
    Compaction,
}

/// Append a new entry to the audit log.
///
/// Reads the last line of the existing log (if any) to compute the
/// chain hash, then appends a new JSONL line. If the last line is
/// corrupted (not valid JSON), a `ChainRecovery` entry is inserted
/// first to record the break point.
///
/// # Arguments
///
/// * `state_dir` — Path to `~/.local/state/localgpt/` (contains the audit log).
/// * `action` — What security event occurred.
/// * `content_sha256` — SHA-256 of the policy content (empty string if N/A).
/// * `source` — Who triggered the action.
pub fn append_audit_entry(
    state_dir: &Path,
    action: AuditAction,
    content_sha256: &str,
    source: &str,
) -> Result<()> {
    append_audit_entry_with_detail(state_dir, action, content_sha256, source, None)
}

/// Append a new entry to the audit log with an optional detail message.
pub fn append_audit_entry_with_detail(
    state_dir: &Path,
    action: AuditAction,
    content_sha256: &str,
    source: &str,
    detail: Option<&str>,
) -> Result<()> {
    let path = audit_file_path(state_dir);

    // Read the last line to compute the chain hash, with corruption recovery
    let prev_hash = if path.exists() {
        let content = fs::read_to_string(&path).context("Failed to read audit log")?;
        match content.lines().last() {
            Some(last_line) if !last_line.is_empty() => {
                // Attempt to parse as JSON to detect corruption
                if serde_json::from_str::<AuditEntry>(last_line).is_ok() {
                    sha256_hex(last_line.as_bytes())
                } else {
                    // Corrupted last line — write a ChainRecovery entry first
                    let raw_hash = sha256_hex(last_line.as_bytes());
                    let recovery = AuditEntry {
                        ts: chrono::Utc::now().to_rfc3339(),
                        action: AuditAction::ChainRecovery,
                        content_sha256: String::new(),
                        prev_entry_sha256: raw_hash,
                        source: "audit_system".to_string(),
                        detail: Some(format!(
                            "Previous entry corrupted ({} bytes), new chain segment",
                            last_line.len()
                        )),
                    };
                    let recovery_json = serde_json::to_string(&recovery)
                        .context("Failed to serialize recovery entry")?;
                    append_line(&path, &recovery_json)?;
                    sha256_hex(recovery_json.as_bytes())
                }
            }
            _ => GENESIS_HASH.to_string(),
        }
    } else {
        GENESIS_HASH.to_string()
    };

    let entry = AuditEntry {
        ts: chrono::Utc::now().to_rfc3339(),
        action,
        content_sha256: content_sha256.to_string(),
        prev_entry_sha256: prev_hash,
        source: source.to_string(),
        detail: detail.map(|d| d.to_string()),
    };

    let json = serde_json::to_string(&entry).context("Failed to serialize audit entry")?;
    append_line(&path, &json)?;

    Ok(())
}

/// Append a single line to a file.
fn append_line(path: &Path, line: &str) -> Result<()> {
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .context("Failed to open audit log")?;
    writeln!(file, "{}", line).context("Failed to write audit entry")?;
    Ok(())
}

/// Read and parse all entries from the audit log.
///
/// Corrupted lines are skipped (not fatal). Returns an empty vector
/// if the log file does not exist.
pub fn read_audit_log(state_dir: &Path) -> Result<Vec<AuditEntry>> {
    let path = audit_file_path(state_dir);

    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&path).context("Failed to read audit log")?;
    let mut entries = Vec::new();

    for line in content.lines() {
        if line.is_empty() {
            continue;
        }
        // Skip corrupted lines rather than failing (RFC §5.6)
        if let Ok(entry) = serde_json::from_str::<AuditEntry>(line) {
            entries.push(entry);
        }
    }

    Ok(entries)
}

/// Verify the integrity of the audit log hash chain.
///
/// Returns a list of indices where the chain is broken (i.e., the
/// `prev_entry_sha256` does not match the SHA-256 of the previous line).
/// Corrupted (non-JSON) lines are reported as broken and skipped.
///
/// An empty return value means the chain is intact.
pub fn verify_audit_chain(state_dir: &Path) -> Result<Vec<usize>> {
    let path = audit_file_path(state_dir);

    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&path).context("Failed to read audit log")?;
    let lines: Vec<&str> = content.lines().filter(|l| !l.is_empty()).collect();

    if lines.is_empty() {
        return Ok(Vec::new());
    }

    let mut broken = Vec::new();

    // Parse entries, tracking which lines are valid
    let mut parsed: Vec<Option<AuditEntry>> = Vec::new();
    for line in &lines {
        parsed.push(serde_json::from_str(line).ok());
    }

    // Check first entry
    if let Some(ref first) = parsed[0] {
        if first.prev_entry_sha256 != GENESIS_HASH {
            broken.push(0);
        }
    } else {
        broken.push(0); // Corrupted first line
    }

    // Check chain links
    for i in 1..lines.len() {
        if parsed[i].is_none() {
            broken.push(i); // Corrupted line
            continue;
        }
        let expected_hash = sha256_hex(lines[i - 1].as_bytes());
        if parsed[i].as_ref().unwrap().prev_entry_sha256 != expected_hash {
            broken.push(i);
        }
    }

    Ok(broken)
}

/// Get the full path to the audit log file.
pub fn audit_file_path(state_dir: &Path) -> PathBuf {
    state_dir.join(AUDIT_FILENAME)
}

/// Compute hex-encoded SHA-256.
fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect()
}

// ── Compaction Audit Helpers ────────────────────────────────────────

/// Structured details for a compaction audit entry.
///
/// Serialized as JSON in the `detail` field of `AuditEntry`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionDetail {
    pub session_id: String,
    pub messages_before: usize,
    pub messages_after: usize,
    pub tokens_before: usize,
    pub tokens_after: usize,
    pub strategy: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub injected_sections: Vec<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub summary_preview: String,
}

/// Log a compaction event to the audit log.
///
/// Creates a `Compaction` audit entry with structured compaction details
/// serialized as JSON in the `detail` field.
pub fn append_compaction_entry(state_dir: &Path, detail: &CompactionDetail) -> Result<()> {
    let detail_json =
        serde_json::to_string(detail).context("Failed to serialize compaction detail")?;
    append_audit_entry_with_detail(
        state_dir,
        AuditAction::Compaction,
        "",
        &format!("session:{}", detail.session_id),
        Some(&detail_json),
    )
}

/// Read compaction entries from the audit log.
///
/// Returns all entries with `action == Compaction`, with their
/// `detail` field parsed into `CompactionDetail`.
pub fn read_compaction_entries(
    state_dir: &Path,
) -> Result<Vec<(AuditEntry, Option<CompactionDetail>)>> {
    let entries = read_audit_log(state_dir)?;
    Ok(entries
        .into_iter()
        .filter(|e| e.action == AuditAction::Compaction)
        .map(|e| {
            let detail = e
                .detail
                .as_deref()
                .and_then(|d| serde_json::from_str::<CompactionDetail>(d).ok());
            (e, detail)
        })
        .collect())
}

/// Aggregate statistics for compaction events.
#[derive(Debug)]
pub struct CompactionStats {
    pub total_events: usize,
    pub total_messages_compacted: usize,
    pub total_tokens_saved: usize,
    pub last_compaction: Option<String>,
}

/// Compute aggregate statistics from compaction audit entries.
pub fn compaction_stats(state_dir: &Path) -> Result<CompactionStats> {
    let entries = read_compaction_entries(state_dir)?;

    let total_events = entries.len();
    let mut total_messages_compacted: usize = 0;
    let mut total_tokens_saved: usize = 0;
    let mut last_compaction: Option<String> = None;

    for (entry, detail) in &entries {
        if let Some(d) = detail {
            total_messages_compacted += d.messages_before.saturating_sub(d.messages_after);
            total_tokens_saved += d.tokens_before.saturating_sub(d.tokens_after);
        }
        last_compaction = Some(entry.ts.clone());
    }

    Ok(CompactionStats {
        total_events,
        total_messages_compacted,
        total_tokens_saved,
        last_compaction,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audit_chain_integrity() {
        let tmp = tempfile::tempdir().unwrap();

        // Write 5 entries
        for i in 0..5 {
            append_audit_entry(
                tmp.path(),
                AuditAction::Verified,
                &format!("sha256_{}", i),
                "test",
            )
            .unwrap();
        }

        let entries = read_audit_log(tmp.path()).unwrap();
        assert_eq!(entries.len(), 5);

        // Chain should be intact
        let broken = verify_audit_chain(tmp.path()).unwrap();
        assert!(broken.is_empty(), "Chain should be intact: {:?}", broken);
    }

    #[test]
    fn first_entry_uses_genesis_hash() {
        let tmp = tempfile::tempdir().unwrap();
        append_audit_entry(tmp.path(), AuditAction::Created, "abc123", "cli").unwrap();

        let entries = read_audit_log(tmp.path()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].prev_entry_sha256, GENESIS_HASH);
        assert_eq!(entries[0].action, AuditAction::Created);
    }

    #[test]
    fn broken_chain_detected() {
        let tmp = tempfile::tempdir().unwrap();

        // Write 3 valid entries
        for i in 0..3 {
            append_audit_entry(
                tmp.path(),
                AuditAction::Verified,
                &format!("sha256_{}", i),
                "test",
            )
            .unwrap();
        }

        // Tamper with the middle line
        let path = audit_file_path(tmp.path());
        let content = fs::read_to_string(&path).unwrap();
        let mut lines: Vec<&str> = content.lines().collect();

        // Replace middle line with different content (breaks chain for entry 2)
        let tampered = lines[1].replace("sha256_1", "tampered_hash");
        lines[1] = &tampered;

        fs::write(&path, lines.join("\n") + "\n").unwrap();

        let broken = verify_audit_chain(tmp.path()).unwrap();
        assert!(!broken.is_empty(), "Should detect broken chain");
        assert!(broken.contains(&2), "Entry 2 should have broken link");
    }

    #[test]
    fn empty_log_no_errors() {
        let tmp = tempfile::tempdir().unwrap();

        let entries = read_audit_log(tmp.path()).unwrap();
        assert!(entries.is_empty());

        let broken = verify_audit_chain(tmp.path()).unwrap();
        assert!(broken.is_empty());
    }

    #[test]
    fn audit_actions_serialize_snake_case() {
        let entry = AuditEntry {
            ts: "2026-02-09T14:00:00Z".to_string(),
            action: AuditAction::TamperDetected,
            content_sha256: "abc".to_string(),
            prev_entry_sha256: GENESIS_HASH.to_string(),
            source: "cli".to_string(),
            detail: None,
        };

        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"tamper_detected\""));
        // detail should be omitted when None
        assert!(!json.contains("\"detail\""));
    }

    #[test]
    fn detail_field_serialized_when_present() {
        let entry = AuditEntry {
            ts: "2026-02-09T14:00:00Z".to_string(),
            action: AuditAction::WriteBlocked,
            content_sha256: String::new(),
            prev_entry_sha256: GENESIS_HASH.to_string(),
            source: "tool:write_file".to_string(),
            detail: Some("Agent attempted write to LocalGPT.md".to_string()),
        };

        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"write_blocked\""));
        assert!(json.contains("Agent attempted write to LocalGPT.md"));

        // Roundtrip
        let parsed: AuditEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(
            parsed.detail.unwrap(),
            "Agent attempted write to LocalGPT.md"
        );
    }

    #[test]
    fn chain_recovery_on_corrupted_line() {
        let tmp = tempfile::tempdir().unwrap();

        // Write 2 valid entries
        append_audit_entry(tmp.path(), AuditAction::Signed, "abc", "cli").unwrap();
        append_audit_entry(tmp.path(), AuditAction::Verified, "abc", "session_start").unwrap();

        // Corrupt the last line
        let path = audit_file_path(tmp.path());
        let mut content = fs::read_to_string(&path).unwrap();
        content.push_str("this is not json\n");
        fs::write(&path, &content).unwrap();

        // Next append should trigger ChainRecovery
        append_audit_entry(tmp.path(), AuditAction::Verified, "abc", "session_start").unwrap();

        let entries = read_audit_log(tmp.path()).unwrap();
        // Should have: Signed, Verified, ChainRecovery, Verified (corrupted line skipped)
        assert_eq!(entries.len(), 4);
        assert_eq!(entries[2].action, AuditAction::ChainRecovery);
        assert!(entries[2].detail.as_ref().unwrap().contains("corrupted"));
        assert_eq!(entries[2].source, "audit_system");
    }

    #[test]
    fn corrupted_lines_skipped_in_read() {
        let tmp = tempfile::tempdir().unwrap();
        let path = audit_file_path(tmp.path());

        // Write a valid entry, then garbage, then another valid entry
        append_audit_entry(tmp.path(), AuditAction::Signed, "abc", "cli").unwrap();
        // Insert garbage directly
        let mut file = fs::OpenOptions::new().append(true).open(&path).unwrap();
        writeln!(file, "not valid json garbage").unwrap();
        drop(file);
        // The next append will trigger chain recovery
        append_audit_entry(tmp.path(), AuditAction::Verified, "abc", "test").unwrap();

        let entries = read_audit_log(tmp.path()).unwrap();
        // Signed + ChainRecovery + Verified = 3 (garbage line skipped)
        assert_eq!(entries.len(), 3);
    }

    // ── Compaction audit tests ──────────────────────────────────────

    fn sample_compaction_detail(session_id: &str, before: usize, after: usize) -> CompactionDetail {
        CompactionDetail {
            session_id: session_id.to_string(),
            messages_before: before,
            messages_after: after,
            tokens_before: before * 100,
            tokens_after: after * 100,
            strategy: "summarize_and_truncate".to_string(),
            injected_sections: vec!["Session Startup".to_string()],
            summary_preview: String::new(),
        }
    }

    #[test]
    fn compaction_log_and_read_back() {
        let tmp = tempfile::tempdir().unwrap();

        let detail = sample_compaction_detail("sess-1", 20, 5);
        append_compaction_entry(tmp.path(), &detail).unwrap();

        let entries = read_compaction_entries(tmp.path()).unwrap();
        assert_eq!(entries.len(), 1);

        let (entry, parsed) = &entries[0];
        assert_eq!(entry.action, AuditAction::Compaction);
        assert_eq!(entry.source, "session:sess-1");

        let d = parsed.as_ref().unwrap();
        assert_eq!(d.messages_before, 20);
        assert_eq!(d.messages_after, 5);
        assert_eq!(d.tokens_before, 2000);
        assert_eq!(d.tokens_after, 500);
        assert_eq!(d.strategy, "summarize_and_truncate");
        assert_eq!(d.injected_sections, vec!["Session Startup"]);
    }

    #[test]
    fn compaction_hash_chain_integrity() {
        let tmp = tempfile::tempdir().unwrap();

        // Mix security and compaction entries
        append_audit_entry(tmp.path(), AuditAction::Verified, "abc", "session_start").unwrap();
        append_compaction_entry(tmp.path(), &sample_compaction_detail("s1", 30, 5)).unwrap();
        append_audit_entry(tmp.path(), AuditAction::Verified, "def", "session_start").unwrap();
        append_compaction_entry(tmp.path(), &sample_compaction_detail("s2", 40, 5)).unwrap();

        // Full chain should be intact
        let broken = verify_audit_chain(tmp.path()).unwrap();
        assert!(broken.is_empty(), "Chain should be intact: {:?}", broken);

        // Should have 4 total entries, 2 compaction
        let all = read_audit_log(tmp.path()).unwrap();
        assert_eq!(all.len(), 4);

        let compactions = read_compaction_entries(tmp.path()).unwrap();
        assert_eq!(compactions.len(), 2);
    }

    #[test]
    fn compaction_tamper_detection() {
        let tmp = tempfile::tempdir().unwrap();

        append_compaction_entry(tmp.path(), &sample_compaction_detail("s1", 20, 5)).unwrap();
        append_compaction_entry(tmp.path(), &sample_compaction_detail("s2", 30, 5)).unwrap();
        append_compaction_entry(tmp.path(), &sample_compaction_detail("s3", 40, 5)).unwrap();

        // Tamper with the middle line by replacing a unique token in the source field
        let path = audit_file_path(tmp.path());
        let content = fs::read_to_string(&path).unwrap();
        let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();

        // Replace "session:s2" with "session:s2-hacked" in the second line
        lines[1] = lines[1].replace("session:s2", "session:s2-hacked");
        fs::write(&path, lines.join("\n") + "\n").unwrap();

        let broken = verify_audit_chain(tmp.path()).unwrap();
        assert!(!broken.is_empty(), "Should detect tampered entry");
        assert!(
            broken.contains(&2),
            "Entry after tampered line should break"
        );
    }

    #[test]
    fn compaction_empty_log() {
        let tmp = tempfile::tempdir().unwrap();

        let entries = read_compaction_entries(tmp.path()).unwrap();
        assert!(entries.is_empty());

        let stats = compaction_stats(tmp.path()).unwrap();
        assert_eq!(stats.total_events, 0);
        assert_eq!(stats.total_messages_compacted, 0);
        assert_eq!(stats.total_tokens_saved, 0);
        assert!(stats.last_compaction.is_none());
    }

    #[test]
    fn compaction_stats_computation() {
        let tmp = tempfile::tempdir().unwrap();

        // 3 compaction events: 20->5, 30->5, 40->5
        append_compaction_entry(tmp.path(), &sample_compaction_detail("s1", 20, 5)).unwrap();
        append_compaction_entry(tmp.path(), &sample_compaction_detail("s2", 30, 5)).unwrap();
        append_compaction_entry(tmp.path(), &sample_compaction_detail("s3", 40, 5)).unwrap();

        let stats = compaction_stats(tmp.path()).unwrap();
        assert_eq!(stats.total_events, 3);
        // Messages: (20-5) + (30-5) + (40-5) = 15 + 25 + 35 = 75
        assert_eq!(stats.total_messages_compacted, 75);
        // Tokens: (2000-500) + (3000-500) + (4000-500) = 1500 + 2500 + 3500 = 7500
        assert_eq!(stats.total_tokens_saved, 7500);
        assert!(stats.last_compaction.is_some());
    }

    #[test]
    fn compaction_action_serializes_snake_case() {
        let json = serde_json::to_string(&AuditAction::Compaction).unwrap();
        assert_eq!(json, "\"compaction\"");

        let parsed: AuditAction = serde_json::from_str("\"compaction\"").unwrap();
        assert_eq!(parsed, AuditAction::Compaction);
    }
}
