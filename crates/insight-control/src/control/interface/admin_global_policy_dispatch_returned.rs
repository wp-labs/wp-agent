// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "message", role = "response", domain = "Control", module = "Control.AdminFacingInterface")]
pub struct AdminGlobalPolicyDispatchReturned {
    pub dispatch: crate::GlobalPolicyDispatch,
}
