// @jumo generated
// @jumo hash=8bfac24d3ba64827

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Reporting", module = "Reporting.Pipeline")]
pub struct ExportResult {
    pub items: String,
    pub errors: String,
    pub kind: String,
}
