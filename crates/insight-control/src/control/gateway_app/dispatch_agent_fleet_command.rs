// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "message", role = "command", domain = "Control", module = "Control.GatewayApp.Application")]
pub struct DispatchAgentFleetCommand {
    pub command_kind: String,
    pub agent_ids: Vec<String>,
    pub requested_by: String,
}
