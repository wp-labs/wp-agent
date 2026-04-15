use std::fs;
use std::io::Read;
use std::net::TcpListener;
use std::thread;
use std::time::Duration;

use warp_insight_contracts::agent_config::LogFileInputSection;
use warp_insight_contracts::telemetry_record::TelemetryRecordContract;
use warp_insight_shared::fs::read_json;
use warp_insightd::bootstrap;
use warp_insightd::daemon;

use super::common::{
    TestLogCheckpointState, standalone_config_with_file_input, standalone_config_with_file_inputs,
    standalone_config_with_tcp_file_input, temp_dir, test_exec_bin,
};

fn bind_tcp_listener(addr: &str) -> Option<TcpListener> {
    match TcpListener::bind(addr) {
        Ok(listener) => Some(listener),
        Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => None,
        Err(err) => panic!("bind tcp listener: {err}"),
    }
}

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
    let records: Vec<warp_insight_contracts::telemetry_record::TelemetryRecordContract> = output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("parse telemetry record"))
        .collect();
    let checkpoint_path = warp_insightd::state_store::log_checkpoints::path_for(&state_dir, "app");
    let checkpoint: TestLogCheckpointState = read_json(&checkpoint_path).expect("read checkpoint");

    assert_eq!(
        snapshot.state,
        warp_insightd::self_observability::HealthState::Active
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
    let records: Vec<warp_insight_contracts::telemetry_record::TelemetryRecordContract> = output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("parse telemetry record"))
        .collect();
    let checkpoint_path = warp_insightd::state_store::log_checkpoints::path_for(&state_dir, "good");
    let checkpoint: TestLogCheckpointState = read_json(&checkpoint_path).expect("read checkpoint");

    assert_eq!(
        snapshot.state,
        warp_insightd::self_observability::HealthState::Active
    );
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].body, "good\n");
    assert_eq!(checkpoint.files.len(), 1);
    assert!(!warp_insightd::state_store::log_checkpoints::path_for(&state_dir, "bad").exists());
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
        warp_insightd::self_observability::HealthState::Active
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
        warp_insightd::self_observability::HealthState::Active
    );
    assert!(!root.join("log").join("warp-parse-records.ndjson").exists());
    assert!(!warp_insightd::state_store::log_checkpoints::path_for(&state_dir, "missing").exists());
}

#[cfg(unix)]
#[test]
fn daemon_run_once_replays_existing_spool_even_when_source_file_is_missing() {
    let root = temp_dir("daemon-file-input-missing-replay");
    let run_dir = root.join("run");
    let state_dir = root.join("state");
    let log_dir = root.join("log");
    let missing_input = root.join("missing.log");
    bootstrap::initialize(&root, &run_dir, &state_dir, &log_dir).expect("bootstrap");

    let spool_path = root
        .join("state")
        .join("spool")
        .join("logs")
        .join("missing.ndjson");
    fs::create_dir_all(spool_path.parent().expect("spool dir")).expect("create spool dir");
    let first = serde_json::to_string(&TelemetryRecordContract::new_log(
        "2026-04-14T00:00:00Z".to_string(),
        "missing".to_string(),
        missing_input.display().to_string(),
        "first\n".to_string(),
        0,
        6,
    ))
    .expect("encode first");
    let second = serde_json::to_string(&TelemetryRecordContract::new_log(
        "2026-04-14T00:00:01Z".to_string(),
        "missing".to_string(),
        missing_input.display().to_string(),
        "second\n".to_string(),
        6,
        13,
    ))
    .expect("encode second");
    fs::write(&spool_path, format!("{first}\n{second}\n")).expect("write spool");

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

    let output_path = root.join("log").join("warp-parse-records.ndjson");
    let output = fs::read_to_string(&output_path).expect("read output");
    let records: Vec<warp_insight_contracts::telemetry_record::TelemetryRecordContract> = output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("parse telemetry record"))
        .collect();

    assert_eq!(
        snapshot.state,
        warp_insightd::self_observability::HealthState::Active
    );
    assert_eq!(records.len(), 2);
    assert_eq!(records[0].body, "first\n");
    assert_eq!(records[1].body, "second\n");
    assert!(!spool_path.exists());
}

#[cfg(unix)]
#[test]
fn daemon_run_once_sends_raw_log_lines_to_tcp_output() {
    let root = temp_dir("daemon-file-input-tcp");
    let run_dir = root.join("run");
    let state_dir = root.join("state");
    let log_dir = root.join("log");
    let input_path = root.join("app.log");
    bootstrap::initialize(&root, &run_dir, &state_dir, &log_dir).expect("bootstrap");
    fs::write(&input_path, "alpha\nbeta\n").expect("write input log");

    let Some(listener) = bind_tcp_listener("127.0.0.1:0") else {
        return;
    };
    let port = listener.local_addr().expect("listener addr").port();
    let server = thread::spawn(move || {
        let (mut socket, _) = listener.accept().expect("accept");
        socket
            .set_read_timeout(Some(Duration::from_secs(2)))
            .expect("set timeout");
        let mut buf = Vec::new();
        let mut chunk = [0u8; 128];
        loop {
            match socket.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => buf.extend_from_slice(&chunk[..n]),
                Err(err)
                    if matches!(
                        err.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) =>
                {
                    break;
                }
                Err(err) => panic!("read tcp payload: {err}"),
            }
        }
        String::from_utf8(buf).expect("utf8 payload")
    });

    let config =
        standalone_config_with_tcp_file_input(&root, &input_path, "127.0.0.1", port, "line");
    let snapshot = daemon::run_once(&daemon::DaemonLoop {
        config: &config,
        exec_bin: &test_exec_bin(&root),
    })
    .expect("daemon run once");

    let payload = server.join().expect("join server");
    let checkpoint_path = warp_insightd::state_store::log_checkpoints::path_for(&state_dir, "app");
    let checkpoint: TestLogCheckpointState = read_json(&checkpoint_path).expect("read checkpoint");

    assert_eq!(
        snapshot.state,
        warp_insightd::self_observability::HealthState::Active
    );
    assert_eq!(payload, "alpha\nbeta\n");
    assert_eq!(checkpoint.files.len(), 1);
    assert_eq!(
        checkpoint.files[0].checkpoint_offset,
        "alpha\nbeta\n".len() as u64
    );
    assert!(!root.join("log").join("warp-parse-records.ndjson").exists());
}

#[cfg(unix)]
#[test]
fn daemon_run_once_replays_spool_when_tcp_output_recovers() {
    let root = temp_dir("daemon-file-input-tcp-replay");
    let run_dir = root.join("run");
    let state_dir = root.join("state");
    let log_dir = root.join("log");
    let input_path = root.join("app.log");
    bootstrap::initialize(&root, &run_dir, &state_dir, &log_dir).expect("bootstrap");
    fs::write(&input_path, "first\nsecond\n").expect("write input log");

    let Some(reserved) = bind_tcp_listener("127.0.0.1:0") else {
        return;
    };
    let port = reserved.local_addr().expect("listener addr").port();
    drop(reserved);

    let failing_config =
        standalone_config_with_tcp_file_input(&root, &input_path, "127.0.0.1", port, "line");
    let first_snapshot = daemon::run_once(&daemon::DaemonLoop {
        config: &failing_config,
        exec_bin: &test_exec_bin(&root),
    })
    .expect("first daemon run once");
    let spool_path = root
        .join("state")
        .join("spool")
        .join("logs")
        .join("app.ndjson");
    let spooled = fs::read_to_string(&spool_path).expect("read spool");

    assert_eq!(
        first_snapshot.state,
        warp_insightd::self_observability::HealthState::Active
    );
    assert!(spooled.contains("\"body\":\"first\\n\""));
    assert!(spooled.contains("\"body\":\"second\\n\""));

    fs::write(&input_path, "first\nsecond\nthird\n").expect("append third line");
    let Some(listener) = bind_tcp_listener(&format!("127.0.0.1:{port}")) else {
        return;
    };
    let server = thread::spawn(move || {
        let (mut socket, _) = listener.accept().expect("accept");
        socket
            .set_read_timeout(Some(Duration::from_secs(2)))
            .expect("set timeout");
        let mut buf = Vec::new();
        let mut chunk = [0u8; 128];
        loop {
            match socket.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => buf.extend_from_slice(&chunk[..n]),
                Err(err)
                    if matches!(
                        err.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) =>
                {
                    break;
                }
                Err(err) => panic!("read tcp payload: {err}"),
            }
        }
        String::from_utf8(buf).expect("utf8 payload")
    });

    let recovered_config =
        standalone_config_with_tcp_file_input(&root, &input_path, "127.0.0.1", port, "line");
    let second_snapshot = daemon::run_once(&daemon::DaemonLoop {
        config: &recovered_config,
        exec_bin: &test_exec_bin(&root),
    })
    .expect("second daemon run once");
    let payload = server.join().expect("join server");
    let checkpoint_path = warp_insightd::state_store::log_checkpoints::path_for(&state_dir, "app");
    let checkpoint: TestLogCheckpointState = read_json(&checkpoint_path).expect("read checkpoint");

    assert_eq!(
        second_snapshot.state,
        warp_insightd::self_observability::HealthState::Active
    );
    assert_eq!(payload, "first\nsecond\nthird\n");
    assert!(!spool_path.exists());
    assert_eq!(checkpoint.files.len(), 1);
    assert_eq!(
        checkpoint.files[0].checkpoint_offset,
        "first\nsecond\nthird\n".len() as u64
    );
}
