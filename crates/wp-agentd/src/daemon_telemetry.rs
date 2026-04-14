use std::path::PathBuf;

use wp_agent_contracts::agent_config::{AgentConfigContract, LogFileInputSection};

use crate::telemetry::logs::file_input::{FileInputConfig, FileInputProcessor, ProcessOutcome};
use crate::telemetry::logs::file_watcher::StartupPosition;
use crate::telemetry::logs::multiline::MultilineMode;
use crate::telemetry::warp_parse::FileRecordSink;

pub(super) struct TelemetryTick {
    pub(super) outcomes: Vec<ProcessOutcome>,
    pub(super) failures: Vec<TelemetryFailure>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum TelemetryFailureKind {
    MissingInput,
    ProcessingFailed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct TelemetryFailure {
    pub(super) kind: TelemetryFailureKind,
    pub(super) input_id: String,
    pub(super) path: String,
    pub(super) detail: String,
}

impl TelemetryTick {
    pub(super) fn is_active(&self) -> bool {
        !self.failures.is_empty()
            || self.outcomes.iter().any(|outcome| {
                outcome.records_processed > 0 || outcome.replayed_spool > 0 || outcome.spooled > 0
            })
    }
}

pub(super) fn process_telemetry_inputs(config: &AgentConfigContract) -> TelemetryTick {
    let mut outcomes = Vec::new();
    let mut failures = Vec::new();
    for input in &config.telemetry.logs.file_inputs {
        let source_path = PathBuf::from(&input.path);
        if !source_path.exists() {
            failures.push(TelemetryFailure {
                kind: TelemetryFailureKind::MissingInput,
                input_id: input.input_id.clone(),
                path: input.path.clone(),
                detail: "source path does not exist".to_string(),
            });
            continue;
        }
        let sink = FileRecordSink::new(PathBuf::from(&config.telemetry.logs.output_file));
        let mut processor = FileInputProcessor::new(
            FileInputConfig {
                input_id: input.input_id.clone(),
                source_path,
                state_dir: PathBuf::from(&config.paths.state_dir),
                spool_path: PathBuf::from(&config.telemetry.logs.spool_dir)
                    .join(format!("{}.ndjson", input.input_id)),
                startup_position: startup_position_for(input),
                multiline_mode: multiline_mode_for(input),
                in_memory_budget_bytes: config.telemetry.logs.in_memory_buffer_bytes as usize,
            },
            sink,
        );
        match processor.process_once() {
            Ok(outcome) => outcomes.push(outcome),
            Err(err) => failures.push(TelemetryFailure {
                kind: TelemetryFailureKind::ProcessingFailed,
                input_id: input.input_id.clone(),
                path: input.path.clone(),
                detail: err.to_string(),
            }),
        }
    }
    TelemetryTick { outcomes, failures }
}

fn multiline_mode_for(input: &LogFileInputSection) -> MultilineMode {
    match input.multiline_mode.as_str() {
        "indented" => MultilineMode::IndentedContinuation,
        _ => MultilineMode::None,
    }
}

fn startup_position_for(input: &LogFileInputSection) -> StartupPosition {
    match input.startup_position.as_str() {
        "tail" => StartupPosition::Tail,
        _ => StartupPosition::Head,
    }
}
