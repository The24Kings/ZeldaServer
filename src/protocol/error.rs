#[derive(Default, Debug, Clone)]
pub enum ErrorCode {
    #[default]
    Other,          // 0
    BadRoom,        // 1
    PlayerExists,   // 2
    BadMonster,     // 3
    StatError,      // 4
    NotReady,       // 5
    NoTarget,       // 6
    NoFight,        // 7
    NoPlayerCombat, // 8
}

impl Into<u8> for ErrorCode {
    fn into(self) -> u8 {
        self as u8
    }
}

impl From<u8> for ErrorCode {
    fn from(value: u8) -> Self {
        match value {
            0 => ErrorCode::Other,
            1 => ErrorCode::BadRoom,
            2 => ErrorCode::PlayerExists,
            3 => ErrorCode::BadMonster,
            4 => ErrorCode::StatError,
            5 => ErrorCode::NotReady,
            6 => ErrorCode::NoTarget,
            7 => ErrorCode::NoFight,
            8 => ErrorCode::NoPlayerCombat,
            _ => ErrorCode::Other, // Default case
        }
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ErrorCode::Other => write!(f, "Other"),
            ErrorCode::BadRoom => write!(f, "BadRoom"),
            ErrorCode::PlayerExists => write!(f, "PlayerExists"),
            ErrorCode::BadMonster => write!(f, "BadMonster"),
            ErrorCode::StatError => write!(f, "StatError"),
            ErrorCode::NotReady => write!(f, "NotReady"),
            ErrorCode::NoTarget => write!(f, "NoTarget"),
            ErrorCode::NoFight => write!(f, "NoFight"),
            ErrorCode::NoPlayerCombat => write!(f, "NoPlayerCombat"),
        }
    }
}
