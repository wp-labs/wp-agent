//! Shared error-code placeholders.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    Unknown,
    InvalidArgument,
    InvalidState,
    Io,
}
