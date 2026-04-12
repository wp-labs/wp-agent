//! Versioned contract objects shared by edge and center components.

pub mod action_plan;
pub mod action_result;
pub mod agent_config;
pub mod capability_report;
pub mod gateway;
pub mod state_exec;
pub mod state_logs;

pub const API_VERSION_V1ALPHA1: &str = "v1alpha1";
pub const SCHEMA_VERSION_V1ALPHA1: &str = "v1alpha1";
