// @jumo generated
// @jumo hash=e39a0b357a3492b2

#[derive(
    Debug, Clone, PartialEq, Eq, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo,
)]
#[jumo(kind = "state", domain = "Reporting")]
pub enum HealthState {
    Healthy,
    Degraded,
    Unhealthy,
}
