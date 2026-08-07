// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "struct", domain = "Control", module = "Control.GatewaySupervision")]
pub struct GatewayInitialConfig {
    pub control_center_endpoint: String,
    pub policy_version: String,
    pub telemetry_output: String,
}
