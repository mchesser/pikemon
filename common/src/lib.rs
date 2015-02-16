#![feature(core, io)]
extern crate "rustc-serialize" as rustc_serialize;

pub mod error;
pub mod data;

pub type PlayerId = u32;

#[derive(Clone, RustcEncodable, RustcDecodable)]
pub enum NetworkEvent {
    FullUpdate(PlayerId, PlayerData),
    MovementUpdate(PlayerId, MovementData),
    UpdateRequest,
    PlayerJoin(PlayerId),
    PlayerQuit(PlayerId),
    Chat(PlayerId, String),
    BattleDataRequest(PlayerId, PlayerId),
    BattleDataResponse(PlayerId, data::PlayerBattleData),
    ServerFailure,
}

#[derive(Clone, Debug, PartialEq, RustcEncodable, RustcDecodable)]
pub struct PlayerData {
    pub name: Vec<u8>,
    // pub sprite: Vec<u8>, -- TODO: allow players to choose their own sprite
    // pub sprite_data: SpriteData,
    pub movement_data: MovementData,
}

impl PlayerData {
    pub fn new() -> PlayerData {
        PlayerData {
            name: vec![],
            movement_data: MovementData::new(),
        }
    }

    /// Check if this player is occupying a particular tile
    pub fn check_collision(&self, x: u8, y: u8) -> bool {
        (x, y) == (self.movement_data.map_x, self.movement_data.map_y) ||
            (x, y) == self.movement_data.move_target()
    }

    /// Check if one player is visible to another player
    pub fn is_visible_to(&self, other: &PlayerData) -> bool {
        self.movement_data.map_id == other.movement_data.map_id
    }
}

#[derive(Copy, Clone, Debug, PartialEq, FromPrimitive, RustcEncodable, RustcDecodable)]
pub enum Direction {
    Down = 0x0,
    Up = 0x4,
    Left = 0x8,
    Right = 0xC
}

#[derive(Copy, Clone, Debug, PartialEq, RustcEncodable, RustcDecodable)]
pub struct MovementData {
    pub map_id: u8,
    pub map_x: u8,
    pub map_y: u8,
    pub direction: Direction,
    pub walk_counter: u8,
}

impl MovementData {
    pub fn new() -> MovementData {
        MovementData {
            map_id: 0,
            map_x: 0,
            map_y: 0,
            direction: Direction::Down,
            walk_counter: 0,
        }
    }

    /// Returns the tile that the player is currently moving towards
    pub fn move_target(&self) -> (u8, u8) {
        if self.walk_counter != 0 {
            match self.direction {
                Direction::Down  => (self.map_x, self.map_y + 1),
                Direction::Up    => (self.map_x, self.map_y - 1),
                Direction::Left  => (self.map_x - 1, self.map_y),
                Direction::Right => (self.map_x + 1, self.map_y),
            }
        }
        else {
            (self.map_x, self.map_y)
        }
    }
}

/// The sprite data for a 16x16 sprite
#[derive(Copy)]
pub struct SpriteData {
    pub x: isize,
    pub y: isize,
    pub index: usize,
    pub flags: u8,
}
