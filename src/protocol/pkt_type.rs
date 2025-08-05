use serde::Serialize;

#[derive(Default, Serialize, Debug, Clone, Copy)]
#[repr(u8)]
pub enum PktType {
    #[default]
    Default,
    Message,
    ChangeRoom,
    Fight,
    PVPFight,
    Loot,
    Start,
    Error,
    Accept,
    Room,
    Character,
    Game,
    Leave,
    Connection,
    Version,
}

impl Into<u8> for PktType {
    fn into(self) -> u8 {
        self as u8
    }
}

impl From<u8> for PktType {
    fn from(value: u8) -> Self {
        match value {
            0 => PktType::Default,
            1 => PktType::Message,
            2 => PktType::ChangeRoom,
            3 => PktType::Fight,
            4 => PktType::PVPFight,
            5 => PktType::Loot,
            6 => PktType::Start,
            7 => PktType::Error,
            8 => PktType::Accept,
            9 => PktType::Room,
            10 => PktType::Character,
            11 => PktType::Game,
            12 => PktType::Leave,
            13 => PktType::Connection,
            14 => PktType::Version,
            _ => PktType::Default,
        }
    }
}

impl std::fmt::Display for PktType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            PktType::Default => "Default",
            PktType::Message => "Message",
            PktType::ChangeRoom => "ChangeRoom",
            PktType::Fight => "Fight",
            PktType::PVPFight => "PVPFight",
            PktType::Loot => "Loot",
            PktType::Start => "Start",
            PktType::Error => "Error",
            PktType::Accept => "Accept",
            PktType::Room => "Room",
            PktType::Character => "Character",
            PktType::Game => "Game",
            PktType::Leave => "Leave",
            PktType::Connection => "Connection",
            PktType::Version => "Version",
        };
        write!(f, "{name}")
    }
}
