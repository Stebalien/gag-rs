use ::libc;
use std::io;
use std::os::unix::io::{RawFd, AsRawFd};
use std::sync::atomic::{Ordering, AtomicBool, ATOMIC_BOOL_INIT};

/// Os independent type alias for raw file descriptors/handles.
pub type RawFile = RawFd;

/// Os independent trait for getting the raw descriptor/handle of a file.
pub trait AsRawFile: AsRawFd {
    fn as_raw_file(&self) -> RawFile;
}

impl<T> AsRawFile for T where T: AsRawFd {
    fn as_raw_file(&self) -> RawFile {
        self.as_raw_fd()
    }
}

pub struct RedirectInner {
    is_stderr: bool,
    std_fd_backup: RawFd,
}

static REDIRECT_FLAGS: [AtomicBool; 2] = [ATOMIC_BOOL_INIT, ATOMIC_BOOL_INIT];

fn get_std_fd(is_stderr: bool) -> RawFd {
    if is_stderr { libc::STDERR_FILENO } else { libc::STDOUT_FILENO }
}

impl RedirectInner {
    pub fn make(is_stderr: bool, file_fd: RawFile) -> io::Result<RedirectInner> {
        let std_fd = get_std_fd(is_stderr);

        let std_fd_backup = unsafe { libc::dup(std_fd) };
        if std_fd_backup < 0 {
            return Err(io::Error::last_os_error());
        }

        // If this ends up panicing, something is seriously wrong. Regardless, we will only end up
        // leaking a single file descriptor so it's not the end of the world.
        if REDIRECT_FLAGS[is_stderr as usize].fetch_or(true, Ordering::Relaxed) {
            unsafe { libc::close(std_fd_backup); }
            return Err(io::Error::new(io::ErrorKind::AlreadyExists, "Redirect already exists."));
        }

        // Dropping this will close std_fd_backup
        let fds = RedirectInner {
            is_stderr: is_stderr,
            std_fd_backup: std_fd_backup as RawFd,
        };

        match unsafe { libc::dup2(file_fd, std_fd) } {
            // Drop is still correct even if this doesn't succeed. 
            -1 => Err(io::Error::last_os_error()),
            _ => Ok(fds),
        }
    }
}

impl Drop for RedirectInner {
    fn drop(&mut self) {
        unsafe {
            let std_fd = get_std_fd(self.is_stderr);
            // Check for errors?
            libc::dup2(self.std_fd_backup, std_fd);
            libc::close(self.std_fd_backup);
            REDIRECT_FLAGS[self.is_stderr as usize].store(false, Ordering::Relaxed);
        }
    }
}

