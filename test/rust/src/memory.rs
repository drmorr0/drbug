use std::io::{
    Write,
    stdout,
};
use std::str;

use nix::sys::signal::{
    Signal,
    raise,
};

fn main() {
    let a: u64 = 0xcafecafe;
    print!("{:x}", (&a as *const u64) as u64); // no leading 0x for ease of parsing
    stdout().flush().unwrap();
    raise(Signal::SIGTRAP).unwrap();

    let b = [0u8; 11];
    print!("{:x}", (&b as *const [u8; 11]) as u64); // no leading 0x for ease of parsing
    stdout().flush().unwrap();
    raise(Signal::SIGTRAP).unwrap();

    print!("{}", str::from_utf8(&b).unwrap());
    stdout().flush().unwrap();
}
