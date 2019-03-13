//! Temporarily redirect stdout/stderr.
//!
//! For example, one can temporarily throw away stdout output:
//!
//! ```
//! use gag::Gag;
//! println!("Hello world!");
//! {
//!     let print_gag = Gag::stdout().unwrap();
//!     println!("No one will see this!");
//!     println!("Or this!");
//! }
//! println!("But they will see this!");
//! ```
//!
//! You can also temporarily un-gag by dropping the gag:
//!
//! ```
//! use gag::Gag;
//! let mut print_gag = Gag::stdout().unwrap();
//! println!("No one will see this!");
//! if true {
//!     drop(print_gag);
//!     println!("They will see this...");
//!     print_gag = Gag::stdout().unwrap();
//!     println!("Not this...");
//! }
//! println!("Nor this.");
//! ```
//!
//! However, you can't open multiple Gags/Redirects/Holds for the same output at once:
//!
//! ```
//! use gag::Gag;
//! // Multiple stdout gags
//! let gag_a = Gag::stdout().unwrap();
//! let gag_b_result = Gag::stdout();
//! assert!(gag_b_result.is_err());
//! assert_eq!(gag_b_result.err().expect("Expected an error").kind(),
//!            std::io::ErrorKind::AlreadyExists);
//!
//! // However, you can still gag stderr:
//! let gag_c = Gag::stderr().unwrap();
//! ```
//!
//! If you don't want to throw away stdout, you can write it to a file:
//!
//! ```
//! use std::fs::OpenOptions;
//! use std::io::{Read, Write, Seek, SeekFrom};
//! use gag::Redirect;
//!
//! println!("Displayed");
//!
//! // Open a log
//! let log = OpenOptions::new()
//!     .truncate(true)
//!     .read(true)
//!     .create(true)
//!     .write(true)
//!     .open("/tmp/my_log.log")
//!     .unwrap();
//!
//! let print_redirect = Redirect::stdout(log).unwrap();
//! println!("Hidden");
//!
//! // Extract redirect
//! let mut log = print_redirect.into_inner();
//! println!("Displayed");
//!
//! let mut buf = String::new();
//! log.seek(SeekFrom::Start(0)).unwrap();
//! log.read_to_string(&mut buf).unwrap();
//! assert_eq!(&buf[..], "Hidden\n");
//!
//! ```
//!
//! Alternatively, you can buffer stdout to a temporary file. On linux 3.11+, this file is
//! guarenteed to be stored in-memory.
//!
//! ```
//! use std::io::Read;
//! use gag::BufferRedirect;
//!
//! let mut buf = BufferRedirect::stdout().unwrap();
//! println!("Hello world!");
//!
//! let mut output = String::new();
//! buf.read_to_string(&mut output).unwrap();
//!
//! assert_eq!(&output[..], "Hello world!\n");
//! ```
//!
//! Finally, if you just want to temporarily hold std output, you can use `Hold` to hold the output
//! until dropped:
//!
//! ```
//! use gag::Hold;
//!
//! let hold = Hold::stdout().unwrap();
//! println!("first");
//! println!("second");
//! drop(hold); // printing happens here!
//! ```
#[cfg(unix)]
extern crate libc;
#[cfg(unix)]
extern crate tempfile;

#[cfg(unix)]
mod buffer;
#[cfg(unix)]
mod gag;
#[cfg(unix)]
mod hold;
#[cfg(unix)]
mod redirect;

#[cfg(unix)]
pub use buffer::{Buffer, BufferRedirect};
#[cfg(unix)]
pub use gag::Gag;
#[cfg(unix)]
pub use hold::Hold;
#[cfg(unix)]
pub use redirect::{Redirect, RedirectError};

#[cfg(windows)]
extern crate winapi;
#[cfg(windows)]
pub mod windows;
