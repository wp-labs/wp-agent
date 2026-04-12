//! `ActionPlan` runtime skeleton.

pub mod result_writer;
pub mod runtime;
pub mod workdir;

pub fn run() -> Result<(), String> {
    let workdir_path = parse_cli_args(std::env::args().skip(1))?;
    let workdir = workdir::ExecutionWorkdir::open(&workdir_path)
        .map_err(|err| format!("open workdir failed: {err}"))?;
    let result = runtime::execute(&workdir).map_err(|err| format!("execute failed: {err}"))?;
    result_writer::write(&workdir, &result)
        .map_err(|err| format!("persist result failed: {err}"))?;
    eprintln!("wp-agent-exec finished in {}", workdir.root.display());
    Ok(())
}

fn parse_cli_args<I>(mut args: I) -> Result<std::path::PathBuf, String>
where
    I: Iterator<Item = String>,
{
    match args.next().as_deref() {
        Some("run") => {}
        Some(other) => return Err(format!("unsupported subcommand: {other}")),
        None => return std::env::current_dir().map_err(|err| err.to_string()),
    }

    match (args.next().as_deref(), args.next()) {
        (Some("--workdir"), Some(path)) => Ok(std::path::PathBuf::from(path)),
        (Some(flag), _) => Err(format!("unsupported flag: {flag}")),
        (None, _) => Err("missing --workdir <path>".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use wp_agent_contracts::action_plan::{
        ActionPlanConstraints, ActionPlanContract, ActionPlanMeta, ActionPlanProgram,
        ActionPlanStep, ActionPlanTarget, ApprovalMode, RiskLevel,
    };
    use wp_agent_contracts::action_result::FinalStatus;
    use wp_agent_shared::fs::read_json;
    use wp_agent_shared::time::now_rfc3339;

    use crate::result_writer;
    use crate::runtime;
    use crate::workdir::{ExecRuntimeContext, ExecutionWorkdir};

    fn temp_dir(name: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("duration")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("wp-agent-exec-{name}-{suffix}"));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    fn write_fixture_workdir(dir: &std::path::Path) {
        let workdir = ExecutionWorkdir::open(dir).expect("open workdir");
        let plan = ActionPlanContract::new(
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
        );
        let runtime = ExecRuntimeContext {
            execution_id: "exec_001".to_string(),
            spawned_at: now_rfc3339(),
            deadline_at: None,
            agent_id: "agent-001".to_string(),
            node_id: "node-001".to_string(),
            workdir: dir.display().to_string(),
        };

        wp_agent_shared::fs::write_json_atomic(&workdir.plan_path, &plan).expect("write plan");
        wp_agent_shared::fs::write_json_atomic(&workdir.runtime_path, &runtime)
            .expect("write runtime");
    }

    #[test]
    fn execute_workdir_writes_result_and_state() {
        let dir = temp_dir("success");
        write_fixture_workdir(&dir);

        let workdir = ExecutionWorkdir::open(&dir).expect("open workdir");
        let result = runtime::execute(&workdir).expect("execute");
        result_writer::write(&workdir, &result).expect("write result");

        let stored_result: wp_agent_contracts::action_result::ActionResultContract =
            read_json(&workdir.result_path).expect("read result");
        let stored_state: crate::workdir::ExecProgressState =
            read_json(&workdir.state_path).expect("read state");

        assert_eq!(stored_result.final_status, FinalStatus::Succeeded);
        assert_eq!(stored_state.state, "succeeded");
    }
}
