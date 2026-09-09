//! Unified output envelope for warp-insight data exchange.
//!
//! All external-facing output (discovery snapshots, metrics batches,
//! and future event batches) uses this envelope. The envelope provides
//! source identity, idempotency fields (`output_id` + `seq`), and
//! kind-based payload routing for downstream consumers (warp-parse ETL).

use serde::{Deserialize, Serialize};

/// Unified output envelope wrapping any payload type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExporterOutput<T> {
    /// Schema identifier for the envelope itself.
    /// Target spec value: "warp-insight/v1".
    pub api_version: String,
    /// Payload type discriminator: "disc_snap" / "metrics".
    pub kind: String,
    /// Globally unique output identifier.
    /// Format: `<agent_id>_<seq>` or `<seq>` when agent_id is unavailable.
    pub output_id: String,
    /// Monotonically increasing sequence number within daemon lifetime.
    pub seq: u64,
    /// When this output was generated (RFC3339 UTC).
    pub generated_at: String,
    /// Source agent identity.
    pub source: ExporterSource,
    /// Kind-specific payload.
    pub payload: T,
}

/// Source agent identity within the envelope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExporterSource {
    /// Logical agent identity, stable across instances.
    pub agent_id: String,
    /// Current daemon run instance, for upgrade/replacement tracking.
    pub instance_id: String,
    /// Probe kind that produced this output, e.g. "host", "process", "container".
    /// Present for disc_snap outputs; absent for metrics outputs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub probe: Option<String>,
}

impl ExporterSource {
    pub fn new(agent_id: &str, instance_id: &str) -> Self {
        Self {
            agent_id: agent_id.to_string(),
            instance_id: instance_id.to_string(),
            probe: None,
        }
    }

    pub fn with_probe(mut self, probe: &str) -> Self {
        self.probe = Some(probe.to_string());
        self
    }
}

impl<T> ExporterOutput<T> {
    pub fn new(
        kind: &str,
        output_id: String,
        seq: u64,
        generated_at: String,
        source: ExporterSource,
        payload: T,
    ) -> Self {
        Self {
            api_version: "warp-insight/v1".to_string(),
            kind: kind.to_string(),
            output_id,
            seq,
            generated_at,
            source,
            payload,
        }
    }
}

pub const EXPORTER_API_VERSION: &str = "warp-insight/v1";
