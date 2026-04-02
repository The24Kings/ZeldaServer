use lurk_lcsc::send_error;
use lurk_lcsc::{LurkError, PktError, PktPVPFight};
use std::net::TcpStream;
use std::sync::Arc;
use tracing::info;

use crate::logic::GameState;

impl GameState {
    pub fn handle_pvp_fight(&self, author: Arc<TcpStream>, content: PktPVPFight) {
        info!("Received: {}", content);

        send_error!(
            author.clone(),
            PktError::new(LurkError::NOPLAYERCOMBAT, "No player combat allowed")
        );
    }
}
