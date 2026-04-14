//! Conversion from folded lines into structured telemetry records.

use wp_agent_contracts::telemetry_record::TelemetryRecordContract;

use super::multiline::FoldedLine;

pub fn parse_folded_lines(
    observed_at: &str,
    input_id: &str,
    source_path: &str,
    lines: Vec<FoldedLine>,
) -> Vec<TelemetryRecordContract> {
    lines
        .into_iter()
        .map(|line| {
            TelemetryRecordContract::new_log(
                observed_at.to_string(),
                input_id.to_string(),
                source_path.to_string(),
                line.body,
                line.start_offset,
                line.end_offset,
            )
        })
        .collect()
}
