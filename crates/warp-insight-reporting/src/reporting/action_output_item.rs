// @jumo generated
// @jumo hash=ce5a7af2683d15db

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Reporting", module = "Reporting.Contract")]
pub struct ActionOutputItem {
    pub value: String,
    pub redacted: String,
    pub name: String,
}
