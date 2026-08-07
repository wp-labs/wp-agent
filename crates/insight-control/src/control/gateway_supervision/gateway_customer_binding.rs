// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "struct", domain = "Control", module = "Control.GatewaySupervision")]
pub struct GatewayCustomerBinding {
    pub gateway_id: String,
    pub customer_id: String,
    pub status: String,
    pub bound_at: crate::DateTime,
}
