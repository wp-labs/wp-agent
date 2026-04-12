//! Static validators for contracts, config, and local state.

pub mod action_plan;
pub mod action_result;
pub mod config;
pub mod state;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub code: &'static str,
}

impl ValidationError {
    pub const fn new(code: &'static str) -> Self {
        Self { code }
    }
}
