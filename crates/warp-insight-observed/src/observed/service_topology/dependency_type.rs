// @moju generated
#[derive(Debug, Clone, Copy, PartialEq, Eq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "variant", domain = "Observed", module = "Observed.ServiceTopology")]
pub enum DependencyType {
    SyncRpc,
    AsyncMq,
    Database,
    Cache,
    ExternalApi,
}
