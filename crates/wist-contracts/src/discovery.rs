//! Discovery runtime contract types shared by edge modules.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::SCHEMA_VERSION_V1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiscoverySnapshotContract {
    pub schema_version: String,
    pub snapshot_id: String,
    pub revision: u64,
    pub generated_at: String,
    #[serde(default)]
    pub origins: Vec<DiscoveryOrigin>,
    #[serde(default)]
    pub resources: Vec<DiscoveredResource>,
    #[serde(default)]
    pub targets: Vec<DiscoveredTarget>,
}

impl DiscoverySnapshotContract {
    pub fn new(snapshot_id: String, revision: u64, generated_at: String) -> Self {
        Self {
            schema_version: SCHEMA_VERSION_V1.to_string(),
            snapshot_id,
            revision,
            generated_at,
            origins: Vec::new(),
            resources: Vec::new(),
            targets: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiscoveryOrigin {
    pub origin_id: String,
    pub probe: String,
    pub source: String,
    pub observed_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiscoveryCacheMeta {
    pub schema_version: String,
    pub snapshot_id: String,
    pub revision: u64,
    pub generated_at: String,
    #[serde(default)]
    pub origins: Vec<DiscoveryOrigin>,
    pub last_success_at: Option<String>,
    pub last_error: Option<String>,
}

impl DiscoveryCacheMeta {
    pub fn new(
        snapshot_id: String,
        revision: u64,
        generated_at: String,
        origins: Vec<DiscoveryOrigin>,
        last_success_at: Option<String>,
        last_error: Option<String>,
    ) -> Self {
        Self {
            schema_version: SCHEMA_VERSION_V1.to_string(),
            snapshot_id,
            revision,
            generated_at,
            origins,
            last_success_at,
            last_error,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiscoveredResource {
    pub resource_id: String,
    pub kind: String,
    pub origin_idx: usize,
    #[serde(default)]
    pub attributes: BTreeMap<String, String>,
    /// When this resource was first discovered (RFC3339 UTC).
    #[serde(default)]
    pub discovered_at: String,
    /// When this resource was last confirmed present (RFC3339 UTC).
    #[serde(default)]
    pub last_seen_at: String,
    /// Health status from the discovery runtime's perspective.
    #[serde(default = "default_health")]
    pub health: String,
    /// Name of the probe that discovered this resource, e.g. "local_runtime".
    #[serde(default)]
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiscoveredTarget {
    pub target_id: String,
    pub kind: String,
    pub origin_idx: usize,
    pub resource_ref: String,
    #[serde(default)]
    pub execution_hints: BTreeMap<String, String>,
    /// Target state from the discovery runtime's perspective.
    #[serde(default = "default_target_state")]
    pub state: String,
}

fn default_health() -> String {
    "unknown".to_string()
}

fn default_target_state() -> String {
    "active".to_string()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CandidateCollectionTarget {
    pub candidate_id: String,
    pub target_ref: String,
    pub collection_kind: String,
    pub resource_ref: String,
    #[serde(default)]
    pub execution_hints: Vec<StringKeyValue>,
    pub generated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StringKeyValue {
    pub key: String,
    pub value: String,
}

impl StringKeyValue {
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}
