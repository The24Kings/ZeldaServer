use lurk_lcsc::Protocol;
use lurk_sansio::{ClientId, Input};

/// Translate a lurk_lcsc Protocol message into a lurk-sansio Input event.
pub fn translate(client: ClientId, protocol: Protocol) -> Option<Input> {
    match protocol {
        Protocol::Character(pkt) => Some(Input::Character {
            client,
            name: pkt.name,
            attack: pkt.attack,
            defense: pkt.defense,
            regen: pkt.regen,
            description: pkt.description,
        }),
        Protocol::Start(_) => Some(Input::Start { client }),
        Protocol::ChangeRoom(pkt) => Some(Input::ChangeRoom {
            client,
            room_number: pkt.room_number,
        }),
        Protocol::Fight(_) => Some(Input::Fight { client }),
        Protocol::PVPFight(pkt) => Some(Input::PvpFight {
            client,
            target_name: pkt.target_name.into(),
        }),
        Protocol::Loot(pkt) => Some(Input::Loot {
            client,
            target_name: pkt.target_name.into(),
        }),
        Protocol::Message(pkt) => Some(Input::Message {
            client,
            sender_name: pkt.sender.into(),
            recipient_name: pkt.recipient.into(),
            message: pkt.message,
        }),
        Protocol::Leave(_) => Some(Input::Leave { client }),
        _ => None,
    }
}
