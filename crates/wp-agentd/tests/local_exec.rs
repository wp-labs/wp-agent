use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
#[cfg(unix)]
use std::{os::unix::fs::PermissionsExt, path::Path};

use wp_agent_contracts::action_plan::{
    ActionPlanConstraints, ActionPlanContract, ActionPlanMeta, ActionPlanProgram, ActionPlanStep,
    ActionPlanTarget, ApprovalMode, RiskLevel,
};
use wp_agent_contracts::action_result::FinalStatus;
use wp_agent_shared::fs::read_json;
use wp_agentd::bootstrap;
use wp_agentd::reporting_pipeline;
use wp_agentd::scheduler::{SchedulerRequest, submit_local_plan};
use wp_agentd::state_store::{execution_queue, reporting, running};

fn temp_dir(name: &str) -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("duration")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("wp-agentd-local-exec-{name}-{suffix}"));
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

#[cfg(unix)]
fn test_exec_bin(root: &Path) -> PathBuf {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root");
    let wrapper = root.join("wp-agent-exec-wrapper.sh");
    fs::write(
        &wrapper,
        format!(
            "#!/bin/sh\ncd \"{}\"\nexec cargo run -q -p wp-agent-exec -- \"$@\"\n",
            workspace_root.display()
        ),
    )
    .expect("write wrapper");
    let mut perms = fs::metadata(&wrapper)
        .expect("wrapper metadata")
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&wrapper, perms).expect("set wrapper permissions");
    wrapper
}

fn sample_plan() -> ActionPlanContract {
    ActionPlanContract::new(
        ActionPlanMeta {
            action_id: "act_001".to_string(),
            request_id: "req_001".to_string(),
            template_id: None,
            tenant_id: "tenant_a".to_string(),
            environment_id: "prod-cn".to_string(),
            plan_version: 1,
            compiled_at: "2026-04-12T10:00:00Z".to_string(),
            expires_at: "2026-04-12T10:05:00Z".to_string(),
        },
        ActionPlanTarget {
            agent_id: "agent-001".to_string(),
            instance_id: Some("instance-001".to_string()),
            node_id: "node-001".to_string(),
            host_name: None,
            platform: "linux".to_string(),
            arch: "amd64".to_string(),
            selectors: Default::default(),
        },
        ActionPlanConstraints {
            risk_level: RiskLevel::R1,
            approval_ref: None,
            approval_mode: ApprovalMode::Required,
            requested_by: "alice@example.com".to_string(),
            reason: None,
            max_total_duration_ms: 30_000,
            step_timeout_default_ms: 10_000,
            execution_profile: "agent_exec_v1".to_string(),
            required_capabilities: vec!["process.list".to_string()],
        },
        ActionPlanProgram {
            entry: "step_collect".to_string(),
            steps: vec![ActionPlanStep {
                id: "step_collect".to_string(),
                kind: "invoke".to_string(),
                op: Some("process.list".to_string()),
            }],
        },
    )
}

#[cfg(unix)]
#[test]
fn scheduler_drains_queue_and_prepares_report() {
    let root = temp_dir("spawn");
    let run_dir = root.join("run");
    let state_dir = root.join("state");
    let log_dir = root.join("log");
    bootstrap::initialize(&root, &run_dir, &state_dir, &log_dir).expect("bootstrap");

    let outcome = submit_local_plan(&SchedulerRequest {
        run_dir: run_dir.clone(),
        state_dir: state_dir.clone(),
        exec_bin: test_exec_bin(&root),
        plan: sample_plan(),
        instance_id: "instance-001".to_string(),
    })
    .expect("submit local plan");

    let queue_path = execution_queue::path_for(&state_dir);
    let queue_state = execution_queue::load_or_default(&queue_path).expect("queue state");
    let reporting_path = reporting::path_for(&state_dir, &outcome.execution_id);
    let running_path = running::path_for(&state_dir, &outcome.execution_id);
    let reporting_state: reporting::ReportingState =
        read_json(&reporting_path).expect("read reporting state");
    let report_envelope_path =
        reporting_pipeline::envelope_path_for(&state_dir, &outcome.execution_id);
    let report_envelope: wp_agent_contracts::gateway::ReportActionResult =
        read_json(&report_envelope_path).expect("read report envelope");

    assert!(queue_state.items.is_empty());
    assert!(!running_path.exists());
    assert_eq!(outcome.report.final_status, "succeeded");
    assert_eq!(outcome.report.result.final_status, FinalStatus::Succeeded);
    assert_eq!(reporting_state.final_state, "succeeded");
    assert_eq!(reporting_state.plan_digest, outcome.plan_digest);
    assert_eq!(report_envelope.execution_id, outcome.execution_id);
    assert_eq!(report_envelope.action_id, "act_001");
    assert_eq!(report_envelope.result.final_status, FinalStatus::Succeeded);
    assert!(reporting_state.result_digest.is_some());
    assert!(reporting_state.result_signature.is_some());
    assert!(report_envelope_path.exists());
}
