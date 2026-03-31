use lurk_lcsc::CharacterFlags;
use lurk_lcsc::PktLeave;
use std::net::TcpStream;
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::logic::map;
use crate::logic::state::GameState;

impl GameState {
    pub fn handle_leave(&mut self, author: Arc<TcpStream>, content: PktLeave) {
        info!("Received: {}", content);

        // ================================================================================
        // Grab the player and deactivate them, alert the server and the room that the player
        // has been deactivated, but is technically still there.
        // Shutdown the connection.
        // ================================================================================
        let player = match map::player_from_stream(&mut self.players, author.clone()) {
            Some((_, player)) => player,
            None => return,
        };

        player.flags = CharacterFlags::empty();
        player.author = None;

        let player = player.clone(); // End mutable borrow of player

        let room = match self.rooms.get(&player.current_room) {
            Some(room) => room,
            None => {
                warn!("Unable to find where the player left off in the map");
                return;
            }
        };

        self.broadcast(format!("{} has left the game.", player.name));
        self.alert_room(room, &player);

        match author.shutdown(std::net::Shutdown::Both) {
            Ok(_) => info!("Connection shutdown successfully"),
            Err(e) => error!("Failed to shutdown connection: {}", e),
        }
    }
}
