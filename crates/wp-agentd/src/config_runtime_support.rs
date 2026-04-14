use std::env;
use std::path::{Component, Path, PathBuf};

use wp_agent_contracts::agent_config::AgentConfigContract;

use crate::config_runtime::ConfigError;

pub(super) fn default_file_config_text() -> String {
    r#"schema_version = "v1"

[telemetry.logs]
in_memory_buffer_bytes = 1048576
spool_dir = "state/spool/logs"
output_file = "log/warp-parse-records.ndjson"

# 可选：
# [agent]
# # 为空时会自动生成实例名；如需显式指定可取消注释。
# # instance_name = "monitoring-host-01"
#
# 可选：
# [paths]
# # 下面这些默认分别是 ".", "run", "state", "log"
# # root_dir = "."
# # run_dir = "run"
# # state_dir = "state"
# # log_dir = "log"
#
# 示例：把某个监控系统日志文件送到本地 warp-parse record 输出文件。
# 取消注释后，把 path 改成你的真实日志路径。
#
# [[telemetry.logs.file_inputs]]
# input_id = "monitoring-app"
# path = "/var/log/monitoring/app.log"
# startup_position = "head"
# multiline_mode = "none"
"#
    .to_string()
}

pub(super) fn expand_env_contract(
    mut config: AgentConfigContract,
) -> Result<AgentConfigContract, ConfigError> {
    config.agent.agent_id = expand_optional(config.agent.agent_id)?;
    config.agent.environment_id = expand_optional(config.agent.environment_id)?;
    config.agent.instance_name = expand_optional(config.agent.instance_name)?;
    config.control_plane.endpoint = expand_optional(config.control_plane.endpoint)?;
    config.control_plane.tls_mode = expand_optional(config.control_plane.tls_mode)?;
    config.control_plane.auth_mode = expand_optional(config.control_plane.auth_mode)?;
    config.paths.root_dir = expand_string(config.paths.root_dir)?;
    config.paths.run_dir = expand_string(config.paths.run_dir)?;
    config.paths.state_dir = expand_string(config.paths.state_dir)?;
    config.paths.log_dir = expand_string(config.paths.log_dir)?;
    config.telemetry.logs.spool_dir = expand_string(config.telemetry.logs.spool_dir)?;
    config.telemetry.logs.output_file = expand_string(config.telemetry.logs.output_file)?;
    for input in &mut config.telemetry.logs.file_inputs {
        input.input_id = expand_string(std::mem::take(&mut input.input_id))?;
        input.path = expand_string(std::mem::take(&mut input.path))?;
        input.startup_position = expand_string(std::mem::take(&mut input.startup_position))?;
        input.multiline_mode = expand_string(std::mem::take(&mut input.multiline_mode))?;
    }
    Ok(config)
}

pub(super) fn resolve_paths(
    mut config: AgentConfigContract,
    config_path: &Path,
) -> AgentConfigContract {
    let config_dir = config_path.parent().unwrap_or_else(|| Path::new("."));
    let root_dir = absolutize(config_dir, &config.paths.root_dir);

    config.paths.root_dir = root_dir.display().to_string();
    config.paths.run_dir = absolutize(&root_dir, &config.paths.run_dir)
        .display()
        .to_string();
    config.paths.state_dir = absolutize(&root_dir, &config.paths.state_dir)
        .display()
        .to_string();
    config.paths.log_dir = absolutize(&root_dir, &config.paths.log_dir)
        .display()
        .to_string();
    config.telemetry.logs.spool_dir = absolutize(&root_dir, &config.telemetry.logs.spool_dir)
        .display()
        .to_string();
    config.telemetry.logs.output_file = absolutize(&root_dir, &config.telemetry.logs.output_file)
        .display()
        .to_string();
    config.telemetry.logs.file_inputs = config
        .telemetry
        .logs
        .file_inputs
        .into_iter()
        .map(|mut input| {
            input.path = absolutize(&root_dir, &input.path).display().to_string();
            input
        })
        .collect();
    config
}

fn absolutize(base: &Path, raw: &str) -> PathBuf {
    let path = Path::new(raw);
    let joined = if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    };
    normalize_path(joined)
}

fn normalize_path(path: PathBuf) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}

fn expand_optional(value: Option<String>) -> Result<Option<String>, ConfigError> {
    value.map(expand_string).transpose()
}

fn expand_string(value: String) -> Result<String, ConfigError> {
    let mut out = String::with_capacity(value.len());
    let mut cursor = 0usize;
    while let Some(start) = value[cursor..].find("${") {
        let start = cursor + start;
        out.push_str(&value[cursor..start]);
        let rest = &value[start + 2..];
        let Some(end_rel) = rest.find('}') else {
            out.push_str(&value[start..]);
            return Ok(out);
        };
        let end = start + 2 + end_rel;
        let name = &value[start + 2..end];
        let expanded = env::var(name).map_err(|_| ConfigError::MissingEnvVar(name.to_string()))?;
        out.push_str(&expanded);
        cursor = end + 1;
    }
    out.push_str(&value[cursor..]);
    Ok(out)
}
