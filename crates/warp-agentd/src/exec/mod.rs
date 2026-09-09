//! 动作执行域：本地执行、子进程生命周期控制、隔离与恢复。

pub mod execution_support;
pub mod local_exec;
pub mod planner_bridge;
pub mod process_control;
pub mod quarantine;
pub mod recovery;
