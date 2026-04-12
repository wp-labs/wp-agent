//! Runtime entrypoints.

use std::path::Path;

pub fn execute(workdir: &Path) {
    eprintln!("execute plan in {}", workdir.display());
}
