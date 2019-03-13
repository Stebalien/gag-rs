extern crate gag;

#[cfg(windows)]
fn main() {
    /////////////////////////////////////////////////////////////
    // Stdout gagging
	println!("STDOUT GAGGING", );
    println!("you will see this");
    let gag = gag::windows::stdout().unwrap();
    println!("but not this");
    drop(gag);
    println!("and this");
    /////////////////////////////////////////////////////////////

    /////////////////////////////////////////////////////////////
    // Stderr gagging
	println!("STDERR GAGGING", );
    eprintln!("you will see this");
    let gag = gag::windows::stderr().unwrap();
    eprintln!("but not this");
    drop(gag);
    eprintln!("and this");
    /////////////////////////////////////////////////////////////

    /////////////////////////////////////////////////////////////
    // Redirecting
	println!("REDIRECTING", );
    use std::io::{Read, Write};

    std::thread::spawn(move || {
        let mut gag = gag::windows::stdout().unwrap();
        let mut stderr = std::io::stderr();
        loop {
            let mut buf = Vec::new();
            gag.read_to_end(&mut buf).unwrap();
            stderr.write_all(&buf).unwrap();
        }
    });

    println!("This should be printed on stderr");
    eprintln!("This will be printed on stderr as well");

    // This will exit and close the spawned thread.
    // In most cases you will want to setup a channel and send a break signal to the loop,
    // and then join the thread back into it once you are finished.

    /////////////////////////////////////////////////////////////
}

#[cfg(unix)]
fn main() {}
