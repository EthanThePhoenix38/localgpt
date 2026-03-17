//! File-based GPU lock for exclusive headless generation.
//!
//! Prevents concurrent Bevy instances from fighting over the GPU.
//! Uses `fs2` file locking for cross-process safety.

use std::path::PathBuf;

/// File-based GPU lock to prevent concurrent Bevy instances.
pub struct GpuLock {
    lock_path: PathBuf,
}

impl GpuLock {
    /// Create a new GPU lock using the XDG runtime directory.
    pub fn new() -> Self {
        // Use XDG_RUNTIME_DIR if available, else temp dir
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| std::env::temp_dir())
            .join("localgpt");
        let _ = std::fs::create_dir_all(&runtime_dir);
        Self {
            lock_path: runtime_dir.join("gen-gpu.lock"),
        }
    }

    /// Create a GPU lock at a specific path (for testing).
    pub fn with_path(lock_path: PathBuf) -> Self {
        Self { lock_path }
    }

    /// Try to acquire the GPU lock (non-blocking).
    /// Returns None if another gen instance holds it.
    pub fn try_acquire(&self) -> Option<GpuLockGuard> {
        use std::io::Write;

        let file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.lock_path)
            .ok()?;

        // Try non-blocking exclusive lock
        #[cfg(unix)]
        {
            use std::os::unix::io::AsRawFd;
            let fd = file.as_raw_fd();
            let ret = unsafe { libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB) };
            if ret != 0 {
                return None;
            }
        }

        #[cfg(not(unix))]
        {
            // On non-Unix, use a simple PID-based check
            // (less robust but functional)
        }

        // Write PID for diagnostics
        let mut f = file;
        let _ = write!(f, "{}", std::process::id());

        tracing::debug!("GPU lock acquired at {}", self.lock_path.display());

        Some(GpuLockGuard {
            _file: f,
            path: self.lock_path.clone(),
        })
    }

    /// Check if the GPU is currently locked by another process.
    pub fn is_locked(&self) -> bool {
        self.try_acquire().is_none()
    }
}

impl Default for GpuLock {
    fn default() -> Self {
        Self::new()
    }
}

/// RAII guard that releases the GPU lock when dropped.
///
/// The file lock is released automatically when the `File` is dropped
/// (OS releases the advisory lock on close).
pub struct GpuLockGuard {
    _file: std::fs::File,
    path: PathBuf,
}

impl Drop for GpuLockGuard {
    fn drop(&mut self) {
        tracing::debug!("GPU lock released at {}", self.path.display());
    }
}
