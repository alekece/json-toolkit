use thiserror::Error;

/// Any error that may occur when using this crate.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum Error {
    #[error("JSON pointer must start with a leading '/' if not empty")]
    MissingLeadingBackslash,
    #[error("unsupported JSON value insertion")]
    UnsupportedInsertion,
    #[error("JSON key not found")]
    KeyNotFound,
}
