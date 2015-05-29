use num::FromPrimitive;

pub const FALSE: u8 = 0;
pub const TRUE: u8 = 1;

/// This gets added to the trainer class when setting CURRENT_OPPONENT
pub const TRAINER_TAG: u8 = 0xC8;

/// Any tile that has a value above this is a menu tile and therefore should be drawn on top of
/// sprites.
pub const MAX_MAP_TILE: u8 = 0x5F;

#[derive(Copy, Clone, Debug, PartialEq, Eq, RustcEncodable, RustcDecodable)]
pub enum Direction {
    Down = 0x0,
    Up = 0x4,
    Left = 0x8,
    Right = 0xC
}

impl FromPrimitive for Direction {
    fn from_i64(n: i64) -> Option<Direction> {
        match n {
            0x0 => Some(Direction::Down),
            0x4 => Some(Direction::Up),
            0x8 => Some(Direction::Left),
            0xC => Some(Direction::Right),
            _ => None,
        }
    }
    
    fn from_u64(n: u64) -> Option<Direction> {
        FromPrimitive::from_i64(n as i64)
    }
}


pub enum BattleType {
    Normal = 0,
    OldMan = 1,
    Safari = 2,
}

pub enum ActiveBattle {
    None = 0,
    Wild = 1,
    Trainer = 2,
}

pub enum TrainerClass {
    Unknown = 0x00,
    ProfOak = 0x1A,
}

pub mod pokeid {
    // TODO: Add more pokemon
    pub const RYHDON: u8 = 0x01;
    pub const WEEDLE: u8 = 0x70;
}

pub mod status {
    pub const NONE    : u8 = 0;
    pub const POISON  : u8 = 3;
    pub const BURN    : u8 = 4;
    pub const FREEZE  : u8 = 5;
    pub const PARALYZE: u8 = 6;
    pub const SLEEP   : u8 = 7;
}

pub mod types {
    pub const NORMAL  : u8 = 0x00;
    pub const FIGHTING: u8 = 0x01;
    pub const FLYING  : u8 = 0x02;
    pub const POISON  : u8 = 0x03;
    pub const GROUND  : u8 = 0x04;
    pub const ROCK    : u8 = 0x05;
    pub const BUG     : u8 = 0x07;
    pub const GHOST   : u8 = 0x08;
    pub const FIRE    : u8 = 0x14;
    pub const WATER   : u8 = 0x15;
    pub const GRASS   : u8 = 0x16;
    pub const ELECTRIC: u8 = 0x17;
    pub const PSYCHIC : u8 = 0x18;
    pub const ICE     : u8 = 0x19;
    pub const DRAGON  : u8 = 0x1A;
}

pub mod moves {
    // TODO: Add more moves
    pub const NONE        : u8 = 0x00;
    pub const POUND       : u8 = 0x01;
    pub const KARATE_CHOP : u8 = 0x02;
    pub const DOUBLESLAP  : u8 = 0x03;
    pub const COMET_PUNCH : u8 = 0x04;
    pub const MEGA_PUNCH  : u8 = 0x05;
    pub const PAY_DAY     : u8 = 0x06;
    pub const FIRE_PUNCH  : u8 = 0x07;
    pub const ICE_PUNCH   : u8 = 0x08;
    pub const THUNDERPUNCH: u8 = 0x09;
    pub const SCRATCH     : u8 = 0x0a;
    pub const VICEGRIP    : u8 = 0x0b;
    pub const GUILLOTINE  : u8 = 0x0c;
    pub const RAZOR_WIND  : u8 = 0x0d;
    pub const SWORDS_DANCE: u8 = 0x0e;
    pub const CUT         : u8 = 0x0f;
}
