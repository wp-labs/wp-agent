// @jumo generated
// @jumo hash=1ddfc8e51bef1161

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Reporting", module = "Reporting.Contract")]
pub struct ActionPlanContract {
    pub target: String,
    pub plan_meta: String,
    pub constraints: String,
    pub program: String,
    pub api_version: String,
    pub kind: String,
}
