use serde::Serialize;

#[derive(Default, Serialize)]
#[repr(u8)]
pub enum ErrorCode {
    #[default]
    OTHER,
    BADROOM,
    PLAYEREXISTS,
    BADMONSTER,
    STATERROR,
    NOTREADY,
    NOTARGET,
    NOFIGHT,
    NOPLAYERCOMBAT,
}

impl Into<u8> for ErrorCode {
    fn into(self) -> u8 {
        self as u8
    }
}

impl From<u8> for ErrorCode {
    fn from(value: u8) -> Self {
        match value {
            0 => ErrorCode::OTHER,
            1 => ErrorCode::BADROOM,
            2 => ErrorCode::PLAYEREXISTS,
            3 => ErrorCode::BADMONSTER,
            4 => ErrorCode::STATERROR,
            5 => ErrorCode::NOTREADY,
            6 => ErrorCode::NOTARGET,
            7 => ErrorCode::NOFIGHT,
            8 => ErrorCode::NOPLAYERCOMBAT,
            _ => ErrorCode::OTHER, // Default case
        }
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ErrorCode::OTHER => write!(f, "Other"),
            ErrorCode::BADROOM => write!(f, "BadRoom"),
            ErrorCode::PLAYEREXISTS => write!(f, "PlayerExists"),
            ErrorCode::BADMONSTER => write!(f, "BadMonster"),
            ErrorCode::STATERROR => write!(f, "StatError"),
            ErrorCode::NOTREADY => write!(f, "NotReady"),
            ErrorCode::NOTARGET => write!(f, "NoTarget"),
            ErrorCode::NOFIGHT => write!(f, "NoFight"),
            ErrorCode::NOPLAYERCOMBAT => write!(f, "NoPlayerCombat"),
        }
    }
}
