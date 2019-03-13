//! Windows gagging.

#![warn(missing_docs)]

use std::io;
use winapi;
use winapi::ctypes::c_void;
use winapi::shared::{minwindef::DWORD, ntdef::NULL};
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
	eof: bool,
	mrker: std::marker::PhantomData<Io>,
}

impl Gag<Stdout> {
	/// Gags the stderr stream.
	///
	/// # Example
	/// ```rust
	/// println!("you will see this");
	/// let gag = Gag::stdout().unwrap();
	/// println!("but not this");
	/// drop(gag);
	/// println!("and this");
	/// ```
	pub fn stdout() -> io::Result<Self> {
		Gag::redirect(STD_OUTPUT_HANDLE)
	}
}

impl Gag<Stdout> {
	/// Gags the stderr stream.
	///
	/// # Example
	/// ```rust
	/// eprintln!("you will see this");
	/// let gag = Gag::stderr().unwrap();
	/// eprintln!("but not this");
	/// drop(gag);
	/// eprintln!("and this");
	/// ```
	pub fn stderr() -> io::Result<Self> {
		Gag::redirect(STD_ERROR_HANDLE)
	}
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
			eof: false,
			mrker: std::marker::PhantomData,
		})
	}
}

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
		if self.eof {
			self.eof = false;
			return Ok(0); // eof
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
			_ => {
				if bytes_read < buf_len {
					// read less than the buffer, for pipes this means EOF reached
					self.eof = true;
				}
				Ok(bytes_read as usize)
			}
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
