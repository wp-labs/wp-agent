// @jumo generated
// @jumo hash=d9478804747d3476

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Reporting", module = "Reporting.Pipeline")]
pub struct ExporterOutput {
    pub path: String,
    pub kind: String,
}
