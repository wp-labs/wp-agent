use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use wist_shared::paths::ACTIONS_DIR;

use super::initialize;

fn temp_root(name: &str) -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("warp-agentd-bootstrap-{name}-{suffix}"))
}

fn assert_dir(path: &Path) {
    assert!(path.is_dir(), "expected directory: {}", path.display());
}

#[test]
fn initialize_creates_the_full_runtime_directory_layout() {
    let root = temp_root("layout");
    let run_dir = root.join("run");
    let state_dir = root.join("state");
    let log_dir = root.join("log");

    initialize(&root, &run_dir, &state_dir, &log_dir).expect("initialize layout");

    assert_dir(&root);
    assert_dir(&run_dir);
    assert_dir(&run_dir.join(ACTIONS_DIR));
    assert_dir(&state_dir);
    assert_dir(&log_dir);
    assert_dir(&state_dir.join("running"));
    assert_dir(&state_dir.join("reporting"));
    assert_dir(&state_dir.join("history"));
    assert_dir(&state_dir.join("logs").join("file_inputs"));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn initialize_is_idempotent_when_layout_already_exists() {
    let root = temp_root("idempotent");
    let run_dir = root.join("run");
    let state_dir = root.join("state");
    let log_dir = root.join("log");

    initialize(&root, &run_dir, &state_dir, &log_dir).expect("first initialize");
    initialize(&root, &run_dir, &state_dir, &log_dir).expect("second initialize");

    assert_dir(&state_dir.join("logs").join("file_inputs"));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn initialize_stops_with_an_error_when_a_directory_cannot_be_created() {
    let root = temp_root("blocked");
    let blocker = root.join("blocker");
    fs::create_dir_all(&root).expect("create root");
    fs::write(&blocker, "not a directory").expect("write blocker file");

    let blocked_root = blocker.join("under-a-file");
    let result = initialize(
        &blocked_root,
        &root.join("run"),
        &root.join("state"),
        &root.join("log"),
    );
    let _ = fs::remove_dir_all(&root);

    assert!(
        result.is_err(),
        "expected failure under a regular file path"
    );
}
