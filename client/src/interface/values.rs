#![allow(dead_code)]

pub const FALSE: u8 = 0;
pub const TRUE: u8 = 1;

pub enum PlayerDir {
    Down = 0,
    Up = 4,
    Left = 8,
    Right = 12,
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

// This gets added to the trainer class when setting CURRENT_OPPONENT
pub const TRAINER_TAG: u8 = 0xC8;
