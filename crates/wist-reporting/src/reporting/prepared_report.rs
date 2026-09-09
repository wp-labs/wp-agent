// @jumo generated
// @jumo hash=0d635dd514b820fe

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Reporting", module = "Reporting.Pipeline")]
pub struct PreparedReport {
    pub origin: String,
    pub report: String,
}
