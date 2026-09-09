// @jumo generated
// @jumo hash=fde998639e35cff6

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Reporting", module = "Reporting.Pipeline")]
pub struct ExporterSource {
    #[jumo(unique)]
    pub source_id: String,
    pub kind: String,
}
