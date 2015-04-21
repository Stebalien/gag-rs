use ::winapi;
use std::io;
use std::os::windows::io::{RawHandle, AsRawHandle};
use std::sync::atomic::{Ordering, AtomicBool, ATOMIC_BOOL_INIT};

extern "system" {
    pub fn SetStdHandle(nStdHandle: winapi::DWORD, hHandle: winapi::HANDLE) -> winapi::BOOL;
    pub fn GetStdHandle(nStdHandle: winapi::DWORD) -> winapi::HANDLE;
}

/// Os independent type alias for raw file descriptors/handles.
pub type RawFile = RawFd;

/// Os independent trait for getting the raw descriptor/handle of a file.
pub trait AsRawFile: AsRawHandle {
    fn as_raw_file(&self) -> RawFile;
}
impl<T> AsRawFile for T where T: AsRawHandle {
    fn as_raw_file(&self) -> RawFile {
        self.as_raw_handle()
    }
}

pub struct RedirectInner {
    is_stderr: bool,
    std_handle_backup: RawHandle,
}

static REDIRECT_FLAGS: [AtomicBool; 2] = [ATOMIC_BOOL_INIT, ATOMIC_BOOL_INIT];

fn get_std_handle(is_stderr: bool) -> RawFd {
    if is_stderr { winapi::STD_ERROR_HANDLE } else { winapi::STD_OUTPUT_HANDLE }
}


impl RedirectInner {
    fn make(is_stderr: bool, file_handle: RawFile) -> io::Result<RedirectInner> {
        let std_handle = get_std_handle(is_stderr);

        let std_handle_backup = match unsafe { GetStdHandle(std) } {
            winapi::INVALID_HANDLE_VALUE => return Err(io::Error::last_os_error()),
            0 => return Err(io::Error::new(io::ErrorKind::Other, "Terminal not connected")),
            other => other,
        };

        if REDIRECT_FLAGS[is_stderr as usize].fetch_or(true, Ordering::Relaxed) {
            return Err(io::Error::new(io::ErrorKind::AlreadyExists, "Redirect already exists."));
        }

        // Dropping this will close std_fd_dup
        let inner = RedirectInner {
            is_stderr: is_stderr,
            std_handle_backup: std_handle_backup,
        };

        if unsafe { SetStdHandle(std_handle, file_handle) } {
            Ok(fds)
        } else {
            // Drop is still correct even if this doesn't succeed. 
            Err(io::Error::last_os_error())
        }
    }
}

impl Drop for RedirectInner {
    fn drop(&mut self) {
        unsafe {
            let std_handle = get_std_handle(self.is_stderr);
            // Check for errors?
            SetStdHandle(std_handle, self.std_handle_backup);
            REDIRECT_FLAGS[self.is_stderr as usize].store(false, Ordering::Relaxed);
        }
    }
}

