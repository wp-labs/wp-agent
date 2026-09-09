// @jumo generated
// @jumo hash=c7fb3e88f7c747e0

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Reporting", module = "Reporting.Contract")]
pub struct ActionPlanStep {
    pub op: String,
    #[jumo(unique)]
    pub id: String,
    pub kind: String,
}
