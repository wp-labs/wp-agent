use std::io;
use std::path::Path;

use wp_agent_contracts::telemetry_record::TelemetryRecordContract;

use crate::telemetry::buffer::TelemetryBuffer;
use crate::telemetry::spool;
use crate::telemetry::warp_parse::RecordSink;

use super::state_support::DeliveryOutcome;

pub(super) async fn deliver_records<S: RecordSink>(
    sink: &mut S,
    spool_path: &Path,
    in_memory_budget_bytes: usize,
    records: Vec<TelemetryRecordContract>,
) -> io::Result<DeliveryOutcome> {
    let records_processed = records.len();
    let mut emitted_directly = 0usize;
    let mut spooled = 0usize;
    let mut buffer = TelemetryBuffer::new(in_memory_budget_bytes);
    let staged = buffer.stage_all(records);

    if spool::has_records_async(spool_path).await? {
        let mut to_spool = staged.staged;
        to_spool.extend(staged.overflowed);
        spooled = to_spool.len();
        spool::append_records_async(spool_path, &to_spool).await?;
    } else {
        if !staged.overflowed.is_empty() {
            spooled += staged.overflowed.len();
            spool::append_records_async(spool_path, &staged.overflowed).await?;
        }
        if !staged.staged.is_empty() {
            match sink.write_records(&staged.staged).await {
                Ok(()) => {
                    emitted_directly = staged.staged.len();
                }
                Err(_) => {
                    spooled += staged.staged.len();
                    spool::append_records_async(spool_path, &staged.staged).await?;
                }
            }
        }
    }

    Ok(DeliveryOutcome {
        records_processed,
        emitted_directly,
        spooled,
    })
}

pub(super) async fn replay_spool_if_present<S: RecordSink>(
    sink: &mut S,
    spool_path: &Path,
    batch_size: usize,
) -> io::Result<usize> {
    if !spool::has_records_async(spool_path).await? {
        return Ok(0);
    }

    spool::replay_records_async(spool_path, sink, batch_size).await
}
