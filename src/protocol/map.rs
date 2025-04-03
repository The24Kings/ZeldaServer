use serde_json::Value;
use std::fs::File;

use super::packet::character::Character;

/*
   Build the game map from a JSON file
   the map should compile into a struct:

   Map {
       tiles: Vec<Tile>,
       players: Vec<Player>,
       monsters: Vec<Monster>,
       desc: String
   }
   Tile {
       id: u8,                         // Unique ID
       title: String,                  // Title of the tile (also needs to be unique)
       exits: Vec<Exit>,               // Possible exits (Points to other tiles)
       players: Vec<Player_index>,     // All players currently on the tile (index to the player list in the map)
       monsters: Vec<Monster_index>,   // All monsters currently on the tile (index to the monster list in the map)
       desc: String
   }
   Exit {
       id: u8,         // Same as the Tile id we are going to
       title: String,  // Title of the tile we are going to
       desc: String    // May be an abbreviated description of the Tile we are going to
   }

   This struct is mutable and should be passed to the server thread
   The server thread should be able to modify the map according to packets sent by the client
*/
#[derive(Default, Debug, Clone)]
pub struct Map {
    pub tiles: Vec<Tile>,
    pub players: Vec<Character>,
    pub monsters: Vec<Character>,
    pub desc_len: u16,
    pub desc: String,
}

#[derive(Default, Debug, Clone)]
pub struct Tile {
    pub id: u8,               // Unique ID (Room Number)
    pub title: String,        // Title of the tile (Room Name)
    pub exits: Vec<usize>,    // Possible exits (Points to other tiles)
    pub players: Vec<usize>, // All players currently on the tile (index to the player list in the map)
    pub monsters: Vec<usize>, // All monsters currently on the tile (index to the monster list in the map)
    pub description_len: u16,
    pub desc: String,
}

impl Map {
    pub fn new() -> Self {
        Map {
            tiles: Vec::new(),
            players: Vec::new(),
            monsters: Vec::new(),
            desc_len: 0,
            desc: String::new(),
        }
    }

    pub fn find_tile(&self, id: u8) -> Option<&Tile> {
        self.tiles.iter().find(|tile| tile.id == id)
    }

    pub fn find_player(&self, name: String) -> Option<&Character> {
        self.players.iter().find(|player| player.name == name)
    }

    pub fn find_monster(&self, name: String) -> Option<&Character> {
        self.monsters.iter().find(|monster| monster.name == name)
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

    pub fn build(data: File) -> Result<Self, serde_json::Error> {
        println!("[MAP] Building game map...");

        match serde_json::from_reader::<File, Value>(data) {
            Ok(json) => {
                let mut map = Map::new();

                // Parse the JSON data into the Map struct
                if let Some(tiles) = json["tiles"].as_array() {
                    // Add all existing tiles to the map
                    for tile in tiles {
                        let id = tile["id"].as_u64().unwrap_or(99) as u8;
                        let title = tile["title"].as_str().unwrap_or("ERROR").to_string();
                        let desc = tile["desc"].as_str().unwrap_or("ERROR").to_string();
                        let exits = tile["connections"]
                            .as_array()
                            .unwrap_or(&vec![])
                            .iter()
                            .filter_map(|v| v.as_u64())
                            .map(|v| v as usize)
                            .collect::<Vec<_>>();

                        let monsters = tile["monsters"]
                            .as_array()
                            .unwrap_or(&vec![])
                            .iter()
                            .filter_map(|v| v.as_u64())
                            .map(|v| v as usize)
                            .collect::<Vec<_>>();

                        let room = Tile::new(id, title.clone(), desc.clone(), monsters.clone(), exits.clone());

                        map.tiles.push(room.clone());

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

impl Tile {
    pub fn new(id: u8, title: String, desc: String, monsters: Vec<usize>, exits: Vec<usize>) -> Self {
        Tile {
            id,
            title,
            exits,
            players: Vec::new(),
            monsters,
            description_len: desc.len() as u16,
            desc,
        }
    }

    pub fn add_exit(&mut self, exit: usize) {
        self.exits.push(exit);
    }
}
