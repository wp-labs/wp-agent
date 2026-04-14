use std::fs;

use super::{
    FileInputProcessor, config, log_checkpoints, read_json, read_output_records, temp_dir,
};
use crate::telemetry::warp_parse::FileRecordSink;

#[test]
fn copytruncate_with_same_prefix_restarts_from_zero_when_checkpoint_probe_changes() {
    let root = temp_dir("copytruncate-same-prefix");
    let source_path = root.join("app.log");
    let output_path = root.join("log").join("records.ndjson");
    let prefix = "1234567890abcdef1234567890abcdef";
    let original = format!("{prefix}\nold-tail\n");
    let rewritten = format!("{prefix}\nnew-tail\nmore\n");
    fs::create_dir_all(root.join("state")).expect("create state");
    fs::create_dir_all(root.join("log")).expect("create log");
    fs::write(&source_path, &original).expect("write log");

    let mut first = FileInputProcessor::new(
        config(&root, &source_path),
        FileRecordSink::new(output_path.clone()),
    );
    let first_outcome = first.process_once().expect("first process");
    assert_eq!(first_outcome.records_processed, 2);

    fs::write(&source_path, &rewritten).expect("rewrite log");
    let mut second = FileInputProcessor::new(
        config(&root, &source_path),
        FileRecordSink::new(output_path.clone()),
    );
    let second_outcome = second.process_once().expect("second process");
    let records = read_output_records(&output_path);

    assert!(second_outcome.truncated);
    assert_eq!(second_outcome.records_processed, 3);
    assert_eq!(records.len(), 5);
    assert_eq!(records[2].body, format!("{prefix}\n"));
    assert_eq!(records[3].body, "new-tail\n");
    assert_eq!(records[4].body, "more\n");
}

#[test]
fn rotate_keeps_draining_old_file_tail_and_tracks_new_file_separately() {
    let root = temp_dir("rotate");
    let source_path = root.join("app.log");
    let rotated_path = root.join("app.log.1");
    let output_path = root.join("log").join("records.ndjson");
    fs::create_dir_all(root.join("state")).expect("create state");
    fs::create_dir_all(root.join("log")).expect("create log");
    fs::write(&source_path, "first\n").expect("write first log");

    let mut first = FileInputProcessor::new(
        config(&root, &source_path),
        FileRecordSink::new(output_path.clone()),
    );
    first.process_once().expect("first process");

    fs::rename(&source_path, &rotated_path).expect("rotate file");
    fs::write(&rotated_path, "first\nold-tail\n").expect("append old tail");
    fs::write(&source_path, "second\n").expect("write new active log");
    let mut second = FileInputProcessor::new(
        config(&root, &source_path),
        FileRecordSink::new(output_path.clone()),
    );
    let outcome = second.process_once().expect("second process");
    let checkpoint_path = log_checkpoints::path_for(&root.join("state"), "input-app");
    let state: crate::state_store::log_checkpoint_state::LogCheckpointState =
        read_json(&checkpoint_path).expect("read checkpoint");
    let records = read_output_records(&output_path);
    let source_path_string = source_path.display().to_string();
    let rotated_path_string = rotated_path.display().to_string();
    let active = state
        .files
        .iter()
        .find(|entry| entry.path == source_path_string)
        .expect("active checkpoint");
    let rotated = state
        .files
        .iter()
        .find(|entry| entry.path == rotated_path_string)
        .expect("rotated checkpoint");

    assert!(outcome.rotated);
    assert!(!outcome.truncated);
    assert_eq!(outcome.records_processed, 2);
    assert_eq!(records.len(), 3);
    assert_eq!(records[0].body, "first\n");
    assert_eq!(records[1].body, "old-tail\n");
    assert_eq!(records[2].body, "second\n");
    assert_eq!(state.files.len(), 2);
    assert_eq!(active.checkpoint_offset, "second\n".len() as u64);
    assert_eq!(rotated.checkpoint_offset, "first\nold-tail\n".len() as u64);
    assert_eq!(
        active.rotated_from_path.as_deref(),
        Some(source_path_string.as_str())
    );
    assert!(!active.file_id.contains("path:"));
    assert!(!rotated.file_id.contains("path:"));
    assert_ne!(active.file_id, rotated.file_id);
}
