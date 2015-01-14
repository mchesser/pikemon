#![feature(old_orphan_check)]
extern crate "rustc-serialize" as rustc_serialize;

pub mod error;
pub mod data;

pub type PlayerId = u32;

#[derive(Clone, RustcEncodable, RustcDecodable)]
pub enum NetworkEvent {
    Update(PlayerId, PlayerData),
    UpdateRequest,
    PlayerJoin(PlayerId),
    PlayerQuit(PlayerId),
    Chat(PlayerId, String),
    BattleDataRequest(PlayerId, PlayerId),
    BattleDataResponse(PlayerId, data::Party),
    ServerFailure,
}

#[derive(Clone, Show, PartialEq, RustcEncodable, RustcDecodable)]
pub struct PlayerData {
    pub map_id: u8,
    pub map_x: u8,
    pub map_y: u8,
    pub direction: u8,
    pub walk_counter: u8,
}

impl PlayerData {
    pub fn new() -> PlayerData {
        PlayerData {
            map_id: 0,
            map_x: 0,
            map_y: 0,
            direction: 0,
            walk_counter: 0,
        }
    }

    /// Check if this player is occupying a particular tile
    pub fn check_collision(&self, x: u8, y: u8) -> bool {
        if self.map_x == x && self.map_y == y {
            return true;
        }

        if self.walk_counter != 0 {
            return (x, y) == match self.direction {
                0  => (self.map_x, self.map_y + 1),
                4  => (self.map_x, self.map_y - 1),
                8  => (self.map_x - 1, self.map_y),
                12 => (self.map_x + 1, self.map_y),
                _  => return false
            };
        }

        false
    }
}
