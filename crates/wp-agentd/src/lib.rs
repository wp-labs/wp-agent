//! Edge daemon skeleton.

use std::io;
use std::path::{Path, PathBuf};

pub mod bootstrap;
pub mod config_runtime;
pub mod daemon;
pub mod execution_support;
pub mod local_exec;
pub mod process_control;
pub mod quarantine;
pub mod recovery;
pub mod reporting_pipeline;
pub mod scheduler;
pub mod self_observability;
pub mod state_store;

pub fn run() {
    let root = std::env::current_dir().expect("current_dir");
    let config_root = root.join(".wp-agentd");
    let config = config_runtime::load_or_init(&config_root).expect("load config");
    let paths = &config.paths;
    let root_dir = Path::new(&paths.root_dir);
    let run_dir = Path::new(&paths.run_dir);
    let state_dir = Path::new(&paths.state_dir);
    let log_dir = Path::new(&paths.log_dir);

    bootstrap::initialize(root_dir, run_dir, state_dir, log_dir).expect("bootstrap");
    let runtime_path = state_store::agent_runtime::path_for(state_dir);
    let mut runtime_state = state_store::agent_runtime::load_or_default(&runtime_path)
        .expect("load default runtime state");
    sync_runtime_identity(&mut runtime_state, &config);
    state_store::agent_runtime::store(&runtime_path, &runtime_state).expect("write runtime state");
    let queue_path = state_store::execution_queue::path_for(state_dir);
    let queue_state =
        state_store::execution_queue::load_or_default(&queue_path).expect("load execution queue");
    state_store::execution_queue::store(&queue_path, &queue_state).expect("write execution queue");
    self_observability::register();
    let exec_bin = resolve_exec_bin().expect("resolve wp-agent-exec");
    let loop_ctx = daemon::DaemonLoop {
        config: &config,
        exec_bin: &exec_bin,
    };

    if std::env::var("WP_AGENTD_RUN_ONCE").ok().as_deref() == Some("1") {
        let snapshot = daemon::run_once(&loop_ctx).expect("daemon tick");
        self_observability::emit(&snapshot);
        return;
    }

    daemon::run_forever(loop_ctx).expect("daemon loop");
}

fn resolve_exec_bin() -> io::Result<PathBuf> {
    let env_override = std::env::var_os("WP_AGENT_EXEC_BIN").map(PathBuf::from);
    let current_exe = std::env::current_exe()?;
    resolve_exec_bin_from(&current_exe, env_override.as_deref())
}

fn resolve_exec_bin_from(current_exe: &Path, env_override: Option<&Path>) -> io::Result<PathBuf> {
    let candidate = env_override
        .map(Path::to_path_buf)
        .unwrap_or_else(|| current_exe.with_file_name("wp-agent-exec"));
    validate_exec_bin(candidate, env_override.is_some())
}

fn validate_exec_bin(path: PathBuf, from_env: bool) -> io::Result<PathBuf> {
    let metadata = std::fs::metadata(&path).map_err(|err| {
        let origin = if from_env {
            "WP_AGENT_EXEC_BIN"
        } else {
            "current executable sibling"
        };
        io::Error::new(
            err.kind(),
            format!(
                "wp-agent-exec was not found via {origin}: {} ({err})",
                path.display()
            ),
        )
    })?;
    if !metadata.is_file() {
        return Err(io::Error::other(format!(
            "wp-agent-exec path is not a file: {}",
            path.display()
        )));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        if metadata.permissions().mode() & 0o111 == 0 {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!("wp-agent-exec is not executable: {}", path.display()),
            ));
        }
    }
    Ok(path)
}

fn sync_runtime_identity(
    runtime_state: &mut wp_agent_contracts::state_exec::AgentRuntimeState,
    config: &wp_agent_contracts::agent_config::AgentConfigContract,
) {
    if let Some(agent_id) = config
        .agent
        .agent_id
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        runtime_state.agent_id = agent_id.to_string();
    }
    if let Some(instance_id) = config
        .agent
        .instance_name
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        runtime_state.instance_id = instance_id.to_string();
    }
}

#[cfg(test)]
mod tests {
    use super::{resolve_exec_bin_from, sync_runtime_identity};
    use std::fs;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};
    use wp_agent_contracts::agent_config::{
        AgentConfigContract, AgentSection, ControlPlaneSection, ExecutionSection, PathsSection,
    };
    use wp_agent_contracts::state_exec::{AgentRuntimeState, RuntimeMode};

    fn temp_dir(name: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("duration")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("wp-agentd-lib-{name}-{suffix}"));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn resolve_exec_bin_uses_env_override_when_present() {
        let root = temp_dir("override");
        let current_exe = root.join("bin").join("wp-agentd");
        let override_path = root.join("custom").join("wp-agent-exec");
        fs::create_dir_all(current_exe.parent().expect("current_exe parent"))
            .expect("create current_exe parent");
        fs::create_dir_all(override_path.parent().expect("override parent"))
            .expect("create override parent");
        fs::write(&override_path, b"#!/bin/sh\n").expect("write override");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let mut perms = fs::metadata(&override_path)
                .expect("override metadata")
                .permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&override_path, perms).expect("set override permissions");
        }

        let resolved =
            resolve_exec_bin_from(&current_exe, Some(Path::new(&override_path))).expect("resolve");

        assert_eq!(resolved, override_path);
    }

    #[test]
    fn resolve_exec_bin_rejects_missing_candidate() {
        let root = temp_dir("missing");
        let current_exe = root.join("bin").join("wp-agentd");
        fs::create_dir_all(current_exe.parent().expect("current_exe parent"))
            .expect("create current_exe parent");

        let err = resolve_exec_bin_from(&current_exe, None).expect_err("missing exec should fail");
        assert!(err.to_string().contains("wp-agent-exec was not found"));
    }

    #[cfg(unix)]
    #[test]
    fn resolve_exec_bin_rejects_non_executable_file() {
        let root = temp_dir("not-executable");
        let current_exe = root.join("bin").join("wp-agentd");
        let candidate = root.join("bin").join("wp-agent-exec");
        fs::create_dir_all(current_exe.parent().expect("current_exe parent"))
            .expect("create current_exe parent");
        fs::write(&candidate, b"#!/bin/sh\n").expect("write candidate");
        let mut perms = fs::metadata(&candidate)
            .expect("candidate metadata")
            .permissions();
        perms.set_mode(0o644);
        fs::set_permissions(&candidate, perms).expect("set candidate permissions");

        let err = resolve_exec_bin_from(&current_exe, None)
            .expect_err("non executable candidate should fail");

        assert!(err.to_string().contains("not executable"));
    }

    #[test]
    fn sync_runtime_identity_prefers_config_identity_when_present() {
        let mut runtime = AgentRuntimeState::new(
            "local-agent".to_string(),
            "local-instance".to_string(),
            "0.1.0".to_string(),
            RuntimeMode::Normal,
            "2026-04-12T10:00:00Z".to_string(),
        );
        let config = AgentConfigContract::new(
            AgentSection {
                agent_id: Some("agent-from-config".to_string()),
                environment_id: Some("prod".to_string()),
                instance_name: Some("instance-from-config".to_string()),
            },
            ControlPlaneSection {
                enabled: false,
                endpoint: None,
                tls_mode: None,
                auth_mode: None,
            },
            PathsSection {
                root_dir: ".".to_string(),
                run_dir: "run".to_string(),
                state_dir: "state".to_string(),
                log_dir: "log".to_string(),
            },
            ExecutionSection {
                max_running_actions: 1,
                cancel_grace_ms: 5_000,
                default_stdout_limit_bytes: 1,
                default_stderr_limit_bytes: 1,
            },
        );

        sync_runtime_identity(&mut runtime, &config);

        assert_eq!(runtime.agent_id, "agent-from-config");
        assert_eq!(runtime.instance_id, "instance-from-config");
    }
}
