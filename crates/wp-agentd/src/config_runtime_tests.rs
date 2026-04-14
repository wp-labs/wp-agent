use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::{
    ConfigError, default_config_template, ensure_default_config, load_from_path, load_or_init,
};

fn temp_dir(name: &str) -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("duration")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("wp-agentd-{name}-{suffix}"));
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

#[test]
fn load_or_init_creates_default_config() {
    let root = temp_dir("init");
    let config = load_or_init(&root).expect("load or init");

    assert!(root.join("agent.toml").exists());
    assert_eq!(config.paths.root_dir, root.display().to_string());
    assert_eq!(config.paths.run_dir, root.join("run").display().to_string());
    assert_eq!(
        config.telemetry.logs.spool_dir,
        root.join("state")
            .join("spool")
            .join("logs")
            .display()
            .to_string()
    );
    assert!(config.agent.instance_name.is_none());
    assert_eq!(config.execution.max_running_actions, 1);
}

#[test]
fn ensure_default_config_reports_created_then_existing() {
    let root = temp_dir("ensure-default");

    let first = ensure_default_config(&root).expect("create default config");
    let second = ensure_default_config(&root).expect("reuse existing config");

    assert!(first.created);
    assert_eq!(first.path, root.join("agent.toml"));
    assert!(!second.created);
    assert_eq!(second.path, first.path);
}

#[test]
fn default_config_template_contains_file_input_example() {
    let template = default_config_template();

    assert!(template.contains("[telemetry.logs]"));
    assert!(template.contains("# [[telemetry.logs.file_inputs]]"));
    assert!(template.contains("output_file = \"log/warp-parse-records.ndjson\""));
    assert!(!template.contains("max_running_actions = 1"));
    assert!(!template.contains("instance_name = \"local\""));
}

#[test]
fn load_from_path_expands_env_and_resolves_paths() {
    let root = temp_dir("load");
    let config_path = root.join("agent.toml");
    let home = std::env::var("HOME").expect("HOME");
    fs::write(
        &config_path,
        r#"
schema_version = "v1"

[agent]
environment_id = "prod"
instance_name = "${HOME}/instance"

[control_plane]
enabled = false

[paths]
root_dir = "${HOME}/agent-root"
run_dir = "run"
state_dir = "state"
log_dir = "log"

[execution]
max_running_actions = 1
cancel_grace_ms = 5000
default_stdout_limit_bytes = 1048576
default_stderr_limit_bytes = 1048576

[telemetry.logs]
spool_dir = "state/spool/logs"
output_file = "log/out.ndjson"

[[telemetry.logs.file_inputs]]
input_id = "app"
path = "${HOME}/logs/app.log"
multiline_mode = "indented"
"#,
    )
    .expect("write config");

    let config = load_from_path(&config_path).expect("load config");

    assert_eq!(
        config.paths.root_dir,
        Path::new(&home).join("agent-root").display().to_string()
    );
    assert_eq!(
        config.paths.state_dir,
        Path::new(&home)
            .join("agent-root")
            .join("state")
            .display()
            .to_string()
    );
    assert_eq!(
        config.agent.instance_name.as_deref(),
        Some(format!("{home}/instance").as_str())
    );
    assert_eq!(
        config.telemetry.logs.output_file,
        Path::new(&home)
            .join("agent-root")
            .join("log")
            .join("out.ndjson")
            .display()
            .to_string()
    );
    assert_eq!(
        config.telemetry.logs.file_inputs[0].path,
        Path::new(&home)
            .join("logs")
            .join("app.log")
            .display()
            .to_string()
    );
}

#[test]
fn load_from_path_rejects_unsupported_max_running_actions() {
    let root = temp_dir("unsupported-max-running-actions");
    let config_path = root.join("agent.toml");
    fs::write(
        &config_path,
        r#"
schema_version = "v1"

[agent]
instance_name = "local"

[control_plane]
enabled = false

[paths]
root_dir = "."
run_dir = "run"
state_dir = "state"
log_dir = "log"

[execution]
max_running_actions = 2
cancel_grace_ms = 5000
default_stdout_limit_bytes = 1048576
default_stderr_limit_bytes = 1048576
"#,
    )
    .expect("write config");

    let err = load_from_path(&config_path).expect_err("unsupported max running actions");
    assert!(matches!(
        err,
        ConfigError::Validation("unsupported_max_running_actions")
    ));
}

#[test]
fn load_from_path_uses_defaults_for_missing_paths_and_execution() {
    let root = temp_dir("missing-defaultable-sections");
    let config_path = root.join("agent.toml");
    fs::write(
        &config_path,
        r#"
schema_version = "v1"

[telemetry.logs]
output_file = "log/out.ndjson"
spool_dir = "state/spool/logs"
"#,
    )
    .expect("write config");

    let config = load_from_path(&config_path).expect("load config");

    assert_eq!(config.paths.root_dir, root.display().to_string());
    assert_eq!(config.paths.run_dir, root.join("run").display().to_string());
    assert_eq!(
        config.paths.state_dir,
        root.join("state").display().to_string()
    );
    assert_eq!(config.paths.log_dir, root.join("log").display().to_string());
    assert_eq!(config.execution.max_running_actions, 1);
    assert_eq!(config.execution.cancel_grace_ms, 5_000);
    assert!(config.agent.instance_name.is_none());
}
