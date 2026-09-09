// @jumo generated
// @jumo hash=935e9782936adab0

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Reporting", module = "Reporting.Contract")]
pub struct CapabilityReportSections {
    pub exec: String,
    pub logs: String,
    pub upgrade: String,
    pub metrics: String,
    pub limits: String,
}
