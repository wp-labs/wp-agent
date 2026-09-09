// @jumo generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Control", module = "Control.Gateway.Supervision")]
pub struct GatewayCustomerBinding {
    pub gateway_id: String,
    pub customer_id: String,
    pub status: String,
    pub bound_at: crate::DateTime,
}
