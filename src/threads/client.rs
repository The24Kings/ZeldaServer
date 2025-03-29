use std::io::Read;
use std::sync::Arc;
use std::net::TcpStream;

pub fn client(stream: Arc<TcpStream>) {
    println!("Client connected: {}", stream.peer_addr().unwrap());
    let mut buffer = [0; 1024];

    loop {
        match stream.as_ref().read(&mut buffer) {
            Ok(0) => {
                println!("Client disconnected");
                break;
            }
            Ok(n) => {
                println!("Received {} bytes", n);
                // Process the data received from the client
                // For example, you can deserialize the packet and handle it accordingly
            }
            Err(e) => {
                eprintln!("Error reading from stream: {}", e);
                break;
            }
        }
    }
}