use std::path::Path;

use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use warp_insight_contracts::telemetry_record::TelemetryRecordContract;

use crate::state_store::log_checkpoint_state::PendingMultilineState;
use crate::telemetry::logs::file_reader::RawFileLine;
use crate::telemetry::logs::multiline::{MultilineMode, flush_pending, fold_lines};
use crate::telemetry::logs::parser::parse_folded_lines;

const MULTILINE_IDLE_FLUSH_MS: i64 = 1000;

pub(super) fn records_from_read(
    records: &mut Vec<TelemetryRecordContract>,
    observed_at: &str,
    input_id: &str,
    source_path: &Path,
    multiline_mode: MultilineMode,
    lines: Vec<RawFileLine>,
    pending: Option<PendingMultilineState>,
) -> Option<PendingMultilineState> {
    let source_path = source_path.display().to_string();
    let folded = fold_lines(multiline_mode, &source_path, observed_at, lines, pending);
    records.extend(parse_folded_lines(
        observed_at,
        input_id,
        &source_path,
        folded.emitted,
    ));
    folded.pending
}

pub(super) fn records_from_pending(
    observed_at: &str,
    input_id: &str,
    pending: Option<PendingMultilineState>,
) -> Vec<TelemetryRecordContract> {
    let Some(pending) = pending else {
        return Vec::new();
    };
    let source_path = pending.source_path.clone();
    parse_folded_lines(
        observed_at,
        input_id,
        &source_path,
        flush_pending(Some(pending)),
    )
}

pub(super) fn flush_pending_if_source_changes(
    records: &mut Vec<TelemetryRecordContract>,
    pending: &mut Option<PendingMultilineState>,
    observed_at: &str,
    input_id: &str,
    next_source_path: &Path,
) {
    if pending
        .as_ref()
        .is_some_and(|entry| entry.source_path != next_source_path.display().to_string())
    {
        records.extend(records_from_pending(observed_at, input_id, pending.take()));
    }
}

pub(super) fn rebind_pending_source_on_rotate(
    pending: &mut Option<PendingMultilineState>,
    previous_source_path: &Path,
    rotated_path: &Path,
) {
    let previous_source_path = previous_source_path.display().to_string();
    let rotated_path = rotated_path.display().to_string();
    if let Some(pending) = pending
        .as_mut()
        .filter(|entry| entry.source_path == previous_source_path)
    {
        pending.source_path = rotated_path;
    }
}

pub(super) fn pending_should_flush(
    pending: Option<&PendingMultilineState>,
    observed_at: &str,
) -> bool {
    let Some(pending) = pending else {
        return false;
    };
    let Ok(last_updated_at) = OffsetDateTime::parse(&pending.last_updated_at, &Rfc3339) else {
        return true;
    };
    let Ok(observed_at) = OffsetDateTime::parse(observed_at, &Rfc3339) else {
        return true;
    };
    observed_at - last_updated_at >= time::Duration::milliseconds(MULTILINE_IDLE_FLUSH_MS)
}
