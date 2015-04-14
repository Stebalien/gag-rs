use std::io::{self, Read};

use redirect::Redirect;
use tempfile::TempFile;

/// Buffer output in an in-memory buffer.
pub struct BufferRedirect {
    #[allow(dead_code)]
    redir: Redirect<TempFile>,
    outer: TempFile,
}

/// An in-memory read-only buffer into which BufferRedirect buffers output.
pub struct Buffer(TempFile);

impl Read for Buffer {
    #[inline(always)]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

impl BufferRedirect {
    /// Buffer stdout.
    pub fn stdout() -> io::Result<BufferRedirect> {
        let mut iter = try!(TempFile::shared(2)).into_iter();
        let inner = iter.next().unwrap();
        let outer = iter.next().unwrap();
        let redir = try!(Redirect::stdout(inner));
        Ok(BufferRedirect {
            redir: redir,
            outer: outer
        })
    }
    /// Buffer stderr.
    pub fn stderr() -> io::Result<BufferRedirect> {
        let mut iter = try!(TempFile::shared(2)).into_iter();
        let inner = iter.next().unwrap();
        let outer = iter.next().unwrap();
        let redir = try!(Redirect::stderr(inner));
        Ok(BufferRedirect {
            redir: redir,
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

