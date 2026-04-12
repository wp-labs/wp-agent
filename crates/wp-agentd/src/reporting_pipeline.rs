//! Local result reporting preparation.

use std::io;
use std::path::{Path, PathBuf};

use wp_agent_contracts::action_result::ActionResultContract;
use wp_agent_contracts::gateway::{ReportActionResult, ResultAttestation};
use wp_agent_shared::fs::write_json_atomic;
use wp_agent_shared::integrity::{digest_json, sign_placeholder};
use wp_agent_shared::paths::REPORT_ENVELOPE_SUFFIX;
use wp_agent_shared::time::now_rfc3339;

use crate::state_store::reporting::{self, ReportingState};

#[derive(Debug, Clone)]
pub struct ReportingRequest<'a> {
    pub state_dir: &'a Path,
    pub execution_id: &'a str,
    pub action_id: &'a str,
    pub request_id: &'a str,
    pub plan_digest: &'a str,
    pub agent_id: &'a str,
    pub instance_id: &'a str,
    pub final_state: &'a str,
    pub result_path: &'a Path,
    pub result: &'a ActionResultContract,
}

#[derive(Debug, Clone)]
pub struct PreparedReport {
    pub envelope_path: PathBuf,
    pub envelope: ReportActionResult,
    pub state: ReportingState,
}

pub fn prepare_local_report(request: ReportingRequest<'_>) -> io::Result<PreparedReport> {
    let result_digest = digest_json(request.result)?;
    let reported_at = now_rfc3339();
    let attestation = ResultAttestation {
        result_digest: result_digest.clone(),
        signature: sign_placeholder(request.agent_id, &result_digest),
        issued_by: request.agent_id.to_string(),
        attested_at: reported_at.clone(),
    };
    let envelope = ReportActionResult::new(
        format!("rep_{}", request.execution_id),
        request.action_id.to_string(),
        1,
        request.final_state.to_string(),
        request.execution_id.to_string(),
        request.plan_digest.to_string(),
        request.agent_id.to_string(),
        request.instance_id.to_string(),
        attestation.clone(),
        reported_at.clone(),
        request.result.clone(),
    );

    let envelope_path = envelope_path_for(request.state_dir, request.execution_id);
    write_json_atomic(&envelope_path, &envelope)?;

    let state = ReportingState::new(
        request.execution_id.to_string(),
        request.action_id.to_string(),
        request.plan_digest.to_string(),
        request.request_id.to_string(),
        request.final_state.to_string(),
        request.result_path.display().to_string(),
        Some(envelope_path.display().to_string()),
        Some(attestation.result_digest),
        Some(attestation.signature),
        1,
        Some(reported_at),
        None,
    );
    let state_path = reporting::path_for(request.state_dir, request.execution_id);
    reporting::store(&state_path, &state)?;

    Ok(PreparedReport {
        envelope_path,
        envelope,
        state,
    })
}

pub fn envelope_path_for(state_dir: &Path, execution_id: &str) -> PathBuf {
    state_dir
        .join("reporting")
        .join(format!("{execution_id}{REPORT_ENVELOPE_SUFFIX}"))
}
