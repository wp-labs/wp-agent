// @jumo generated
// @jumo hash=da7ecaf505afe2dd

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Control", module = "Control.Agent.Identity")]
pub struct AgentHostProfile {
    pub cloud_instance_id: String,
    #[jumo(unique)]
    pub node_id: String,
    pub hostname: String,
    pub os: String,
    pub machine_id: String,
    pub arch: String,
    pub k8s_node_uid: String,
    pub ip_addresses: Vec<String>,
}
