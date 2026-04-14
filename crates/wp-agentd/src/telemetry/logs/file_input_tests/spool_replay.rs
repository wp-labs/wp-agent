use std::fs;
use std::io;

use super::{FileInputProcessor, TestSink, config, read_output_records, spool, temp_dir};
use crate::telemetry::warp_parse::FileRecordSink;

#[test]
fn sink_failure_spools_records_and_only_then_advances_checkpoint() {
    let root = temp_dir("spool");
    let source_path = root.join("app.log");
    fs::create_dir_all(root.join("state")).expect("create state");
    fs::write(&source_path, "first\nsecond\n").expect("write log");
    let mut processor = FileInputProcessor::new(
        config(&root, &source_path),
        TestSink {
            fail_writes: true,
            ..Default::default()
        },
    );

    let outcome = processor.process_once().expect("process");
    let spooled = spool::load_records(&root.join("state").join("spool").join("input-app.ndjson"))
        .expect("load spool");

    assert_eq!(outcome.emitted_directly, 0);
    assert_eq!(outcome.spooled, 2);
    assert_eq!(spooled.len(), 2);
    assert_eq!(outcome.checkpoint_offset, "first\nsecond\n".len() as u64);
}

#[test]
fn replays_spooled_records_after_sink_recovers_and_clears_spool() {
    let root = temp_dir("spool-replay");
    let source_path = root.join("app.log");
    let output_path = root.join("log").join("records.ndjson");
    let first_line = "abcdefghijklmnopqrstuvwxyz123456\n";
    let second_line = "ABCDEFGHIJKLMNOPQRSTUVWXYZ654321\n";
    fs::create_dir_all(root.join("state")).expect("create state");
    fs::create_dir_all(root.join("log")).expect("create log");
    fs::write(&source_path, format!("{first_line}{second_line}")).expect("write log");

    let mut failing = FileInputProcessor::new(
        config(&root, &source_path),
        TestSink {
            fail_writes: true,
            ..Default::default()
        },
    );
    let first = failing.process_once().expect("first process");
    assert_eq!(first.spooled, 2);

    fs::write(&source_path, format!("{first_line}{second_line}third\n")).expect("append third");
    let mut replay = FileInputProcessor::new(
        config(&root, &source_path),
        FileRecordSink::new(output_path.clone()),
    );
    let second = replay.process_once().expect("second process");
    let spooled = spool::load_records(&root.join("state").join("spool").join("input-app.ndjson"))
        .expect("load spool");
    let records = read_output_records(&output_path);

    assert_eq!(second.replayed_spool, 2);
    assert_eq!(second.records_processed, 1);
    assert!(spooled.is_empty());
    assert_eq!(records.len(), 3);
    assert_eq!(records[0].body, first_line);
    assert_eq!(records[1].body, second_line);
    assert_eq!(records[2].body, "third\n");
}

#[test]
fn replay_failure_is_reported_when_spool_exists_but_sink_is_still_unavailable() {
    let root = temp_dir("spool-replay-failure");
    let source_path = root.join("app.log");
    fs::create_dir_all(root.join("state")).expect("create state");
    fs::write(&source_path, "first\nsecond\n").expect("write log");

    let mut first = FileInputProcessor::new(
        config(&root, &source_path),
        TestSink {
            fail_writes: true,
            ..Default::default()
        },
    );
    first.process_once().expect("first process");

    let mut replay = FileInputProcessor::new(
        config(&root, &source_path),
        TestSink {
            fail_writes: true,
            ..Default::default()
        },
    );
    let err = replay.process_once().expect_err("replay should fail");

    assert_eq!(err.kind(), io::ErrorKind::Other);
    assert!(
        root.join("state")
            .join("spool")
            .join("input-app.ndjson")
            .exists()
    );
}
