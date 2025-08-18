use std::io;
use std::sync::mpsc::Sender;

use crate::protocol::Protocol;

pub fn input(_sender: Sender<Protocol>) -> ! {
    loop {
        let mut input = String::new();

        match io::stdin().read_line(&mut input) {
            Ok(n) => {
                println!("{n} bytes read");
                println!("{input}");
            }
            Err(error) => println!("error: {error}"),
        }
    } // Take input from the console.
}
