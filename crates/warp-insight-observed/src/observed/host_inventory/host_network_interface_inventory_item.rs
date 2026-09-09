// @jumo generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Observed", module = "Observed.HostInventory")]
pub struct HostNetworkInterfaceInventoryItem {
    pub name: String,
    pub mac_addr: Option<String>,
    pub ip_addresses: Vec<String>,
    pub mtu: Option<i64>,
}
