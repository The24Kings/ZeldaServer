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
                println!("[PROCESS] Received data: {:?}", data);
            }
            Err(e) => {
                eprintln!("[PROCESS] Error reading from stream: {}", e);

                match e.kind() {
                    std::io::ErrorKind::ConnectionReset => {
                        eprintln!("[PROCESS] Connection reset by peer. Terminating thread.");
                        break;
                    }
                    std::io::ErrorKind::ConnectionAborted => {
                        eprintln!("[PROCESS] Connection aborted. Terminating thread.");
                        break;
                    }
                    std::io::ErrorKind::NotConnected => {
                        eprintln!("[PROCESS] Not connected. Terminating thread.");
                        break;
                    }
                    std::io::ErrorKind::BrokenPipe => {
                        eprintln!("[PROCESS] Broken pipe. Terminating thread.");
                        break;
                    }
                    std::io::ErrorKind::UnexpectedEof => {
                        eprintln!("[PROCESS] Unexpected EOF. Terminating thread.");
                        break;
                    }
                    _ => {
                        eprintln!("[PROCESS] Non-terminal error: {}. Continuing.", e);
                        // Continue processing other packets
                        continue;
                    }
                }
            }
        }
    }
}
