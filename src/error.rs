//! Error and diagnostic plumbing.
//!
//! Two layers, deliberately:
//!
//! * [`Diagnostic`] — a *located, coded* message about the source
//!   (the user's tzdata). These are what a tzdata author wants to read.
//! * [`Error`] — the crate's `Result` error. It either wraps a diagnostic (most parse and
//!   compile failures carry one) or describes an environmental failure (I/O, an unusable
//!   configuration, a failed oracle invocation).

use std::path::{Path, PathBuf};

use crate::diagnostics::Diagnostic;

/// Crate-wide result type.
pub type Result<T> = std::result::Result<T, Error>;

/// The crate's top-level error.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A located, coded problem with the source or the requested compilation.
    #[error("{0}")]
    Diagnostic(Box<Diagnostic>),

    /// An I/O failure against a specific path.
    #[error("{path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// A configuration problem (e.g. missing `--out`).
    #[error("{0}")]
    Config(String),

    /// A free-form internal error that is not tied to a source location.
    #[error("{0}")]
    Message(String),
}

impl Error {
    /// Wrap a path-tagged I/O error.
    pub fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Error::Io {
            path: path.into(),
            source,
        }
    }

    /// Build a configuration error.
    pub fn config(msg: impl Into<String>) -> Self {
        Error::Config(msg.into())
    }

    /// Build a free-form error.
    pub fn message(msg: impl Into<String>) -> Self {
        Error::Message(msg.into())
    }

    /// Borrow the wrapped diagnostic, if this is one.
    pub fn diagnostic(&self) -> Option<&Diagnostic> {
        match self {
            Error::Diagnostic(d) => Some(d),
            _ => None,
        }
    }
}

impl From<Diagnostic> for Error {
    fn from(d: Diagnostic) -> Self {
        Error::Diagnostic(Box::new(d))
    }
}

/// Helper for I/O against a borrowed path.
pub(crate) fn io_at<T>(path: &Path, r: std::io::Result<T>) -> Result<T> {
    r.map_err(|e| Error::io(path, e))
}
