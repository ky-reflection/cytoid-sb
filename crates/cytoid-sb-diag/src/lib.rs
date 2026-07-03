use miette::Diagnostic;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum SbError {
    #[error("failed to read {path}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("invalid storyboard JSON in {path}")]
    JsonParse {
        path: String,
        #[source]
        source: serde_json::Error,
    },

    #[error("validation failed with {count} issue(s)")]
    Validation { count: usize },

    #[error("{message}")]
    Other { message: String },

    #[error("compile not implemented for {kind} inputs yet")]
    UnsupportedInput { kind: String },
}

pub type SbResult<T> = Result<T, SbError>;

pub fn validation_error(count: usize) -> SbError {
    SbError::Validation { count }
}
