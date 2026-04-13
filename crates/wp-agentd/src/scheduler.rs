//! Local execution queue scheduler.

use std::io;
use std::path::{Path, PathBuf};

use wp_agent_contracts::action_plan::{ActionPlanContract, RiskLevel};
use wp_agent_contracts::action_result::ActionResultContract;
use wp_agent_contracts::gateway::ReportActionResult;
use wp_agent_shared::fs::{read_json, write_json_atomic};
use wp_agent_shared::integrity::digest_json;
use wp_agent_shared::paths::{ACTIONS_DIR, WORKDIR_PLAN_FILE, WORKDIR_RESULT_FILE};
use wp_agent_shared::time::{after_millis_rfc3339, now_rfc3339};

use crate::execution_support::{final_state_name, find_duplicate_execution};
use crate::local_exec::{LocalExecRequest, execute as execute_local, next_execution_id};
use crate::process_control::running_state_is_active;
use crate::quarantine::{QuarantineRequest, quarantine_execution};
use crate::recovery::synthesize_recovery_result;
use crate::reporting_pipeline::{
    ReportingRequest, ensure_local_report, load_complete_local_report, prepare_local_report,
};
use crate::state_store::execution_queue::{self, ExecutionQueueItem};
use crate::state_store::running;

#[derive(Debug, Clone)]
pub struct SchedulerRequest {
    pub run_dir: PathBuf,
    pub state_dir: PathBuf,
    pub plan: ActionPlanContract,
}

#[derive(Debug, Clone)]
pub struct SchedulerOutcome {
    pub execution_id: String,
    pub plan_digest: String,
}

#[derive(Debug, Clone)]
pub struct DrainRequest {
    pub run_dir: PathBuf,
    pub state_dir: PathBuf,
    pub exec_bin: PathBuf,
    pub instance_id: String,
    pub cancel_grace_ms: u64,
    pub stdout_limit_bytes: u64,
    pub stderr_limit_bytes: u64,
}

#[derive(Debug, Clone)]
pub struct DrainOutcome {
    pub execution_id: String,
    pub plan_digest: String,
    pub report: ReportActionResult,
}

enum ExistingExecution {
    None,
    Blocked,
    Reconciled(Box<DrainOutcome>),
}

enum QueueHeadDisposition {
    Blocked,
    Quarantined,
    Produced(DrainOutcome),
}

pub fn submit_local_plan(request: &SchedulerRequest) -> io::Result<SchedulerOutcome> {
    let execution_id = next_execution_id();
    let plan_digest = digest_json(&request.plan)?;
    let deadline_at = Some(after_millis_rfc3339(
        request.plan.constraints.max_total_duration_ms,
    ));
    if let Some(existing_execution_id) = find_duplicate_execution(
        &request.state_dir,
        &request.plan.meta.action_id,
        &plan_digest,
    )? {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!(
                "duplicate action plan already tracked locally: action_id={} plan_digest={} execution_id={existing_execution_id}",
                request.plan.meta.action_id, plan_digest
            ),
        ));
    }

    let workdir = request.run_dir.join(ACTIONS_DIR).join(&execution_id);
    std::fs::create_dir_all(&workdir)?;
    wp_agent_shared::fs::write_json_atomic(&workdir.join(WORKDIR_PLAN_FILE), &request.plan)?;

    let queue_path = execution_queue::path_for(&request.state_dir);
    let queue_write = (|| -> io::Result<()> {
        let mut queue = execution_queue::load_or_default(&queue_path)?;
        queue.enqueue(ExecutionQueueItem::new(
            execution_id.clone(),
            request.plan.meta.action_id.clone(),
            plan_digest.clone(),
            request.plan.meta.request_id.clone(),
            100,
            now_rfc3339(),
            deadline_at,
            true,
            Some(risk_level_name(request.plan.constraints.risk_level).to_string()),
        ));
        execution_queue::store(&queue_path, &queue)
    })();
    if let Err(err) = queue_write {
        let _ = std::fs::remove_dir_all(&workdir);
        return Err(err);
    }

    Ok(SchedulerOutcome {
        execution_id,
        plan_digest,
    })
}

pub fn drain_next(request: &DrainRequest) -> io::Result<bool> {
    Ok(drain_next_with_report(request)?.is_some())
}

pub fn drain_next_with_report(request: &DrainRequest) -> io::Result<Option<DrainOutcome>> {
    let queue_path = execution_queue::path_for(&request.state_dir);
    let mut queue = execution_queue::load_or_default(&queue_path)?;
    loop {
        let Some(item) = queue.items.first().cloned() else {
            return Ok(None);
        };

        match handle_queue_head(request, &item)? {
            QueueHeadDisposition::Blocked => return Ok(None),
            QueueHeadDisposition::Quarantined => {
                queue = execution_queue::load_or_default(&queue_path)?;
            }
            QueueHeadDisposition::Produced(outcome) => {
                queue.remove(&item.execution_id);
                execution_queue::store(&queue_path, &queue)?;
                return Ok(Some(outcome));
            }
        }
    }
}

fn risk_level_name(risk: RiskLevel) -> &'static str {
    match risk {
        RiskLevel::R0 => "R0",
        RiskLevel::R1 => "R1",
        RiskLevel::R2 => "R2",
        RiskLevel::R3 => "R3",
    }
}

fn read_queued_plan(workdir: &Path) -> io::Result<ActionPlanContract> {
    read_json(&workdir.join(WORKDIR_PLAN_FILE))
}

fn reconcile_existing_execution(
    request: &DrainRequest,
    item: &ExecutionQueueItem,
    plan: &ActionPlanContract,
    workdir: &Path,
    running_path: &Path,
) -> io::Result<ExistingExecution> {
    if running_path.exists() {
        let mut state = running::load(running_path)?;
        if running_state_is_active(&mut state, running_path)? {
            return Ok(ExistingExecution::Blocked);
        }
        if let Some(outcome) = reconcile_completed_execution(request, item, plan, workdir)? {
            running::remove(running_path)?;
            return Ok(ExistingExecution::Reconciled(Box::new(outcome)));
        }

        let result_path = workdir.join(WORKDIR_RESULT_FILE);
        let recovered = synthesize_recovery_result(&state);
        write_json_atomic(&result_path, &recovered)?;
        let prepared = ensure_local_report(ReportingRequest {
            state_dir: &request.state_dir,
            execution_id: &item.execution_id,
            action_id: &item.action_id,
            request_id: &item.request_id,
            plan_digest: &item.plan_digest,
            agent_id: &plan.target.agent_id,
            instance_id: &request.instance_id,
            final_state: final_state_name(&recovered),
            result_path: &result_path,
            result: &recovered,
        })?;
        running::remove(running_path)?;
        return Ok(ExistingExecution::Reconciled(Box::new(DrainOutcome {
            execution_id: item.execution_id.clone(),
            plan_digest: item.plan_digest.clone(),
            report: prepared.envelope,
        })));
    }

    Ok(
        match reconcile_completed_execution(request, item, plan, workdir)? {
            Some(outcome) => ExistingExecution::Reconciled(Box::new(outcome)),
            None => ExistingExecution::None,
        },
    )
}

fn handle_queue_head(
    request: &DrainRequest,
    item: &ExecutionQueueItem,
) -> io::Result<QueueHeadDisposition> {
    let workdir = request.run_dir.join(ACTIONS_DIR).join(&item.execution_id);
    let running_path = running::path_for(&request.state_dir, &item.execution_id);
    let plan = match read_queued_plan(&workdir) {
        Ok(plan) => plan,
        Err(err) => {
            quarantine_execution(QuarantineRequest::queued_item(
                &request.state_dir,
                item,
                format!("queued execution plan unavailable: {err}"),
                Some(&running_path),
            ))?;
            return Ok(QueueHeadDisposition::Quarantined);
        }
    };

    match reconcile_existing_execution(request, item, &plan, &workdir, &running_path) {
        Ok(ExistingExecution::Blocked) => return Ok(QueueHeadDisposition::Blocked),
        Ok(ExistingExecution::Reconciled(outcome)) => {
            return Ok(QueueHeadDisposition::Produced(*outcome));
        }
        Ok(ExistingExecution::None) => {}
        Err(err) => {
            quarantine_execution(QuarantineRequest::queued_item(
                &request.state_dir,
                item,
                format!("queued execution state unavailable: {err}"),
                Some(&running_path),
            ))?;
            return Ok(QueueHeadDisposition::Quarantined);
        }
    }

    let local_result = match execute_local(&LocalExecRequest {
        execution_id: item.execution_id.clone(),
        run_dir: request.run_dir.clone(),
        state_dir: request.state_dir.clone(),
        exec_bin: request.exec_bin.clone(),
        cancel_grace_ms: request.cancel_grace_ms,
        stdout_limit_bytes: request.stdout_limit_bytes,
        stderr_limit_bytes: request.stderr_limit_bytes,
        plan_digest: item.plan_digest.clone(),
        request_id: item.request_id.clone(),
        plan: plan.clone(),
    }) {
        Ok(local_result) => local_result,
        Err(err) => {
            quarantine_execution(QuarantineRequest::queued_item(
                &request.state_dir,
                item,
                format!("local execution failed: {err}"),
                Some(&running_path),
            ))?;
            return Ok(QueueHeadDisposition::Quarantined);
        }
    };

    let prepared = match prepare_local_report(ReportingRequest {
        state_dir: &request.state_dir,
        execution_id: &item.execution_id,
        action_id: &item.action_id,
        request_id: &item.request_id,
        plan_digest: &item.plan_digest,
        agent_id: &plan.target.agent_id,
        instance_id: &request.instance_id,
        final_state: final_state_name(&local_result.result),
        result_path: &local_result.workdir.join(WORKDIR_RESULT_FILE),
        result: &local_result.result,
    }) {
        Ok(prepared) => prepared,
        Err(err) => {
            quarantine_execution(QuarantineRequest::queued_item(
                &request.state_dir,
                item,
                format!("local execution report preparation failed: {err}"),
                Some(&running_path),
            ))?;
            return Ok(QueueHeadDisposition::Quarantined);
        }
    };

    running::remove(&running_path)?;
    Ok(QueueHeadDisposition::Produced(DrainOutcome {
        execution_id: item.execution_id.clone(),
        plan_digest: item.plan_digest.clone(),
        report: prepared.envelope,
    }))
}

fn reconcile_completed_execution(
    request: &DrainRequest,
    item: &ExecutionQueueItem,
    plan: &ActionPlanContract,
    workdir: &Path,
) -> io::Result<Option<DrainOutcome>> {
    let result_path = workdir.join(WORKDIR_RESULT_FILE);
    if let Some(prepared) = load_complete_local_report(&request.state_dir, &item.execution_id)? {
        return Ok(Some(DrainOutcome {
            execution_id: item.execution_id.clone(),
            plan_digest: item.plan_digest.clone(),
            report: prepared.envelope,
        }));
    }

    if !result_path.exists() {
        return Ok(None);
    }

    let result: ActionResultContract = read_json(&result_path)?;
    let final_state = final_state_name(&result);
    let prepared = ensure_local_report(ReportingRequest {
        state_dir: &request.state_dir,
        execution_id: &item.execution_id,
        action_id: &item.action_id,
        request_id: &item.request_id,
        plan_digest: &item.plan_digest,
        agent_id: &plan.target.agent_id,
        instance_id: &request.instance_id,
        final_state,
        result_path: &result_path,
        result: &result,
    })?;

    Ok(Some(DrainOutcome {
        execution_id: item.execution_id.clone(),
        plan_digest: item.plan_digest.clone(),
        report: prepared.envelope,
    }))
}
