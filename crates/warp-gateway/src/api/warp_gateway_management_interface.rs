// @moju generated
// @moju hash=5cd0b68d5b14ca27

use crate::control::AdminShowAgentRuntimeStatus;

#[derive(::moju_derive::MoJu)]
#[moju(kind = "interface", domain = "Control", module = "Control.UserFacingInterface")]
pub struct WarpGateWayManagementInterface;

impl WarpGateWayManagementInterface {
    pub fn route() -> (&'static str, &'static str) {
        ("GET", "/api/v1/admin/agents/{agent_id}/runtime-status")
    }
}

pub fn handler(_input: AdminShowAgentRuntimeStatus) -> Result<(), crate::AppError> {
    todo!()
}
