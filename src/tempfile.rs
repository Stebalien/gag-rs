use ::libc::{self, c_int, O_EXCL, O_RDWR};
use ::tempdir::TempDir;
use std::io;
use std::os::unix::io::FromRawFd;
use std::fs::File;

fn generic_tempfile_pair() -> io::Result<(File, File)> {
    let dir = try!(TempDir::new("rust-gag"));
    let tmp_path = dir.path().join("gag");
    // (write, read)
    Ok((try!(File::create(&tmp_path)), try!(File::open(&tmp_path))))
    // Intentionally drop tmp_path to delete the directory.
}

#[cfg(target_os = "linux")]
pub fn tempfile_pair() -> io::Result<(File, File)> {
    const O_TMPFILE: libc::c_int = 4259840;
    // There has to be a better way to do this...
    static DEV_SHM: [i8; 9] = [
        b'/' as i8,
        b'd' as i8,
        b'e' as i8,
        b'v' as i8,
        b'/' as i8,
        b's' as i8,
        b'h' as i8,
        b'm' as i8,
        0i8];

    match unsafe {
        libc::open(&DEV_SHM as *const i8, O_EXCL | O_TMPFILE | O_RDWR, 0o600)
    } {
        // Always fall back to the generic version.
        -1 => generic_tempfile_pair(),
        fd => Ok((FromRawFd::from_raw_fd(fd), try!(File::open(format!("/proc/self/fd/{}", fd)))))
    }
}

#[cfg(not(target_os = "linux"))]
#[inline(always)]
pub fn tempfile_pair() -> io::Result<(File, File)> {
    generic_tempfile_pair()
}
