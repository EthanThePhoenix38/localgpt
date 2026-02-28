//! Document loaders for extracting text from various file formats.
//!
//! Uses a shell-command approach inspired by aichat's document_loaders pattern.
//! Zero Rust PDF/DOCX dependencies, user-extensible via config.

use anyhow::{Context, Result, bail};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use std::time::Duration;
use tracing::debug;

/// Default timeout for document conversion commands (30 seconds)
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Default document loaders (shell commands)
fn default_loaders() -> HashMap<String, String> {
    let mut loaders = HashMap::new();
    // PDF via poppler-utils
    loaders.insert("pdf".to_string(), "pdftotext $1 -".to_string());
    // DOCX/EPUB/HTML via pandoc
    loaders.insert("docx".to_string(), "pandoc --to plain $1".to_string());
    loaders.insert("epub".to_string(), "pandoc --to plain $1".to_string());
    loaders.insert("html".to_string(), "pandoc --to plain $1".to_string());
    loaders.insert("htm".to_string(), "pandoc --to plain $1".to_string());
    loaders
}

/// Document loaders configuration
#[derive(Debug, Clone)]
pub struct DocumentLoaders {
    /// Map of file extension (lowercase) to shell command template
    /// $1 in the template is replaced with the file path
    loaders: HashMap<String, String>,
    /// Timeout for conversion commands
    timeout: Duration,
}

impl Default for DocumentLoaders {
    fn default() -> Self {
        Self {
            loaders: default_loaders(),
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
        }
    }
}

impl DocumentLoaders {
    /// Create a new DocumentLoaders with default loaders
    pub fn new() -> Self {
        Self::default()
    }

    /// Create DocumentLoaders with custom loaders merged with defaults
    pub fn with_custom(custom: &HashMap<String, String>) -> Self {
        let mut loaders = default_loaders();
        // Override/add custom loaders
        for (ext, cmd) in custom {
            loaders.insert(ext.to_lowercase(), cmd.clone());
        }
        Self {
            loaders,
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
        }
    }

    /// Set timeout for conversion commands
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Check if a loader exists for the given extension
    pub fn has_loader(&self, ext: &str) -> bool {
        self.loaders.contains_key(&ext.to_lowercase())
    }

    /// Get list of supported extensions
    pub fn supported_extensions(&self) -> Vec<&str> {
        let mut exts: Vec<&str> = self.loaders.keys().map(|s| s.as_str()).collect();
        exts.sort();
        exts
    }

    /// Extract text from a document file
    ///
    /// # Arguments
    /// * `path` - Path to the document file
    ///
    /// # Returns
    /// * `Ok(String)` - Extracted text content
    /// * `Err` - If no loader is configured or the command fails
    pub fn extract_text(&self, path: &Path) -> Result<String> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| anyhow::anyhow!("File has no extension: {}", path.display()))?;

        let ext_lower = ext.to_lowercase();
        let template = self
            .loaders
            .get(&ext_lower)
            .ok_or_else(|| anyhow::anyhow!("No loader configured for .{} files", ext_lower))?;

        // Sanitize file path to prevent command injection
        let sanitized_path = sanitize_path(path)?;

        // Replace $1 with the file path
        let cmd = template.replace("$1", &sanitized_path);

        debug!("Extracting text from {} via: {}", path.display(), cmd);

        // Execute via sh -c
        let output = Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .output()
            .context("Failed to execute document loader command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let hint = get_install_hint(&ext_lower);

            bail!(
                "Document loader failed for .{} file.\n\
                 Command: {}\n\
                 Exit code: {}\n\
                 Error: {}\n\
                 {}",
                ext_lower,
                cmd,
                output.status.code().unwrap_or(-1),
                stderr.trim(),
                hint
            );
        }

        let text = String::from_utf8(output.stdout)
            .context("Document loader produced non-UTF8 output")?;

        Ok(text)
    }

    /// Add or update a loader for a specific extension
    pub fn set_loader(&mut self, ext: &str, command: &str) {
        self.loaders.insert(ext.to_lowercase(), command.to_string());
    }

    /// Remove a loader for a specific extension
    pub fn remove_loader(&mut self, ext: &str) -> Option<String> {
        self.loaders.remove(&ext.to_lowercase())
    }
}

/// Sanitize a file path for use in shell commands
fn sanitize_path(path: &Path) -> Result<String> {
    let path_str = path.display().to_string();

    // Check for dangerous characters that could allow command injection
    let dangerous_chars = ['`', '$', '(', ')', ';', '|', '&', '<', '>', '\n', '\r'];

    for ch in dangerous_chars {
        if path_str.contains(ch) {
            bail!(
                "File path contains potentially dangerous character '{}': {}",
                ch,
                path_str
            );
        }
    }

    // Shell-escape the path using single quotes
    // Replace single quotes with '\'' (end quote, escaped quote, start quote)
    let escaped = path_str.replace('\'', "'\\''");
    Ok(format!("'{}'", escaped))
}

/// Get installation hints for common document tools
fn get_install_hint(ext: &str) -> &'static str {
    match ext {
        "pdf" => "Hint: Install poppler-utils (apt: poppler-utils, brew: poppler, dnf: poppler-utils)",
        "docx" | "epub" | "html" | "htm" => "Hint: Install pandoc (apt: pandoc, brew: pandoc, dnf: pandoc)",
        "xlsx" => "Hint: Install xlsx2csv (pip: xlsx2csv)",
        "pptx" => "Hint: Install LibreOffice (apt: libreoffice, brew: libreoffice)",
        _ => "Hint: Check the loader command in your config.toml",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_default_loaders() {
        let loaders = DocumentLoaders::new();

        assert!(loaders.has_loader("pdf"));
        assert!(loaders.has_loader("PDF")); // case insensitive
        assert!(loaders.has_loader("docx"));
        assert!(loaders.has_loader("epub"));
        assert!(loaders.has_loader("html"));

        assert!(!loaders.has_loader("xyz"));
    }

    #[test]
    fn test_custom_loaders() {
        let mut custom = HashMap::new();
        custom.insert("xyz".to_string(), "xyz-converter $1".to_string());

        let loaders = DocumentLoaders::with_custom(&custom);

        // Custom loader added
        assert!(loaders.has_loader("xyz"));

        // Default loaders preserved
        assert!(loaders.has_loader("pdf"));
    }

    #[test]
    fn test_custom_loader_overrides_default() {
        let mut custom = HashMap::new();
        custom.insert("pdf".to_string(), "my-custom-pdf-tool $1".to_string());

        let loaders = DocumentLoaders::with_custom(&custom);

        // Custom loader should override default
        assert!(loaders.loaders.get("pdf").unwrap().contains("my-custom-pdf-tool"));
    }

    #[test]
    fn test_sanitize_path_simple() {
        let path = Path::new("/home/user/document.pdf");
        let sanitized = sanitize_path(path).unwrap();
        assert_eq!(sanitized, "'/home/user/document.pdf'");
    }

    #[test]
    fn test_sanitize_path_with_spaces() {
        let path = Path::new("/home/user/My Document.pdf");
        let sanitized = sanitize_path(path).unwrap();
        assert_eq!(sanitized, "'/home/user/My Document.pdf'");
    }

    #[test]
    fn test_sanitize_path_with_single_quote() {
        let path = Path::new("/home/user/O'Reilly.pdf");
        let sanitized = sanitize_path(path).unwrap();
        assert_eq!(sanitized, "'/home/user/O'\\''Reilly.pdf'");
    }

    #[test]
    fn test_sanitize_path_rejects_command_injection() {
        let dangerous_paths = vec![
            "/home/user/$(whoami).pdf",
            "/home/user/`id`.pdf",
            "/home/user/;rm -rf /.pdf",
            "/home/user/|cat /etc/passwd.pdf",
        ];

        for path_str in dangerous_paths {
            let path = Path::new(path_str);
            assert!(sanitize_path(path).is_err(), "Should reject: {}", path_str);
        }
    }

    #[test]
    fn test_extract_text_no_loader() {
        let loaders = DocumentLoaders::new();
        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("test.xyz");
        fs::write(&file, "content").unwrap();

        let result = loaders.extract_text(&file);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No loader"));
    }

    #[test]
    fn test_extract_text_missing_file() {
        let loaders = DocumentLoaders::new();
        let result = loaders.extract_text(Path::new("/nonexistent/file.pdf"));

        // Should fail (file doesn't exist)
        assert!(result.is_err());
    }

    #[test]
    fn test_supported_extensions() {
        let loaders = DocumentLoaders::new();
        let exts = loaders.supported_extensions();

        assert!(exts.contains(&"docx"));
        assert!(exts.contains(&"epub"));
        assert!(exts.contains(&"html"));
        assert!(exts.contains(&"pdf"));
    }

    #[test]
    fn test_set_and_remove_loader() {
        let mut loaders = DocumentLoaders::new();

        // Add custom loader
        loaders.set_loader("xyz", "xyz-tool $1");
        assert!(loaders.has_loader("xyz"));

        // Remove it
        let removed = loaders.remove_loader("xyz");
        assert_eq!(removed, Some("xyz-tool $1".to_string()));
        assert!(!loaders.has_loader("xyz"));
    }
}
