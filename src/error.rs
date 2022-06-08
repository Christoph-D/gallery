//! Error helpers.
use anyhow::anyhow;
use std::path::Path;

/// Creates a new error with the given message and path.
pub fn path_error(msg: &str, path: &Path) -> anyhow::Error {
    anyhow!("{}: {}", msg, path.to_string_lossy())
}

pub trait PathErrorContext<T, E> {
    /// Adds context to an existing error.
    fn path_context(self, msg: &str, path: &Path) -> anyhow::Result<T>;
}

impl<T, E, C> PathErrorContext<T, E> for C
where
    C: anyhow::Context<T, E>,
{
    fn path_context(self, msg: &str, path: &Path) -> anyhow::Result<T> {
        self.with_context(|| format!("{}: \"{}\"", msg, path.to_string_lossy()))
    }
}
