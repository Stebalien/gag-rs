#![feature(unsafe_destructor, libc, io_ext)]
extern crate libc;

mod redirect;
mod gag;

pub use gag::Gag;
pub use redirect::Redirect;

#[test]
fn it_works() {
    println!("first");
    {
        let gag = Gag::stdout().unwrap();
        println!("second");
    }
    println!("third");
}
