use std::fs::OpenOptions;
use std::fs::File;
use std::io;
use redirect::Redirect;

// Helper function for opening /dev/null
fn null() -> io::Result<File> {
    OpenOptions::new().write(true).open("/dev/null")
}

/// Discard output until dropped.
pub struct Gag(Redirect<File>);

impl Gag {
    /// Discard stdout until dropped.
    pub fn stdout() -> io::Result<Gag> {
        Redirect::stdout(try!(null())).map(|r| Gag(r))
    }
    /// Discard stderr until dropped.
    pub fn stderr() -> io::Result<Gag> {
        Redirect::stderr(try!(null())).map(|r| Gag(r))
    }
}
