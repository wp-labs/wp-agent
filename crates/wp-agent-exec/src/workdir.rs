//! Workdir protocol helpers.

use std::path::{Path, PathBuf};

pub fn open(base: &Path) -> PathBuf {
    base.to_path_buf()
}
