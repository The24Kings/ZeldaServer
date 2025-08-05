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
        match self {
            PktType::Default => write!(f, "Default"),
            PktType::Message => write!(f, "Message"),
            PktType::ChangeRoom => write!(f, "ChangeRoom"),
            PktType::Fight => write!(f, "Fight"),
            PktType::PVPFight => write!(f, "PVPFight"),
            PktType::Loot => write!(f, "Loot"),
            PktType::Start => write!(f, "Start"),
            PktType::Error => write!(f, "Error"),
            PktType::Accept => write!(f, "Accept"),
            PktType::Room => write!(f, "Room"),
            PktType::Character => write!(f, "Character"),
            PktType::Game => write!(f, "Game"),
            PktType::Leave => write!(f, "Leave"),
            PktType::Connection => write!(f, "Connection"),
            PktType::Version => write!(f, "Version"),
        }
    }
}
