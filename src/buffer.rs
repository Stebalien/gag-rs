use std::io::{self, Read};
use std::fs::File;

use tempfile::{tempfile_pair, TempFile};
use redirect::Redirect;

/// Buffer output in an in-memory buffer.
pub struct BufferRedirect {
    #[allow(dead_code)]
    redir: Redirect<TempFile>,
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

