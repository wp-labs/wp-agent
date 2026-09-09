// @jumo generated
// @jumo hash=dc6752475e7ec954

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Control", module = "Control.Agent.Enrollment")]
pub struct ManagementEndpointTrustBundle {
    pub control_endpoint: String,
    pub expires_at: crate::control::types::DateTime,
    #[jumo(unique)]
    pub trust_bundle_id: String,
    pub ca_bundle: String,
    pub expected_san: String,
    pub issued_at: crate::control::types::DateTime,
    pub server_name: String,
}
