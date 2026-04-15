//! Bootstrap entrypoints for `warp-insightd`.

use std::fs;
use std::io;
use std::path::Path;

use warp_insight_shared::paths::ACTIONS_DIR;

pub fn initialize(
    root_dir: &Path,
    run_dir: &Path,
    state_dir: &Path,
    log_dir: &Path,
) -> io::Result<()> {
    fs::create_dir_all(root_dir)?;
    fs::create_dir_all(run_dir)?;
    fs::create_dir_all(run_dir.join(ACTIONS_DIR))?;
    fs::create_dir_all(state_dir)?;
    fs::create_dir_all(log_dir)?;
    fs::create_dir_all(state_dir.join("running"))?;
    fs::create_dir_all(state_dir.join("reporting"))?;
    fs::create_dir_all(state_dir.join("history"))?;
    fs::create_dir_all(state_dir.join("logs").join("file_inputs"))?;
    Ok(())
}
