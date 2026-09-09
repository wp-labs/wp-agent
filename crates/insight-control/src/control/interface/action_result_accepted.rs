// @jumo generated
// @jumo hash=f74bf8b38ab755e5

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "message", role = "response", domain = "Reporting", module = "Reporting.Protocol")]
pub struct ActionResultAccepted {
    pub receipt: warp_insight_reporting::ActionResultReceipt,
}
