//! File-based GPU lock for exclusive headless generation.
//!
//! Prevents concurrent Bevy instances from fighting over the GPU.
//! Uses `flock` on Unix and `LockFileEx` on Windows for cross-process safety.

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
        if !try_lock_exclusive(&file) {
            return None;
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

/// Try to acquire an exclusive non-blocking lock on a file.
#[cfg(unix)]
fn try_lock_exclusive(file: &std::fs::File) -> bool {
    use std::os::unix::io::AsRawFd;
    let fd = file.as_raw_fd();
    // SAFETY: `fd` is a valid file descriptor obtained from `file` which is borrowed
    // for the duration of this call. flock() with LOCK_EX|LOCK_NB is non-blocking
    // and only operates on the given fd with no memory safety concerns.
    let ret = unsafe { libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB) };
    ret == 0
}

/// Try to acquire an exclusive non-blocking lock on a file (Windows).
#[cfg(windows)]
fn try_lock_exclusive(file: &std::fs::File) -> bool {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Storage::FileSystem::{
        LOCKFILE_EXCLUSIVE_LOCK, LOCKFILE_FAIL_IMMEDIATELY, LockFileEx,
    };
    let handle = file.as_raw_handle();
    // SAFETY: zeroed OVERLAPPED is a valid initialization for this struct (all-zeros
    // is an accepted initial state). `handle` is valid for the lifetime of `file`.
    // LockFileEx with LOCKFILE_FAIL_IMMEDIATELY is non-blocking and well-defined.
    let mut overlapped = unsafe { std::mem::zeroed() };
    let result = unsafe {
        LockFileEx(
            handle,
            LOCKFILE_EXCLUSIVE_LOCK | LOCKFILE_FAIL_IMMEDIATELY,
            0,
            1,
            0,
            &mut overlapped,
        )
    };
    result != 0
}

/// Fallback for other platforms — always succeeds (no locking).
#[cfg(not(any(unix, windows)))]
fn try_lock_exclusive(_file: &std::fs::File) -> bool {
    tracing::warn!("GPU lock not supported on this platform — concurrent access not prevented");
    true
}
