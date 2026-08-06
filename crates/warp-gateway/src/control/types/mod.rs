// @moju generated
// @moju hash=decaea78429c5f4f

pub type UserId = String;
pub type OrderId = String;
pub type CartId = String;
pub type PaymentIntentId = String;
pub type Money = i64;
pub type Sku = String;
pub type Quantity = i64;
pub type Address = String;
pub type CartItemList = String;
pub type Int = i64;
pub type Bool = bool;
pub type Float = f64;
pub type AgentCredentialVerificationStatus = String;
pub type AgentEnrollmentResultStatus = String;
pub type AgentEnrollmentTokenStatus = String;
pub type AgentEnrollmentTokenValidationStatus = String;
pub type AgentIdentityStatus = String;
pub type HealthState = String;

#[derive(
    Clone, PartialEq, Eq, PartialOrd, Ord, ::serde::Serialize, ::serde::Deserialize,
)]
pub struct DateTime(chrono::DateTime<chrono::Utc>);

impl DateTime {
    pub fn now() -> Self {
        Self(chrono::Utc::now())
    }

    pub fn from_rfc3339(value: &str) -> Option<Self> {
        chrono::DateTime::parse_from_rfc3339(value)
            .ok()
            .map(|value| Self(value.with_timezone(&chrono::Utc)))
    }

    pub fn seconds_until(&self, later: &Self) -> i64 {
        later.0.signed_duration_since(self.0).num_seconds().max(0)
    }
}

impl std::fmt::Debug for DateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::fmt::Display for DateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct Secret(String);

impl std::fmt::Debug for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Secret(***)")
    }
}

impl std::fmt::Display for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("***")
    }
}

// Control/其他模型模块 re-export（shim）
pub use crate::control::enrollment::*;
pub use crate::control::identity::*;
pub use crate::control::status::*;
pub use crate::control::command::*;
pub use crate::control::registry::*;
pub use crate::control::protocol::*;
pub use crate::control::interface::*;
pub use crate::reporting::*;
