#[cfg(windows)]
mod windows;

#[cfg(windows)]
use self::windows::RedirectInner;
#[cfg(windows)]
pub use self::windows::{AsRawFile, RawFile};

#[cfg(unix)]
mod unix;

#[cfg(unix)]
use self::unix::RedirectInner;
#[cfg(unix)]
pub use self::unix::{AsRawFile, RawFile};

use std::io;
use std::any::Any;

pub struct RedirectError<F> {
    pub error: io::Error,
    pub file: F,
}

impl<F> From<RedirectError<F>> for io::Error {
    fn from(err: RedirectError<F>) -> io::Error {
        err.error
    }
}

impl<F: Any> ::std::error::Error for RedirectError<F> {
    fn description(&self) -> &str {
        self.error.description()
    }
    fn cause(&self) -> Option<&::std::error::Error> {
        Some(&self.error)
    }
}

impl<F> ::std::fmt::Display for RedirectError<F> {
    fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        self.error.fmt(fmt)
    }
}

impl<F> ::std::fmt::Debug for RedirectError<F> {
    fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        self.error.fmt(fmt)
    }
}

/// Redirect stderr/stdout to a file.
pub struct Redirect<F> {
    #[allow(dead_code)]
    inner: RedirectInner,
    file: F,
}

impl<F> Redirect<F> where F: AsRawFile {
    fn make(is_stderr: bool, file: F) -> Result<Self, RedirectError<F>> {
        let inner = match RedirectInner::make(is_stderr, file.as_raw_file()) {
            Ok(inner) => inner,
            Err(e) => return Err(RedirectError { error: e, file: file})
        };
        Ok(Redirect {
            inner: inner,
            file: file,
        })
    }
    /// Redirect stdout to `file`.
    pub fn stdout(file: F) -> Result<Self, RedirectError<F>> {
        Redirect::make(false, file)
    }
    /// Redirect stderr to `file`.
    pub fn stderr(file: F) -> Result<Self, RedirectError<F>> {
        Redirect::make(true, file)
    }

    /// Extract inner file object.
    pub fn into_inner(self) -> F {
        self.file
    }
}

