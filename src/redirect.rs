use libc;

use std::any::Any;
use std::io;
use std::os::unix::io::{AsRawFd, RawFd};
use std::sync::atomic::{AtomicBool, Ordering};

static REDIRECT_FLAGS: [AtomicBool; 3] = [
    AtomicBool::new(false),
    AtomicBool::new(false),
    AtomicBool::new(false),
];

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
    fds: RedirectFds,
    file: F,
}

// Separate struct so we can destruct the redirect and drop the file descriptors.
struct RedirectFds {
    std_fd: RawFd,
    std_fd_dup: RawFd,
}

impl RedirectFds {
    fn make(std_fd: RawFd, file_fd: RawFd) -> io::Result<RedirectFds> {
        let std_fd_dup = unsafe { libc::dup(std_fd) };
        if std_fd_dup < 0 {
            return Err(io::Error::last_os_error());
        }

        // If this ends up panicing, something is seriously wrong. Regardless, we will only end up
        // leaking a single file descriptor so it's not the end of the world.
        if REDIRECT_FLAGS[std_fd as usize].fetch_or(true, Ordering::Relaxed) {
            unsafe {
                libc::close(std_fd_dup);
            }
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "Redirect already exists.",
            ));
        }

        // Dropping this will close std_fd_dup
        let fds = RedirectFds { std_fd, std_fd_dup };

        match unsafe { libc::dup2(file_fd, std_fd) } {
            // Drop is still correct even if this doesn't succeed.
            -1 => Err(io::Error::last_os_error()),
            _ => Ok(fds),
        }
    }
}

impl Drop for RedirectFds {
    fn drop(&mut self) {
        unsafe {
            // Check for errors?
            libc::dup2(self.std_fd_dup, self.std_fd);
            libc::close(self.std_fd_dup);
            REDIRECT_FLAGS[self.std_fd as usize].store(false, Ordering::Relaxed);
        }
    }
}

impl<F> Redirect<F>
where
    F: AsRawFd,
{
    fn make(std_fd: RawFd, file: F) -> Result<Self, RedirectError<F>> {
        let fds = match RedirectFds::make(std_fd, file.as_raw_fd()) {
            Ok(fds) => fds,
            Err(error) => return Err(RedirectError { error, file }),
        };
        Ok(Redirect { fds, file })
    }
    /// Redirect stdout to `file`.
    pub fn stdout(file: F) -> Result<Self, RedirectError<F>> {
        Redirect::make(libc::STDOUT_FILENO, file)
    }
    /// Redirect stderr to `file`.
    pub fn stderr(file: F) -> Result<Self, RedirectError<F>> {
        Redirect::make(libc::STDERR_FILENO, file)
    }

    /// Extract inner file object.
    pub fn into_inner(self) -> F {
        self.file
    }
}
