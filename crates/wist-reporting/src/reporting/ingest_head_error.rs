// @jumo generated
// @jumo hash=b270264e12073c5d

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "event", domain = "Reporting", module = "Reporting.Protocol")]
pub struct IngestHeadError {
    pub reason: String,
    pub detail: String,
}
