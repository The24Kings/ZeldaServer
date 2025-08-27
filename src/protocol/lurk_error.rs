use serde::Serialize;

#[derive(Default, Serialize)]
#[repr(u8)]
pub enum LurkError {
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

impl Into<u8> for LurkError {
    fn into(self) -> u8 {
        self as u8
    }
}

impl From<u8> for LurkError {
    fn from(value: u8) -> Self {
        match value {
            0 => LurkError::OTHER,
            1 => LurkError::BADROOM,
            2 => LurkError::PLAYEREXISTS,
            3 => LurkError::BADMONSTER,
            4 => LurkError::STATERROR,
            5 => LurkError::NOTREADY,
            6 => LurkError::NOTARGET,
            7 => LurkError::NOFIGHT,
            8 => LurkError::NOPLAYERCOMBAT,
            _ => LurkError::OTHER, // Default case
        }
    }
}

impl std::fmt::Display for LurkError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            LurkError::OTHER => write!(f, "Other"),
            LurkError::BADROOM => write!(f, "BadRoom"),
            LurkError::PLAYEREXISTS => write!(f, "PlayerExists"),
            LurkError::BADMONSTER => write!(f, "BadMonster"),
            LurkError::STATERROR => write!(f, "StatError"),
            LurkError::NOTREADY => write!(f, "NotReady"),
            LurkError::NOTARGET => write!(f, "NoTarget"),
            LurkError::NOFIGHT => write!(f, "NoFight"),
            LurkError::NOPLAYERCOMBAT => write!(f, "NoPlayerCombat"),
        }
    }
}
