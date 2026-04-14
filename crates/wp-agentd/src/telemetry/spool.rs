//! Durable local spool for structured telemetry records.

use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

use wp_agent_contracts::telemetry_record::TelemetryRecordContract;
use wp_agent_shared::fs::ensure_parent;

use crate::telemetry::warp_parse::RecordSink;

pub fn append_records(path: &Path, records: &[TelemetryRecordContract]) -> io::Result<()> {
    if records.is_empty() {
        return Ok(());
    }

    ensure_parent(path)?;
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    for record in records {
        serde_json::to_writer(&mut file, record).map_err(io::Error::other)?;
        file.write_all(b"\n")?;
    }
    file.sync_all()?;
    Ok(())
}

#[cfg(test)]
pub fn load_records(path: &Path) -> io::Result<Vec<TelemetryRecordContract>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let record =
            serde_json::from_str::<TelemetryRecordContract>(&line).map_err(io::Error::other)?;
        records.push(record);
    }
    Ok(records)
}

pub fn has_records(path: &Path) -> io::Result<bool> {
    if !path.exists() {
        return Ok(false);
    }
    Ok(fs::metadata(path)?.len() > 0)
}

pub fn replay_records<S: RecordSink>(
    path: &Path,
    sink: &mut S,
    batch_size: usize,
) -> io::Result<usize> {
    if !path.exists() {
        return Ok(0);
    }

    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);
    let mut replayed = 0usize;
    let mut batch = Vec::with_capacity(batch_size.max(1));
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let record =
            serde_json::from_str::<TelemetryRecordContract>(&line).map_err(io::Error::other)?;
        batch.push(record);
        if batch.len() >= batch.capacity() {
            sink.write_records(&batch)?;
            replayed += batch.len();
            batch.clear();
        }
    }
    if !batch.is_empty() {
        sink.write_records(&batch)?;
        replayed += batch.len();
    }
    clear(path)?;
    Ok(replayed)
}

pub fn clear(path: &Path) -> io::Result<()> {
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{append_records, clear, has_records, load_records, replay_records};
    use crate::telemetry::warp_parse::RecordSink;
    use wp_agent_contracts::telemetry_record::TelemetryRecordContract;

    fn temp_file(name: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("duration")
            .as_nanos();
        std::env::temp_dir().join(format!("wp-agentd-spool-{name}-{suffix}.ndjson"))
    }

    fn record(body: &str) -> TelemetryRecordContract {
        TelemetryRecordContract::new_log(
            "2026-04-13T00:00:00Z".to_string(),
            "input-a".to_string(),
            "/tmp/app.log".to_string(),
            body.to_string(),
            0,
            body.len() as u64,
        )
    }

    #[test]
    fn append_and_load_round_trip_records() {
        let path = temp_file("round-trip");
        append_records(&path, &[record("a"), record("b")]).expect("append");

        let loaded = load_records(&path).expect("load");

        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].body, "a");
        assert_eq!(loaded[1].body, "b");
        fs::remove_file(path).ok();
    }

    #[test]
    fn clear_removes_spool_file() {
        let path = temp_file("clear");
        append_records(&path, &[record("a")]).expect("append");

        clear(&path).expect("clear");

        assert!(!path.exists());
    }

    #[derive(Default)]
    struct TestSink {
        records: Vec<TelemetryRecordContract>,
        fail_after_batches: Option<usize>,
        batches: usize,
    }

    impl RecordSink for TestSink {
        fn write_records(&mut self, records: &[TelemetryRecordContract]) -> io::Result<()> {
            self.batches += 1;
            if self
                .fail_after_batches
                .is_some_and(|limit| self.batches > limit)
            {
                return Err(io::Error::other("sink unavailable"));
            }
            self.records.extend_from_slice(records);
            Ok(())
        }
    }

    #[test]
    fn replay_records_streams_batches_and_clears_spool_on_success() {
        let path = temp_file("replay");
        append_records(&path, &[record("a"), record("b"), record("c")]).expect("append");
        let mut sink = TestSink::default();

        let replayed = replay_records(&path, &mut sink, 2).expect("replay");

        assert_eq!(replayed, 3);
        assert_eq!(sink.records.len(), 3);
        assert!(!has_records(&path).expect("spool presence"));
    }
}
