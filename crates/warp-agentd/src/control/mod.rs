//! 控制面：agent 注册（enrollment）、运行入口与能力上报。

pub mod capability_report;
pub mod enrollment;
mod runtime_entry;

pub(crate) use runtime_entry::run;
