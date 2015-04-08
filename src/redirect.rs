use ::libc;

use std::os::unix::io::{RawFd, AsRawFd};
use std::io;
use std::sync::atomic::{Ordering, AtomicBool, ATOMIC_BOOL_INIT};

static REDIRECT_FLAGS: [AtomicBool; 2] = [ATOMIC_BOOL_INIT, ATOMIC_BOOL_INIT];

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
            unsafe { libc::close(std_fd_dup); }
            return Err(io::Error::new(io::ErrorKind::AlreadyExists, "Redirect already exists."));
        }

        // Dropping this will close std_fd_dup
        let fds = RedirectFds {
            std_fd: std_fd,
            std_fd_dup: std_fd_dup as RawFd,
        };

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

impl<F> Redirect<F> where F: AsRawFd {
    fn make(std_fd: RawFd, file: F) -> io::Result<Self> {
        Ok(Redirect {
            fds: try!(RedirectFds::make(std_fd, file.as_raw_fd())),
            file: file,
        })
    }
    /// Redirect stdout to `file`.
    pub fn stdout(file: F) -> io::Result<Self> {
        Redirect::make(libc::STDOUT_FILENO, file)
    }
    /// Redirect stderr to `file`.
    pub fn stderr(file: F) -> io::Result<Self> {
        Redirect::make(libc::STDERR_FILENO, file)
    }

    /// Extract inner file object.
    pub fn into_inner(self) -> F {
        self.file
    }
}

