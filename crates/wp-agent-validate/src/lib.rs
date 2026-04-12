//! Static validators for contracts, config, and local state.

pub mod action_plan;
pub mod action_result;
pub mod config;
pub mod state;

use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub code: &'static str,
}

impl ValidationError {
    pub const fn new(code: &'static str) -> Self {
        Self { code }
    }
}

pub(crate) fn require_non_empty(value: &str, code: &'static str) -> Result<(), ValidationError> {
    if value.trim().is_empty() {
        return Err(ValidationError::new(code));
    }
    Ok(())
}

pub(crate) fn parse_rfc3339(
    value: &str,
    code: &'static str,
) -> Result<OffsetDateTime, ValidationError> {
    OffsetDateTime::parse(value, &Rfc3339).map_err(|_| ValidationError::new(code))
}
