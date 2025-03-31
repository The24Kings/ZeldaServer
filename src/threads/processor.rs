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

                match e.kind() {
                    std::io::ErrorKind::ConnectionReset => {
                        eprintln!("[CLIENT] Connection reset by peer. Terminating thread.");
                        break;
                    }
                    std::io::ErrorKind::ConnectionAborted => {
                        eprintln!("[CLIENT] Connection aborted. Terminating thread.");
                        break;
                    }
                    std::io::ErrorKind::NotConnected => {
                        eprintln!("[CLIENT] Not connected. Terminating thread.");
                        break;
                    }
                    std::io::ErrorKind::BrokenPipe => {
                        eprintln!("[CLIENT] Broken pipe. Terminating thread.");
                        break;
                    }
                    std::io::ErrorKind::UnexpectedEof => {
                        eprintln!("[CLIENT] Unexpected EOF. Terminating thread.");
                        break;
                    }
                    _ => {
                        eprintln!("[CLIENT] Non-terminal error: {}. Continuing.", e);
                        // Continue processing other packets
                        continue;
                    }
                }
            }
        }
    }
}
