use std::sync::{Arc, Mutex, mpsc::Receiver};

use crate::protocol::Type;

pub fn server(receiver: Arc<Mutex<Receiver<Type>>>) {
    loop {
        // Wait for a packet from the receiver
        let packet = match receiver.lock().unwrap().recv() {
            Ok(packet) => packet,
            Err(e) => {
                eprintln!("[SERVER] Error receiving packet: {}", e);
                continue; // Skip this iteration and wait for the next packet
            }
        };

        // Match the type of the packet to the enum Type
        match packet {
            Type::Message(content) => {
                println!("[SERVER] \n{:#?}", content);
            }
            Type::ChangeRoom(content) => {
                println!("[SERVER] \n{:#?}", content);
            }
            Type::Fight(content) => {
                println!("[SERVER] \n{:#?}", content);
            }
            Type::PVPFight(content) => {
                println!("[SERVER] \n{:#?}", content);
            }
            Type::Loot(content) => {
                println!("[SERVER] \n{:#?}", content);
            }
            Type::Start(content) => {
                println!("[SERVER] \n{:#?}", content);
            }
            Type::Error(content) => {
                println!("[SERVER] \n{:#?}", content);
            }
            Type::Accept(content) => {
                println!("[SERVER] \n{:#?}", content);
            }
            Type::Room(content) => {
                println!("[SERVER] \n{:#?}", content);
            }
            Type::Character(content) => {
                println!("[SERVER] \n{:#?}", content);
            }
            Type::Game(content) => {
                // Don't do anything, the server only SENDS this, never receive
                println!("[SERVER] \n{:#?}", content);
            }
            Type::Leave(content) => {
                println!("[SERVER] \n{:#?}", content);
            }
            Type::Connection(content) => {
                println!("[SERVER] \n{:#?}", content);
            }
            Type::Version(content) => {
                // Don't do anything, the server only SENDS this, never receive
                println!("[SERVER] \n{:#?}", content);
            }
        }
    }
}
