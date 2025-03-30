use std::sync::{Arc, Mutex, mpsc::Receiver};

use crate::protocol::Type;

pub fn server(receiver: Arc<Mutex<Receiver<Type>>>) {
    loop {
        let packet = receiver.lock().unwrap().recv();

        match packet {
            Ok(packet) => {
                // Process the packet
                match packet {
                    Type::Message(content) => {
                        println!("[SERVER] {:?}", content);
                    }
                    Type::ChangeRoom(content) => {
                        println!("[SERVER] {:?}", content);
                    }
                    Type::Fight(content) => {
                        println!("[SERVER] {:?}", content);
                    }
                    Type::PVPFight(content) => {
                        println!("[SERVER] {:?}", content);
                    }
                    Type::Loot(content) => {
                        println!("[SERVER] {:?}", content);
                    }
                    Type::Start(content) => {
                        println!("[SERVER] {:?}", content);
                    }
                    Type::Error(content) => {
                        eprintln!("[SERVER] {:?}", content);
                    }
                    Type::Accept(content) => {
                        println!("[SERVER] {:?}", content);
                    }
                    Type::Room(content) => {
                        println!("[SERVER] {:?}", content);
                    }
                    Type::Character(content) => {
                        println!("[SERVER] {:?}", content);
                    }
                    Type::Game(content) => {
                        println!("[SERVER] {:?}", content);
                    }
                    Type::Leave(content) => {
                        println!("[SERVER] {:?}", content);
                    }
                    Type::Connection(content) => {
                        println!("[SERVER] {:?}", content);
                    }
                    Type::Version(content) => {
                        println!("[SERVER] {:?}", content);
                    }
                }
            }
            Err(e) => {
                eprintln!("[SERVER] Error receiving packet: {}", e);
            }
        }
    }
}
