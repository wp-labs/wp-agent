// @moju generated
// @moju hash=764a69fcf1d71142

pub mod api;
pub mod app;
pub mod control;
pub mod reporting;
pub mod infra;

pub type AppError = Box<dyn std::error::Error + Send + Sync>;
