//! Local result reporting preparation.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use wp_agent_contracts::action_result::ActionResultContract;
use wp_agent_contracts::gateway::{ReportActionResult, ResultAttestation};
use wp_agent_shared::fs::{read_json, write_json_atomic};
use wp_agent_shared::integrity::{dev_placeholder_issuer, digest_json, sign_dev_placeholder};
use wp_agent_shared::paths::REPORT_ENVELOPE_SUFFIX;
use wp_agent_shared::time::now_rfc3339;
use wp_agent_validate::gateway::validate_report_action_result;

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

#[derive(Debug, Clone)]
enum LocalReportStatus {
    Ready(PreparedReport),
    MissingState,
    CorruptState,
    MissingEnvelope(ReportingState),
    CorruptEnvelope(ReportingState),
}

pub fn load_complete_local_report(
    state_dir: &Path,
    execution_id: &str,
) -> io::Result<Option<PreparedReport>> {
    Ok(match inspect_local_report(state_dir, execution_id)? {
        LocalReportStatus::Ready(prepared) => Some(prepared),
        LocalReportStatus::MissingState
        | LocalReportStatus::CorruptState
        | LocalReportStatus::MissingEnvelope(_)
        | LocalReportStatus::CorruptEnvelope(_) => None,
    })
}

pub fn ensure_local_report(request: ReportingRequest<'_>) -> io::Result<PreparedReport> {
    match inspect_local_report(request.state_dir, request.execution_id)? {
        LocalReportStatus::Ready(prepared) => Ok(prepared),
        LocalReportStatus::MissingState | LocalReportStatus::CorruptState => {
            prepare_local_report(request)
        }
        LocalReportStatus::MissingEnvelope(state) | LocalReportStatus::CorruptEnvelope(state) => {
            rebuild_report_envelope(request, &state)
        }
    }
}

pub fn prepare_local_report(request: ReportingRequest<'_>) -> io::Result<PreparedReport> {
    let (envelope, result_digest, result_signature) = build_report_envelope(
        &request,
        request.action_id,
        request.plan_digest,
        1,
        None,
        None,
    )?;

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
        Some(result_digest),
        Some(result_signature),
        0,
        None,
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

pub fn rebuild_report_envelope(
    request: ReportingRequest<'_>,
    state: &ReportingState,
) -> io::Result<PreparedReport> {
    let (envelope, result_digest, result_signature) = build_report_envelope(
        &request,
        &state.action_id,
        &state.plan_digest,
        state.report_attempt.saturating_add(1),
        state.result_digest.clone(),
        state.result_signature.clone(),
    )?;

    let envelope_path = envelope_path_for(request.state_dir, request.execution_id);
    write_json_atomic(&envelope_path, &envelope)?;

    let mut rebuilt_state = state.clone();
    let mut state_changed = false;
    let envelope_path_str = envelope_path.display().to_string();
    if rebuilt_state.report_envelope_path.as_deref() != Some(envelope_path_str.as_str()) {
        rebuilt_state.report_envelope_path = Some(envelope_path_str);
        state_changed = true;
    }
    if rebuilt_state.result_digest.as_deref() != Some(result_digest.as_str()) {
        rebuilt_state.result_digest = Some(result_digest);
        state_changed = true;
    }
    if rebuilt_state.result_signature.as_deref() != Some(result_signature.as_str()) {
        rebuilt_state.result_signature = Some(result_signature);
        state_changed = true;
    }
    if state_changed {
        let state_path = reporting::path_for(request.state_dir, request.execution_id);
        reporting::store(&state_path, &rebuilt_state)?;
    }

    Ok(PreparedReport {
        envelope_path,
        envelope,
        state: rebuilt_state,
    })
}

pub fn envelope_path_for(state_dir: &Path, execution_id: &str) -> PathBuf {
    state_dir
        .join("reporting")
        .join(format!("{execution_id}{REPORT_ENVELOPE_SUFFIX}"))
}

pub fn remove_local_report_artifacts(state_dir: &Path, execution_id: &str) -> io::Result<()> {
    for path in [
        reporting::path_for(state_dir, execution_id),
        envelope_path_for(state_dir, execution_id),
    ] {
        if path.exists() {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}

fn build_report_envelope(
    request: &ReportingRequest<'_>,
    action_id: &str,
    plan_digest: &str,
    report_attempt: u32,
    result_digest: Option<String>,
    result_signature: Option<String>,
) -> io::Result<(ReportActionResult, String, String)> {
    let result_digest = result_digest.unwrap_or(digest_json(request.result)?);
    let result_signature =
        result_signature.unwrap_or_else(|| sign_dev_placeholder(request.agent_id, &result_digest));
    let reported_at = now_rfc3339();
    let attestation = ResultAttestation {
        result_digest: result_digest.clone(),
        signature: result_signature.clone(),
        issued_by: dev_placeholder_issuer(request.agent_id),
        attested_at: reported_at.clone(),
    };
    let envelope = ReportActionResult::new(
        format!("rep_{}_{}", request.execution_id, report_attempt),
        action_id.to_string(),
        report_attempt,
        request.result.final_status,
        request.execution_id.to_string(),
        plan_digest.to_string(),
        request.agent_id.to_string(),
        request.instance_id.to_string(),
        attestation,
        reported_at,
        request.result.clone(),
    );
    validate_report_action_result(&envelope)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err.code))?;
    Ok((envelope, result_digest, result_signature))
}

fn inspect_local_report(state_dir: &Path, execution_id: &str) -> io::Result<LocalReportStatus> {
    let state_path = reporting::path_for(state_dir, execution_id);
    if !state_path.exists() {
        return Ok(LocalReportStatus::MissingState);
    }

    let state = match reporting::load(&state_path) {
        Ok(state) => state,
        Err(_) => return Ok(LocalReportStatus::CorruptState),
    };
    let envelope_path = envelope_path_for(state_dir, execution_id);
    if !envelope_path.exists() {
        return Ok(LocalReportStatus::MissingEnvelope(state));
    }

    let envelope = match read_json(&envelope_path) {
        Ok(envelope) => envelope,
        Err(_) => return Ok(LocalReportStatus::CorruptEnvelope(state)),
    };
    Ok(LocalReportStatus::Ready(PreparedReport {
        envelope_path,
        envelope,
        state,
    }))
}
