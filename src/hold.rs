use std::io::{self, Read, Write};
use ::BufferRedirect;

/// Hold output until dropped. On drop, the held output is sent to the stdout/stderr.
///
/// Note: This will ignore IO errors when printing held output.
pub struct Hold {
    buf_redir: Option<BufferRedirect>,
    is_stdout: bool,
}

impl Hold {
    /// Hold stderr output.
    pub fn stderr() -> io::Result<Hold> {
        Ok(Hold {
            buf_redir: Some(try!(BufferRedirect::stderr())),
            is_stdout: false,
        })
    }

    /// Hold stdout output.
    pub fn stdout() -> io::Result<Hold> {
        Ok(Hold {
            buf_redir: Some(try!(BufferRedirect::stdout())),
            is_stdout: true,
        })
    }
}

impl Drop for Hold {
    fn drop(&mut self) {
        fn read_into<R: Read, W: Write>(mut from: R, mut to: W) -> io::Result<()> {
            // TODO: use sendfile?
            let mut buf = [0u8; 4096];
            loop {
                match from.read(&mut buf) {
                    Ok(0) => return Ok(()),
                    Ok(size) => try!(to.write_all(&buf[..size])),
                    Err(e) => return Err(e),
                }
            }
        }

        let from = self.buf_redir.take().unwrap().into_inner();
        // Ignore errors.
        if self.is_stdout {
            let stdout = io::stdout();
            let _ = read_into(from, stdout.lock());
        } else {
            let stderr = io::stderr();
            let _ = read_into(from, stderr.lock());
        }
    }
}
