use std::fs;

use wp_agent_contracts::agent_config::LogFileInputSection;
use wp_agent_shared::fs::read_json;
use wp_agentd::bootstrap;
use wp_agentd::daemon;

use super::common::{
    TestLogCheckpointState, standalone_config_with_file_input, standalone_config_with_file_inputs,
    temp_dir, test_exec_bin,
};

#[cfg(unix)]
#[test]
fn daemon_run_once_processes_configured_file_input() {
    let root = temp_dir("daemon-file-input");
    let run_dir = root.join("run");
    let state_dir = root.join("state");
    let log_dir = root.join("log");
    let input_path = root.join("app.log");
    bootstrap::initialize(&root, &run_dir, &state_dir, &log_dir).expect("bootstrap");
    fs::write(&input_path, "first\nsecond\n").expect("write input log");

    let config = standalone_config_with_file_input(&root, &input_path);
    let snapshot = daemon::run_once(&daemon::DaemonLoop {
        config: &config,
        exec_bin: &test_exec_bin(&root),
    })
    .expect("daemon run once");

    let output_path = root.join("log").join("warp-parse-records.ndjson");
    let output = fs::read_to_string(&output_path).expect("read output");
    let records: Vec<wp_agent_contracts::telemetry_record::TelemetryRecordContract> = output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("parse telemetry record"))
        .collect();
    let checkpoint_path = wp_agentd::state_store::log_checkpoints::path_for(&state_dir, "app");
    let checkpoint: TestLogCheckpointState = read_json(&checkpoint_path).expect("read checkpoint");

    assert_eq!(
        snapshot.state,
        wp_agentd::self_observability::HealthState::Active
    );
    assert_eq!(records.len(), 2);
    assert_eq!(records[0].body, "first\n");
    assert_eq!(records[1].body, "second\n");
    assert_eq!(checkpoint.files.len(), 1);
    assert_eq!(
        checkpoint.files[0].checkpoint_offset,
        "first\nsecond\n".len() as u64
    );
}

#[cfg(unix)]
#[test]
fn daemon_run_once_continues_when_one_file_input_fails() {
    let root = temp_dir("daemon-file-input-error-isolated");
    let run_dir = root.join("run");
    let state_dir = root.join("state");
    let log_dir = root.join("log");
    let good_input = root.join("good.log");
    let bad_input = root.join("bad-dir");
    bootstrap::initialize(&root, &run_dir, &state_dir, &log_dir).expect("bootstrap");
    fs::write(&good_input, "good\n").expect("write good input");
    fs::create_dir_all(&bad_input).expect("create bad input dir");

    let config = standalone_config_with_file_inputs(
        &root,
        vec![
            LogFileInputSection {
                input_id: "bad".to_string(),
                path: bad_input.display().to_string(),
                startup_position: "head".to_string(),
                multiline_mode: "none".to_string(),
            },
            LogFileInputSection {
                input_id: "good".to_string(),
                path: good_input.display().to_string(),
                startup_position: "head".to_string(),
                multiline_mode: "none".to_string(),
            },
        ],
    );
    let snapshot = daemon::run_once(&daemon::DaemonLoop {
        config: &config,
        exec_bin: &test_exec_bin(&root),
    })
    .expect("daemon run once");

    let output_path = root.join("log").join("warp-parse-records.ndjson");
    let output = fs::read_to_string(&output_path).expect("read output");
    let records: Vec<wp_agent_contracts::telemetry_record::TelemetryRecordContract> = output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("parse telemetry record"))
        .collect();
    let checkpoint_path = wp_agentd::state_store::log_checkpoints::path_for(&state_dir, "good");
    let checkpoint: TestLogCheckpointState = read_json(&checkpoint_path).expect("read checkpoint");

    assert_eq!(
        snapshot.state,
        wp_agentd::self_observability::HealthState::Active
    );
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].body, "good\n");
    assert_eq!(checkpoint.files.len(), 1);
    assert!(!wp_agentd::state_store::log_checkpoints::path_for(&state_dir, "bad").exists());
}

#[cfg(unix)]
#[test]
fn daemon_run_once_marks_active_when_only_file_input_fails() {
    let root = temp_dir("daemon-file-input-only-error");
    let run_dir = root.join("run");
    let state_dir = root.join("state");
    let log_dir = root.join("log");
    let bad_input = root.join("bad-dir");
    bootstrap::initialize(&root, &run_dir, &state_dir, &log_dir).expect("bootstrap");
    fs::create_dir_all(&bad_input).expect("create bad input dir");

    let config = standalone_config_with_file_inputs(
        &root,
        vec![LogFileInputSection {
            input_id: "bad".to_string(),
            path: bad_input.display().to_string(),
            startup_position: "head".to_string(),
            multiline_mode: "none".to_string(),
        }],
    );
    let snapshot = daemon::run_once(&daemon::DaemonLoop {
        config: &config,
        exec_bin: &test_exec_bin(&root),
    })
    .expect("daemon run once");

    assert_eq!(
        snapshot.state,
        wp_agentd::self_observability::HealthState::Active
    );
    assert!(!root.join("log").join("warp-parse-records.ndjson").exists());
}

#[cfg(unix)]
#[test]
fn daemon_run_once_marks_active_when_configured_file_is_missing() {
    let root = temp_dir("daemon-file-input-missing");
    let run_dir = root.join("run");
    let state_dir = root.join("state");
    let log_dir = root.join("log");
    let missing_input = root.join("missing.log");
    bootstrap::initialize(&root, &run_dir, &state_dir, &log_dir).expect("bootstrap");

    let config = standalone_config_with_file_inputs(
        &root,
        vec![LogFileInputSection {
            input_id: "missing".to_string(),
            path: missing_input.display().to_string(),
            startup_position: "head".to_string(),
            multiline_mode: "none".to_string(),
        }],
    );
    let snapshot = daemon::run_once(&daemon::DaemonLoop {
        config: &config,
        exec_bin: &test_exec_bin(&root),
    })
    .expect("daemon run once");

    assert_eq!(
        snapshot.state,
        wp_agentd::self_observability::HealthState::Active
    );
    assert!(!root.join("log").join("warp-parse-records.ndjson").exists());
    assert!(!wp_agentd::state_store::log_checkpoints::path_for(&state_dir, "missing").exists());
}
