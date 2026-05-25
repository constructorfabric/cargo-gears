#![allow(clippy::expect_used)] // This is only for testing
use std::fs;
use std::sync::Mutex;
use tempfile::TempDir;

/// Shared mutex for tests that change the process-wide current directory.
///
/// Acquire this lock in any test that calls `std::env::set_current_dir` or
/// exercises code paths that do so (e.g. `with_current_dir`,
/// `with_current_dir_for_optional_path`, `parse_and_chdir`).
pub static CWD_MUTEX: Mutex<()> = Mutex::new(());

pub trait TempDirExt {
    fn write(&self, relative_path: &str, content: &str);
}

impl TempDirExt for TempDir {
    fn write(&self, relative_path: &str, content: &str) {
        let path = self.path().join(relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("failed to create parent dir");
        }
        fs::write(path, content).expect("failed to write test file");
    }
}
