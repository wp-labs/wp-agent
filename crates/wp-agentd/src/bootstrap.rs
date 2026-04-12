//! Bootstrap entrypoints for `wp-agentd`.

use std::fs;
use std::io;
use std::path::Path;

pub fn initialize(root_dir: &Path, run_dir: &Path, state_dir: &Path, log_dir: &Path) -> io::Result<()> {
    fs::create_dir_all(root_dir)?;
    fs::create_dir_all(run_dir)?;
    fs::create_dir_all(state_dir)?;
    fs::create_dir_all(log_dir)?;
    fs::create_dir_all(state_dir.join("running"))?;
    fs::create_dir_all(state_dir.join("reporting"))?;
    fs::create_dir_all(state_dir.join("history"))?;
    fs::create_dir_all(state_dir.join("logs").join("file_inputs"))?;
    Ok(())
}
