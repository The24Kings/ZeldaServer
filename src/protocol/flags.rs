use bitflags::bitflags;

bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct CharacterFlags: u8 {
        const ALIVE = 0b10000000;
        const BATTLE = 0b01000000; // A.K.A. Join-Battle
        const MONSTER = 0b00100000;
        const STARTED = 0b00010000;
        const READY = 0b00001000;
    }
}

impl CharacterFlags {
    pub fn is_alive(&self) -> bool {
        self.contains(CharacterFlags::ALIVE)
    }

    pub fn is_battle(&self) -> bool {
        self.contains(CharacterFlags::BATTLE)
    }

    pub fn is_started(&self) -> bool {
        self.contains(CharacterFlags::STARTED)
    }

    pub fn is_ready(&self) -> bool {
        self.contains(CharacterFlags::READY)
    }

    pub fn dead() -> Self {
        CharacterFlags::BATTLE | CharacterFlags::READY
    }

    pub fn alive() -> Self {
        CharacterFlags::ALIVE | CharacterFlags::BATTLE | CharacterFlags::READY
    }

    pub fn reset() -> Self {
        CharacterFlags::ALIVE | CharacterFlags::BATTLE
    }
}
