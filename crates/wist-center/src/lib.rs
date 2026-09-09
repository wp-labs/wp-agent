// @jumo generated
// WarpInsightCenter 上级聚合控制中心（子系统实现）

pub use insight_control::*;
pub use wist_observed::*;
pub use wist_security::*;
pub use wist_reporting::*;

pub mod api;
pub mod config;
pub mod infra;

pub type AppError = Box<dyn std::error::Error + Send + Sync>;
