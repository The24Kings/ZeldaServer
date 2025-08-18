use std::io;
use std::sync::mpsc::Sender;

use serde::Serialize;

use crate::protocol::Protocol;

#[derive(Serialize)]
pub struct Action;

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self)
                .unwrap_or_else(|_| "Failed to serialize Action".to_string())
        )
    }
}

pub fn input(_sender: Sender<Protocol>) -> ! {
    loop {
        // Take input from the console.
        let mut input = String::new();

        match io::stdin().read_line(&mut input) {
            Ok(n) => {
                println!("{n} bytes read");
                println!("{input}");
            }
            Err(error) => println!("error: {error}"),
        }
    }
}
