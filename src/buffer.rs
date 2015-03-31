use std::io::{self, Read};
use std::os::unix::io::Fd;
use std::fs::File;

use super::tempfile::tempfile;
use super::redirect::Redirect;

/// Buffer output in an in-memory buffer.
pub struct BufferRedirect {
    #[allow(dead_code)]
    redir: Redirect<Fd>,
    outer: File,
}

/// An in-memory read-only buffer into which BufferRedirect buffers output.
pub struct Buffer(File);

impl Read for Buffer {
    #[inline(always)]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

fn tempfile_pair() -> io::Result<(Fd, File)> {
    let fd = try!(tempfile());
    // Ugly dirty hack. I need an independent file descriptor so I can read/write from two
    // different points in the file.
    let outer = try!(File::open(format!("/proc/self/fd/{}", fd)));
    Ok((fd, outer))
}

impl BufferRedirect {
    /// Buffer stdout.
    pub fn stdout() -> io::Result<BufferRedirect> {
        let (inner, outer) = try!(tempfile_pair());
        Redirect::stdout(inner).map(|r| BufferRedirect {
            redir: r,
            outer: outer
        })
    }
    /// Buffer stderr.
    pub fn stderr() -> io::Result<BufferRedirect> {
        let (inner, outer) = try!(tempfile_pair());
        Redirect::stderr(inner).map(|r| BufferRedirect {
            redir: r,
            outer: outer
        })
    }

    /// Extract the inner buffer and stop redirecting output.
    pub fn into_inner(self) -> Buffer {
        Buffer(self.outer)
    }
}

impl Read for BufferRedirect {
    #[inline(always)]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.outer.read(buf)
    }
}

