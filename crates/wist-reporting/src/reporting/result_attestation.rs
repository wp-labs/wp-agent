// @jumo generated
// @jumo hash=426210233649fc5b

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Reporting", module = "Reporting.Protocol")]
pub struct ResultAttestation {
    pub issued_by: String,
    pub attested_at: crate::DateTime,
    pub result_digest: String,
    pub signature: String,
}
