//! 跨域共用的原始类型（模型标量 DateTime/Secret 的单一真源）。

pub type Int = i64;
pub type Bool = bool;
pub type Float = f64;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, ::serde::Serialize, ::serde::Deserialize)]
pub struct DateTime(chrono::DateTime<chrono::Utc>);

impl DateTime {
    pub fn now() -> Self { Self(chrono::Utc::now()) }
    pub fn from_rfc3339(value: &str) -> Option<Self> {
        chrono::DateTime::parse_from_rfc3339(value).ok().map(|v| Self(v.with_timezone(&chrono::Utc)))
    }
    pub fn seconds_until(&self, later: &Self) -> i64 {
        later.0.signed_duration_since(self.0).num_seconds().max(0)
    }
}

#[derive(Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct Secret(String);

impl std::fmt::Debug for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str("Secret(***)") }
}
