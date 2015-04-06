use ::libc;
use ::std;

use std::os::unix::io::{RawFd, AsRawFd};
use std::io;
use std::sync::atomic::{Ordering, AtomicBool, ATOMIC_BOOL_INIT};

static REDIRECT_FLAGS: [AtomicBool; 2] = [ATOMIC_BOOL_INIT, ATOMIC_BOOL_INIT];

/// Redirect stderr/stdout to a file.
pub struct Redirect<F> {
    std_fd: RawFd,
    std_fd_dup: RawFd,
    file: F,
}

impl<F> Redirect<F> {
    unsafe fn close(&mut self) {
        // Check for errors?
        libc::dup2(self.std_fd_dup, self.std_fd);
        libc::close(self.std_fd_dup);
        REDIRECT_FLAGS[self.std_fd as usize].store(false, Ordering::Relaxed);
    }
}

impl<F> Redirect<F> where F: AsRawFd {
    fn make(std_fd: RawFd, file: F) -> io::Result<Self> {
        let file_fd = file.as_raw_fd();
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
        let gag = Redirect {
            std_fd: std_fd,
            std_fd_dup: std_fd_dup as RawFd,
            file: file,
        };

        if unsafe { libc::dup2(file_fd, std_fd) } < 0 {
            // Drop is still correct even if this doesn't succeed. 
            return Err(io::Error::last_os_error())
        }
        Ok(gag)
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
    pub fn into_inner(mut self) -> F {
        unsafe {
            // Is this safe?
            self.close();
            let file = std::mem::transmute_copy(&self.file);
            std::mem::forget(self);
            file
        }
    }

}

impl<F> Drop for Redirect<F> {
    fn drop(&mut self) {
        unsafe {
            self.close();
        }
    }
}
