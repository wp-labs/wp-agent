//! 文件型日志采集运行时（file input ingestion）。

pub mod file;
pub mod file_reader;
pub mod file_watcher;

pub use file::{FileInputConfig, FileInputProcessor, ProcessOutcome};
