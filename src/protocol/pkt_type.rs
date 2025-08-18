use serde::Serialize;

#[derive(Default, Serialize, Debug, Clone, Copy)]
#[repr(u8)]
pub enum PktType {
    #[default]
    DEFAULT,
    MESSAGE,
    CHANGEROOM,
    FIGHT,
    PVPFIGHT,
    LOOT,
    START,
    ERROR,
    ACCEPT,
    ROOM,
    CHARACTER,
    GAME,
    LEAVE,
    CONNECTION,
    VERSION,
}

impl Into<u8> for PktType {
    fn into(self) -> u8 {
        self as u8
    }
}

impl From<u8> for PktType {
    fn from(value: u8) -> Self {
        match value {
            0 => PktType::DEFAULT,
            1 => PktType::MESSAGE,
            2 => PktType::CHANGEROOM,
            3 => PktType::FIGHT,
            4 => PktType::PVPFIGHT,
            5 => PktType::LOOT,
            6 => PktType::START,
            7 => PktType::ERROR,
            8 => PktType::ACCEPT,
            9 => PktType::ROOM,
            10 => PktType::CHARACTER,
            11 => PktType::GAME,
            12 => PktType::LEAVE,
            13 => PktType::CONNECTION,
            14 => PktType::VERSION,
            _ => PktType::DEFAULT,
        }
    }
}

impl std::fmt::Display for PktType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PktType::DEFAULT => write!(f, "Default"),
            PktType::MESSAGE => write!(f, "Message"),
            PktType::CHANGEROOM => write!(f, "ChangeRoom"),
            PktType::FIGHT => write!(f, "Fight"),
            PktType::PVPFIGHT => write!(f, "PVPFight"),
            PktType::LOOT => write!(f, "Loot"),
            PktType::START => write!(f, "Start"),
            PktType::ERROR => write!(f, "Error"),
            PktType::ACCEPT => write!(f, "Accept"),
            PktType::ROOM => write!(f, "Room"),
            PktType::CHARACTER => write!(f, "Character"),
            PktType::GAME => write!(f, "Game"),
            PktType::LEAVE => write!(f, "Leave"),
            PktType::CONNECTION => write!(f, "Connection"),
            PktType::VERSION => write!(f, "Version"),
        }
    }
}
