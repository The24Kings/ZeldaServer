use std::net::TcpStream;
use std::sync::Arc;
use std::sync::mpsc::Sender;

use crate::protocol::Type;
use crate::protocol::client::Client;

pub fn processor(stream: Arc<TcpStream>, sender: Sender<Type>) {
    let client = Client::new(stream.clone(), sender);

    loop {
        match client.read() {
            Ok(data) => {
                // Process the data
                println!("[CLIENT] Received data: {:?}", data);
            }
            Err(e) => {
                eprintln!("[CLIENT] Error reading from stream: {}", e);

                if e.kind() == std::io::ErrorKind::BrokenPipe {
                    eprintln!("[CLIENT] Broken pipe detected. Terminating thread.");
                    break;
                }

                if e.kind() == std::io::ErrorKind::UnexpectedEof {
                    eprintln!("[CLIENT] User closed the connection. Terminating thread.");
                    break;
                }
            }
        }
    }
}
