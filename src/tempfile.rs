use ::libc::{self, c_int, O_EXCL, O_RDWR};
use ::tempdir::TempDir;
use std::io;
use std::os::unix::io::{Fd, AsRawFd};
use std::fs::File;


pub enum TempFile {
    Linux(Fd),
    Generic(File),
}

impl Drop for TempFile {
    fn drop(&mut self) {
        use self::TempFile::*;
        match self {
            &mut Linux(fd) => unsafe { libc::close(fd); },
            _ => ()
        };
    }
}
impl AsRawFd for TempFile {
    fn as_raw_fd(&self) -> Fd {
        use self::TempFile::*;
        match self {
            &Linux(fd) => fd,
            &Generic(ref file) => file.as_raw_fd(),
        }
    }
}

fn generic_tempfile_pair() -> io::Result<(TempFile, File)> {
    let dir = try!(TempDir::new("rust-gag"));
    let tmp_path = dir.path().join("gag");
    let inner = try!(File::create(&tmp_path));
    let outer = try!(File::open(&tmp_path));
    // Intentionally drop tmp_path to delete the directory.
    Ok((TempFile::Generic(inner), outer))
}

#[cfg(target_os = "linux")]
pub fn tempfile_pair() -> io::Result<(TempFile, File)> {
    const O_TMPFILE: libc::c_int = 4259840;
    // There has to be a better way to do this...
    static DEV_SHM: [i8; 9] = [b'/' as i8, b'd' as i8, b'e' as i8, b'v' as i8, b'/' as i8, b's' as i8, b'h' as i8, b'm' as i8, 0i8];

    let fd = match unsafe { libc::open(&DEV_SHM as *const i8, O_EXCL | O_TMPFILE | O_RDWR, 0o600) } {
        -1 => return Err(io::Error::last_os_error()),
        // Always fall back to the generic version.
        22 => return generic_tempfile_pair(),
        n => n as Fd,
    };
    // Ugly dirty hack. I need an independent file descriptor so I can read/write from two
    // different points in the file.
    let outer = try!(File::open(format!("/proc/self/fd/{}", fd)));
    Ok((TempFile::Linux(fd), outer))
}

#[cfg(not(target_os = "linux"))]
#[inline(always)]
pub fn tempfile_pair() -> io::Result<(TempFile, File)> {
    generic_tempfile_pair()
}
