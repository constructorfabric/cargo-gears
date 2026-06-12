#![allow(clippy::expect_used)] // This is only for testing
use std::fs;
use tempfile::TempDir;

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
