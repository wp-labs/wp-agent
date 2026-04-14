//! Edge daemon skeleton.

pub mod bootstrap;
pub mod capability_report;
pub mod config_runtime;
pub mod daemon;
pub mod execution_support;
pub mod local_exec;
pub mod process_control;
pub mod quarantine;
pub mod recovery;
pub mod reporting_pipeline;
mod runtime_entry;
pub mod scheduler;
pub mod self_observability;
pub mod state_store;
pub(crate) mod telemetry;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    runtime_entry::run()
}
