#![feature(std_misc)]
extern crate gag;

use std::io::{Read, Write};
use gag::{BufferRedirect, Hold};
use std::sync::{StaticMutex, MUTEX_INIT};

static STDERR_MUTEX: StaticMutex = MUTEX_INIT;


// Catch the cases not covered by the doc tests.

#[test]
fn test_buffer_stderr() {
    let _l = STDERR_MUTEX.lock().unwrap();

    let mut buf = BufferRedirect::stderr().unwrap();
    println!("Don't capture");
    ::std::io::stderr().write_all(b"Hello world!\n").unwrap();

    let mut output = String::new();
    buf.read_to_string(&mut output).unwrap();

    assert_eq!(&output[..], "Hello world!\n");
}

#[test]
fn test_gag_stderr_twice() {
    let _l = STDERR_MUTEX.lock().unwrap();

    let hold = Hold::stderr();
    let hold2 = Hold::stderr();
    assert!(hold.is_ok());
    assert!(hold2.is_err());
}
