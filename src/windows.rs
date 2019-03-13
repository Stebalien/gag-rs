//! Gagging on windows targets.
//!
//! The windows gagging api is independent of the `gag` module, and the ergonomics of gagging is different.
//! The gagging functionality is much the same.
//!
//! See [`stdout()`](./fn.stdout.html) and [`stderr()`](./fn.stdout.html) functions for simple examples.
//! The following example demonstrates how to redirect the output, redirecting stdout to stderr.
//!
//! # Example
//! ```rust,ignore
//! use std::io::{Read, Write};
//!
//! std::thread::spawn(move || {
//!		let mut gag = gag::windows::stdout().unwrap();
//! 	let mut stderr = std::io::stderr();
//! 	loop {
//! 		let mut buf = Vec::new();
//! 		gag.read_to_end(&mut buf).unwrap();
//! 		stderr.write_all(&buf).unwrap();
//! 	}
//! });
//!
//! println!("This should be printed on stderr");
//! eprintln!("This will be printed on stderr as well");
//!
//! // This will exit and close the spawned thread.
//! // In most cases you will want to setup a channel and send a break signal to the loop,
//! // and then join the thread back into it once you are finished.
//! ```

#![warn(missing_docs)]

use std::io;
use winapi;
use winapi::ctypes::c_void;
use winapi::shared::{
    minwindef::{DWORD, LPDWORD},
    ntdef::NULL,
};
use winapi::um::{
    handleapi::INVALID_HANDLE_VALUE,
    minwinbase::{OVERLAPPED, SECURITY_ATTRIBUTES},
    winbase::{STD_ERROR_HANDLE, STD_OUTPUT_HANDLE},
    winnt::HANDLE,
};

/// Gag type -- stdout.
pub struct Stdout;
/// Gag type -- stderr.
pub struct Stderr;

/// Holds the gag.
/// Once dropped will return output to the original device.
pub struct Gag<Io> {
    write_handle: HANDLE,
    read_handle: HANDLE,
    original_handle: HANDLE,
    std_device: DWORD,
    mrker: std::marker::PhantomData<Io>,
}

/// Gags the stdout stream.
///
/// # Example
/// ```rust,ignore
/// println!("you will see this");
/// let gag = gag::windows::stdout().unwrap();
/// println!("but not this");
/// drop(gag);
/// println!("and this");
/// ```
pub fn stdout() -> io::Result<Gag<Stdout>> {
    Gag::redirect(STD_OUTPUT_HANDLE)
}

/// Gags the stderr stream.
///
/// # Example
/// ```rust,ignore
/// eprintln!("you will see this");
/// let gag = gag::windows::stderr().unwrap();
/// eprintln!("but not this");
/// drop(gag);
/// eprintln!("and this");
/// ```
pub fn stderr() -> io::Result<Gag<Stderr>> {
    Gag::redirect(STD_ERROR_HANDLE)
}

impl<Io> Gag<Io> {
    fn redirect(std_device: DWORD) -> io::Result<Self> {
        let original_handle = get_std_handle(std_device)?;

        let (read_handle, write_handle) = create_pipe()?;

        set_std_handle(std_device, write_handle)?;

        Ok(Gag {
            write_handle,
            read_handle,
            original_handle,
            std_device,
            mrker: std::marker::PhantomData,
        })
    }
}

/// We can say `Gag` is safe as the `HANDLE`s are meant to last until we close them (with `CloseHandle`).
/// As this is done on the `drop` of `Gag`, the handles should live for `Gag`s lifetime.
unsafe impl<Io> Send for Gag<Io> {}

impl<Io> Drop for Gag<Io> {
    fn drop(&mut self) {
        // failures could potentially leak memory, but should be okay with as they are only HANDLE size.
        unsafe {
            // failure here could disrupt normal printing
            winapi::um::processenv::SetStdHandle(self.std_device, self.original_handle);
        }
        unsafe {
            winapi::um::handleapi::CloseHandle(self.read_handle);
        }
        unsafe {
            winapi::um::handleapi::CloseHandle(self.write_handle);
        }
    }
}

impl<Io> io::Read for Gag<Io> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if !has_bytes(self.read_handle)? {
			return Ok(0);
		}

        let buf_len: DWORD = buf.len() as DWORD;
        let mut bytes_read: DWORD = 0;

        let read_result = unsafe {
            winapi::um::fileapi::ReadFile(
                self.read_handle,
                buf.as_mut_ptr() as *mut c_void,
                buf_len,
                &mut bytes_read,
                NULL as *mut OVERLAPPED,
            )
        };

        match read_result {
            0 => Err(io::Error::last_os_error()),
            _ => Ok(bytes_read as usize),
        }
    }
}

fn get_std_handle(device: DWORD) -> io::Result<HANDLE> {
    match unsafe { winapi::um::processenv::GetStdHandle(device) } {
        INVALID_HANDLE_VALUE => Err(io::Error::last_os_error()),
        handle => Ok(handle),
    }
}

fn set_std_handle(device: DWORD, handle: HANDLE) -> io::Result<()> {
    match unsafe { winapi::um::processenv::SetStdHandle(device, handle) } {
        0 => Err(io::Error::last_os_error()),
        _ => Ok(()),
    }
}

/// Returns (read_handle, write_handle).
fn create_pipe() -> io::Result<(HANDLE, HANDLE)> {
    let mut read_handle: HANDLE = NULL;
    let mut write_handle: HANDLE = NULL;

    let create_pipe_result = unsafe {
        winapi::um::namedpipeapi::CreatePipe(
            &mut read_handle,
            &mut write_handle,
            NULL as *mut SECURITY_ATTRIBUTES,
            0, // default buffer size
        )
    };

    match create_pipe_result {
        0 => Err(io::Error::last_os_error()),
        _ => Ok((read_handle, write_handle)),
    }
}

/// Uses PeekNamedPipe and checks TotalBytesAvail
fn has_bytes(handle: HANDLE) -> io::Result<bool> {
    let mut bytes_avail: DWORD = 0;

    let result = unsafe {
        winapi::um::namedpipeapi::PeekNamedPipe(
            handle,
            NULL as *mut c_void,
            0,
            NULL as LPDWORD,
            &mut bytes_avail,
            NULL as LPDWORD,
        )
    };
	
    if result == 0 {
        return Err(io::Error::last_os_error());
    }

    Ok(bytes_avail > 0)
}
