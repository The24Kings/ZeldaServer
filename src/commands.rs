use std::sync::mpsc::Sender;

use crate::protocol::Protocol;

pub fn input(_sender: Sender<Protocol>) -> ! {
    loop {} // Take input from the console.
}
