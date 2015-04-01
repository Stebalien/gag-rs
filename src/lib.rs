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
//! If you don't want to throw away stdout, you can write it to a file:
//!
//! ```
//! #![feature(io)]
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
#![feature(unsafe_destructor, io_ext)]
extern crate libc;
extern crate tempdir;

mod redirect;
mod gag;
mod buffer;
mod tempfile;

pub use gag::Gag;
pub use redirect::Redirect;
pub use buffer::{BufferRedirect, Buffer};
