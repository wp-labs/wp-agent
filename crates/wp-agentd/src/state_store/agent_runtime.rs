//! `agent_runtime.json` store.

use wp_agent_contracts::state_exec::AgentRuntimeState;

pub fn load_default() -> AgentRuntimeState {
    AgentRuntimeState {
        schema_version: "v1alpha1".to_string(),
        agent_id: "local-agent".to_string(),
        instance_id: "local-instance".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        mode: wp_agent_contracts::state_exec::RuntimeMode::Normal,
        updated_at: "1970-01-01T00:00:00Z".to_string(),
    }
}
