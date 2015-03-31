use ::libc;
use ::std;

use std::os::unix::io::{Fd, AsRawFd};
use std::io;

/// Private helper trait because AsRawFd is not implemented on &mut T.
trait MyRawFd {
    fn my_raw_fd(&self) -> Fd;
}

impl<T: AsRawFd> MyRawFd for T {
    fn my_raw_fd(&self) -> Fd {
        self.as_raw_fd()
    }
}

impl<'a, T: AsRawFd> MyRawFd for &'a mut T {
    fn my_raw_fd(&self) -> Fd {
        self.as_raw_fd()
    }
}

impl MyRawFd for Fd {
    fn my_raw_fd(&self) -> Fd {
        *self
    }
}

/// Redirect stderr/stdout to a file.
pub struct Redirect<F> {
    std_fd: Fd,
    std_fd_dup: Fd,
    file: F,
}

impl<F> Redirect<F> {
    unsafe fn close(&mut self) {
        // Check for errors?
        libc::dup2(self.std_fd_dup, self.std_fd);
        libc::close(self.std_fd_dup);
    }
}

impl<F> Redirect<F> where F: MyRawFd {
    fn make(std_fd: Fd, file: F) -> io::Result<Self> {
        let file_fd = file.my_raw_fd();
        // BEGIN: Don't panic
        let std_fd_dup = unsafe { libc::dup(std_fd) };
        if std_fd_dup < 0 {
            return Err(io::Error::last_os_error())
        }
        let gag = Redirect {
            std_fd: std_fd,
            std_fd_dup: std_fd_dup as Fd,
            file: file,
        };
        // END: Don't panic.
        if unsafe { libc::dup2(file_fd, std_fd) } < 0 {
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

#[unsafe_destructor]
impl<F> Drop for Redirect<F> {
    fn drop(&mut self) {
        unsafe {
            self.close();
        }
    }
}
