use std::fs;

use super::{
    FileInputProcessor, config, log_checkpoints, read_json, read_output_records, temp_dir,
    write_json_atomic,
};
use crate::telemetry::logs::multiline::MultilineMode;
use crate::telemetry::warp_parse::FileRecordSink;

#[test]
fn indented_multiline_merges_stack_frames() {
    let root = temp_dir("multiline");
    let source_path = root.join("app.log");
    let output_path = root.join("log").join("records.ndjson");
    fs::create_dir_all(root.join("state")).expect("create state");
    fs::create_dir_all(root.join("log")).expect("create log");
    fs::write(&source_path, "ERROR first\n  frame1\nINFO next\n").expect("write log");
    let mut cfg = config(&root, &source_path);
    cfg.multiline_mode = MultilineMode::IndentedContinuation;
    let mut processor = FileInputProcessor::new(cfg, FileRecordSink::new(output_path.clone()));

    let outcome = processor.process_once().expect("process");
    let checkpoint_path = log_checkpoints::path_for(&root.join("state"), "input-app");
    let state: crate::state_store::log_checkpoint_state::LogCheckpointState =
        read_json(&checkpoint_path).expect("read checkpoint");

    assert_eq!(outcome.records_processed, 1);
    let records = read_output_records(&output_path);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].body, "ERROR first\n  frame1\n");
    assert_eq!(
        state
            .pending_multiline
            .as_ref()
            .map(|pending| pending.body.as_str()),
        Some("INFO next\n")
    );
}

#[test]
fn multiline_state_survives_across_ticks_and_flushes_on_idle() {
    let root = temp_dir("multiline-cross-tick");
    let source_path = root.join("app.log");
    let output_path = root.join("log").join("records.ndjson");
    fs::create_dir_all(root.join("state")).expect("create state");
    fs::create_dir_all(root.join("log")).expect("create log");
    fs::write(&source_path, "ERROR first\n").expect("write log");
    let mut cfg = config(&root, &source_path);
    cfg.multiline_mode = MultilineMode::IndentedContinuation;

    let mut first = FileInputProcessor::new(cfg.clone(), FileRecordSink::new(output_path.clone()));
    let first_outcome = first.process_once().expect("first process");
    let checkpoint_path = log_checkpoints::path_for(&root.join("state"), "input-app");
    let first_state: crate::state_store::log_checkpoint_state::LogCheckpointState =
        read_json(&checkpoint_path).expect("read first checkpoint");

    assert_eq!(first_outcome.records_processed, 0);
    assert_eq!(
        first_state
            .pending_multiline
            .as_ref()
            .map(|pending| pending.body.as_str()),
        Some("ERROR first\n")
    );

    fs::write(&source_path, "ERROR first\n  frame1\nINFO next\n").expect("append log");
    let mut second = FileInputProcessor::new(cfg.clone(), FileRecordSink::new(output_path.clone()));
    let second_outcome = second.process_once().expect("second process");
    let second_state: crate::state_store::log_checkpoint_state::LogCheckpointState =
        read_json(&checkpoint_path).expect("read second checkpoint");
    let second_records = read_output_records(&output_path);

    assert_eq!(second_outcome.records_processed, 1);
    assert_eq!(second_records.len(), 1);
    assert_eq!(second_records[0].body, "ERROR first\n  frame1\n");
    assert_eq!(
        second_state
            .pending_multiline
            .as_ref()
            .map(|pending| pending.body.as_str()),
        Some("INFO next\n")
    );

    let mut third = FileInputProcessor::new(cfg, FileRecordSink::new(output_path.clone()));
    let third_outcome = third.process_once().expect("third process");
    let third_state: crate::state_store::log_checkpoint_state::LogCheckpointState =
        read_json(&checkpoint_path).expect("read third checkpoint");
    let third_records = read_output_records(&output_path);

    assert_eq!(third_outcome.records_processed, 0);
    assert_eq!(third_records.len(), 1);
    assert_eq!(
        third_state
            .pending_multiline
            .as_ref()
            .map(|pending| pending.body.as_str()),
        Some("INFO next\n")
    );

    let mut aged_state = third_state.clone();
    aged_state
        .pending_multiline
        .as_mut()
        .expect("pending")
        .last_updated_at = "2000-01-01T00:00:00Z".to_string();
    write_json_atomic(&checkpoint_path, &aged_state).expect("age pending multiline");

    let mut flush_cfg = config(&root, &source_path);
    flush_cfg.multiline_mode = MultilineMode::IndentedContinuation;
    let mut fourth = FileInputProcessor::new(flush_cfg, FileRecordSink::new(output_path.clone()));
    let fourth_outcome = fourth.process_once().expect("fourth process");
    let fourth_state: crate::state_store::log_checkpoint_state::LogCheckpointState =
        read_json(&checkpoint_path).expect("read fourth checkpoint");
    let fourth_records = read_output_records(&output_path);

    assert_eq!(fourth_outcome.records_processed, 1);
    assert!(fourth_state.pending_multiline.is_none());
    assert_eq!(fourth_records.len(), 2);
    assert_eq!(fourth_records[1].body, "INFO next\n");
}

#[test]
fn rotate_keeps_pending_multiline_bound_to_old_file_until_tail_is_drained() {
    let root = temp_dir("rotate-multiline");
    let source_path = root.join("app.log");
    let rotated_path = root.join("app.log.1");
    let output_path = root.join("log").join("records.ndjson");
    fs::create_dir_all(root.join("state")).expect("create state");
    fs::create_dir_all(root.join("log")).expect("create log");
    fs::write(&source_path, "ERROR first\n").expect("write first log");

    let mut initial_cfg = config(&root, &source_path);
    initial_cfg.multiline_mode = MultilineMode::IndentedContinuation;
    let mut first = FileInputProcessor::new(
        initial_cfg.clone(),
        FileRecordSink::new(output_path.clone()),
    );
    let first_outcome = first.process_once().expect("first process");
    assert_eq!(first_outcome.records_processed, 0);

    fs::rename(&source_path, &rotated_path).expect("rotate file");
    fs::write(&rotated_path, "ERROR first\n  frame1\n").expect("append old tail");
    fs::write(&source_path, "INFO next\n").expect("write new active log");

    let mut second = FileInputProcessor::new(initial_cfg, FileRecordSink::new(output_path.clone()));
    let second_outcome = second.process_once().expect("second process");
    let records = read_output_records(&output_path);

    assert!(second_outcome.rotated);
    assert_eq!(second_outcome.records_processed, 1);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].body, "ERROR first\n  frame1\n");

    let checkpoint_path = log_checkpoints::path_for(&root.join("state"), "input-app");
    let state: crate::state_store::log_checkpoint_state::LogCheckpointState =
        read_json(&checkpoint_path).expect("read checkpoint");
    assert_eq!(
        state
            .pending_multiline
            .as_ref()
            .map(|pending| pending.body.as_str()),
        Some("INFO next\n")
    );
}
