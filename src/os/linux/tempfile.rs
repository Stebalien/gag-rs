use ::libc::{self, c_int, O_EXCL, O_RDWR};
use std::io;
use std::os::unix::io::Fd;

const O_TMPFILE: libc::c_int = 4259840;
// There has to be a better way to do this...
static DEV_SHM: [i8; 9] = [b'/' as i8,
                            b'd' as i8,
                            b'e' as i8,
                            b'v' as i8,
                            b'/' as i8,
                            b's' as i8,
                            b'h' as i8,
                            b'm' as i8,
                            0i8];


// Keep this outside impl to prevent users from creating these.
pub fn tempfile() -> io::Result<Fd> {
    let fd = unsafe { libc::open(&DEV_SHM as *const i8, O_EXCL | O_TMPFILE | O_RDWR, 0o600) };
    if fd == -1 {
        Err(io::Error::last_os_error())
    } else {
        Ok(fd as Fd)
    }
}
