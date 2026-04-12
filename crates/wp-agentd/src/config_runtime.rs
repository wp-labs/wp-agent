//! Runtime config loading and mode selection.

use std::path::PathBuf;

use wp_agent_contracts::agent_config::{
    AgentConfigContract, AgentSection, ControlPlaneSection, PathsSection,
};

pub fn load_default(root_dir: PathBuf) -> AgentConfigContract {
    AgentConfigContract {
        schema_version: "v1alpha1".to_string(),
        agent: AgentSection {
            agent_id: None,
            environment_id: None,
            instance_name: Some("local".to_string()),
        },
        control_plane: ControlPlaneSection {
            enabled: false,
            endpoint: None,
        },
        paths: PathsSection {
            run_dir: root_dir.join("run").display().to_string(),
            state_dir: root_dir.join("state").display().to_string(),
            log_dir: root_dir.join("log").display().to_string(),
            root_dir: root_dir.display().to_string(),
        },
    }
}
