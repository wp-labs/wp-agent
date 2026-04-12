//! Result writer helpers.

use std::path::Path;

pub fn write(workdir: &Path) {
    eprintln!("write result in {}", workdir.display());
}
