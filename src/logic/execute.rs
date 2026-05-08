use lurk_engine::{ClientId, GameEngine, Output};
use lurk_protocol::{
    PktAccept, PktCharacter, PktConnection, PktError, PktMessage, PktRoom, PktType, send_to,
};
use std::collections::HashMap;
use std::net::{Shutdown, TcpStream};
use std::sync::Arc;
use tracing::debug;

/// Shutdown and remove a client from the clients map.
pub fn disconnect_client(client: &ClientId, clients: &mut HashMap<ClientId, Arc<TcpStream>>) {
    if let Some(stream) = clients.remove(client) {
        debug!("Disconnecting client {}", client);
        let _ = stream.shutdown(Shutdown::Both);
    }
}

/// Execute a single Output event by performing the actual IO.
pub fn execute_output(
    output: &Output,
    clients: &mut HashMap<ClientId, Arc<TcpStream>>,
    engine: &GameEngine,
) {
    match output {
        Output::SendError {
            client,
            error_code,
            message,
        } => {
            if let Some(stream) = clients.get(client) {
                let pkt = PktError::new(*error_code, message);
                let _ = send_to(stream.as_ref(), &pkt);
            }
        }
        Output::SendAccept {
            client,
            accepted_type,
        } => {
            if let Some(stream) = clients.get(client) {
                let pkt = PktAccept {
                    packet_type: PktType::ACCEPT,
                    accept_type: (*accepted_type).into(),
                };
                let _ = send_to(stream.as_ref(), &pkt);
            }
        }
        Output::SendCharacter { client, character } => {
            if let Some(stream) = clients.get(client) {
                let pkt = PktCharacter {
                    packet_type: PktType::CHARACTER,
                    name: character.name.clone(),
                    flags: character.flags,
                    attack: character.attack,
                    defense: character.defense,
                    regen: character.regen,
                    health: character.health,
                    gold: character.gold,
                    current_room: character.current_room,
                    description_len: character.description.len() as u16,
                    description: character.description.clone(),
                };
                let _ = send_to(stream.as_ref(), &pkt);
            }
        }
        Output::SendRoom { client, room } => {
            if let Some(stream) = clients.get(client) {
                let pkt = PktRoom {
                    packet_type: PktType::ROOM,
                    room_number: room.room_number,
                    room_name: room.title.clone(),
                    description_len: room.description.len() as u16,
                    description: room.description.clone(),
                };
                let _ = send_to(stream.as_ref(), &pkt);
            }
        }
        Output::SendConnection { client, connection } => {
            if let Some(stream) = clients.get(client) {
                let pkt = PktConnection {
                    packet_type: PktType::CONNECTION,
                    room_number: connection.room_number,
                    room_name: connection.title.clone(),
                    description_len: connection.description.len() as u16,
                    description: connection.description.clone(),
                };
                let _ = send_to(stream.as_ref(), &pkt);
            }
        }
        Output::SendMessage {
            client,
            sender_name,
            recipient_name,
            message,
        } => {
            if let Some(stream) = clients.get(client) {
                let pkt = PktMessage::player(sender_name, recipient_name, message);
                let _ = send_to(stream.as_ref(), &pkt);
            }
        }
        Output::SendNarration {
            client,
            recipient_name,
            message,
            narration,
        } => {
            if let Some(stream) = clients.get(client) {
                let pkt = if *narration {
                    PktMessage::narrator(recipient_name, message)
                } else {
                    PktMessage::server(recipient_name, message)
                };
                let _ = send_to(stream.as_ref(), &pkt);
            }
        }
        Output::Broadcast { message } => {
            for (id, stream) in clients {
                let name = engine
                    .players()
                    .iter()
                    .find(|(_, ps)| ps.client == Some(*id))
                    .map(|(n, _)| n.clone());

                if let Some(name) = name {
                    let pkt = PktMessage::server(&name, message);
                    let _ = send_to(stream.as_ref(), &pkt);
                }
            }
        }
        Output::Narrate {
            room_number,
            message,
            narration,
        } => {
            let Some(room) = engine.rooms().get(room_number) else {
                return;
            };
            for player_name in &room.players {
                let Some(ps) = engine.players().get(player_name) else {
                    continue;
                };
                let Some(client_id) = ps.client else {
                    continue;
                };
                let Some(stream) = clients.get(&client_id) else {
                    continue;
                };
                let pkt = if *narration {
                    PktMessage::narrator(player_name, message)
                } else {
                    PktMessage::server(player_name, message)
                };
                let _ = send_to(stream.as_ref(), &pkt);
            }
        }
        Output::AlertRoom {
            room_number,
            character,
        } => {
            let Some(room) = engine.rooms().get(room_number) else {
                return;
            };
            let pkt = PktCharacter {
                packet_type: PktType::CHARACTER,
                name: character.name.clone(),
                flags: character.flags,
                attack: character.attack,
                defense: character.defense,
                regen: character.regen,
                health: character.health,
                gold: character.gold,
                current_room: character.current_room,
                description_len: character.description.len() as u16,
                description: character.description.clone(),
            };
            for player_name in &room.players {
                let Some(ps) = engine.players().get(player_name) else {
                    continue;
                };
                let Some(client_id) = ps.client else {
                    continue;
                };
                let Some(stream) = clients.get(&client_id) else {
                    continue;
                };
                let _ = send_to(stream.as_ref(), &pkt);
            }
        }
        Output::Disconnect { client } => {
            disconnect_client(client, clients);
        }
    }
}
