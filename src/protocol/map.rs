use serde_json::Value;
use std::{fs::File, net::TcpStream, sync::Arc};

use crate::protocol::packet::message::Message;

use super::{
    Type,
    packet::{character::Character, room::Room},
    send,
};

#[derive(Default, Debug, Clone)]
pub struct Map {
    pub init_points: u16,
    pub stat_limit: u16,
    pub rooms: Vec<Room>,
    pub players: Vec<Character>,
    pub monsters: Vec<Character>,
    pub desc_len: u16,
    pub desc: String,
}

impl Map {
    pub fn new() -> Self {
        Map {
            init_points: 100,
            stat_limit: 65525,
            rooms: Vec::new(),
            players: Vec::new(),
            monsters: Vec::new(),
            desc_len: 0,
            desc: String::new(),
        }
    }

    pub fn find_room(&self, id: u16) -> Option<&Room> {
        self.rooms.iter().find(|room| room.room_number == id)
    }

    pub fn find_player(&mut self, name: String) -> Option<&mut Character> {
        self.players.iter_mut().find(|player| player.name == name)
    }

    pub fn find_player_conn(&mut self, _conn: Arc<TcpStream>) -> Option<&mut Character> {
        // self.players.iter_mut().find(|player| {
        //     if let Some(author) = &player.author {
        //         conn.as_ref().map_or(false, |plyr_con| Arc::ptr_eq(author, plyr_con))
        //     } else {
        //         false
        //     }
        // })
        None
    }

    pub fn find_monster(&mut self, name: String) -> Option<&mut Character> {
        self.monsters.iter_mut().find(|monster| monster.name == name)
    }

    pub fn add_player(&mut self, player: Character) {
        self.players.push(player);
    }

    pub fn add_monster(&mut self, monster: Character) {
        self.monsters.push(monster);
    }

    pub fn remove_player(&mut self, name: String) {
        if let Some(pos) = self.players.iter().position(|x| x.name == name) {
            self.players.remove(pos);
        }
    }

    pub fn remove_monster(&mut self, name: String) {
        if let Some(pos) = self.monsters.iter().position(|x| x.name == name) {
            self.monsters.remove(pos);
        }
    }

    /// Broadcast a message to all players in the game
    pub fn broadcast(&self, message: String) -> Result<(), std::io::Error> {
        println!("[BROADCAST] Sending message: {}", message);

        // Send the packet to the server
        for player in &self.players {
            send(Type::Message(
                player.author.clone(), //FIXME: This should be the player's connection, maybe change to use Client rather than Character for the player list
                Message {
                    message_type: 1,
                    message_len: message.len() as u16,
                    recipient: player.name.clone(),
                    sender: "Server".to_string(),
                    narration: false,
                    message: message.clone(),
                }
            ))
            .unwrap_or_else(|e| {
                eprintln!(
                    "[BROADCAST] Failed to send message to {}: {}",
                    player.name, e
                );
            });
        }

        Ok(())
    }

    //TODO: Test this
    /// Alert all players in the current room of a character change
    pub fn alert(&self, id: u16, plyr: &Character) -> Result<(), std::io::Error> {
        println!("[ALERT] Alerting players about: {}", plyr.name);

        if let Some(room) = self.find_room(id) {
            room.players.iter().flatten().for_each(|&player_index| {
                match self.players.get(player_index) {
                    Some(to_alert) => {
                        if let Err(e) = send(Type::Character(plyr.author.clone(), plyr.clone())) { //FIXME: This should be the player's connection, maybe change to use Client rather than Character for the player list
                            eprintln!("[ALERT] Failed to alert {}: {}", to_alert.name, e);
                        }
                    }
                    None => {
                        eprintln!("[ALERT] Invalid player index: {}", player_index);
                    }
                }
            });
        }

        Ok(())
    }

    pub fn get_exits(&self, id: u16) -> Option<Vec<&Room>> {
        if let Some(room) = self.find_room(id) {
            let mut exits = Vec::new();

            if let Some(connections) = &room.connections {
                for exit in connections {
                    if let Some(exit_room) = self.find_room(*exit) {
                        exits.push(exit_room);
                    }
                }
            }

            return Some(exits);
        }

        None
    }

    pub fn build(data: File) -> Result<Self, serde_json::Error> {
        println!("[MAP] Building game map...");

        match serde_json::from_reader::<File, Value>(data) {
            Ok(json) => {
                let mut map = Map::new();

                // Parse the JSON data into the Map struct
                if let Some(tiles) = json["tiles"].as_array() {
                    // Add all existing room to the map
                    for tile in tiles {
                        let id = tile["id"].as_u64().unwrap_or(99) as u16;
                        let title = tile["title"].as_str().unwrap_or("ERROR").to_string();
                        let desc = tile["desc"].as_str().unwrap_or("ERROR").to_string();
                        let exits = tile["connections"]
                            .as_array()
                            .unwrap_or(&vec![])
                            .iter()
                            .filter_map(|v| v.as_u64())
                            .map(|v| v as u16)
                            .collect::<Vec<_>>();

                        let monsters = tile["monsters"]
                            .as_array()
                            .unwrap_or(&vec![])
                            .iter()
                            .filter_map(|v| v.as_u64())
                            .map(|v| v as usize)
                            .collect::<Vec<_>>();

                        // Create a new room and add it to the map
                        let room = Room::new(
                            id,
                            title.clone(),
                            exits.clone(),
                            monsters.clone(),
                            desc.clone(),
                        );

                        map.rooms.push(room.clone());

                        println!("[MAP] {:#?}", room);
                    }
                }

                //TODO: Compile all the monsters into the map's monster vector

                return Ok(map);
            }
            Err(e) => return Err(e),
        }
    }
}
