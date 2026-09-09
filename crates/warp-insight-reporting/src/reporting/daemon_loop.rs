// @jumo generated
// @jumo hash=aab56236cb440d6c

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Reporting", module = "Reporting.Pipeline")]
pub struct DaemonLoop {
    pub exec_bin: String,
    pub config: String,
}
