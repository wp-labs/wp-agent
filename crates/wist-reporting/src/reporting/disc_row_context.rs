// @jumo generated
// @jumo hash=22828a9192739665

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Reporting", module = "Reporting.Pipeline")]
pub struct DiscRowContext {
    pub resource: String,
    pub target: String,
    pub candidate: String,
}
