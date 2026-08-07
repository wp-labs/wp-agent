// @moju generated
#[derive(Debug, Clone, Copy, PartialEq, Eq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "variant", domain = "Observed", module = "Observed.HostResponsibility")]
pub enum AssignmentRole {
    Owner,
    Maintainer,
    Operator,
    Oncall,
    SecurityOwner,
}
