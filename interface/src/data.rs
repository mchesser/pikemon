use values::{moves, types, status, pokeid};
use values::Direction;

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

#[derive(Clone, RustcEncodable, RustcDecodable)]
pub struct Party {
    pub num_pokemon: u8,
    pub pokemon: (PokemonData, PokemonData, PokemonData, PokemonData, PokemonData, PokemonData),
}

pub const BATTLE_DATA_SIZE: usize = 0x194;
#[derive(Clone, RustcEncodable, RustcDecodable)]
pub struct BattleData {
    // Should be [u8; BATTLE_DATA_SIZE], but needs to be a Vec to be encodable
    // (due to rust limitations)
    pub data: Vec<u8>,
}

#[derive(Clone, RustcEncodable, RustcDecodable)]
#[allow(missing_copy_implementations)]
pub struct PokemonData {
    pub species: u8,
    pub hp: u16,
    pub unknown: u8, // TODO: Determine what this does
    pub status: u8,
    pub type1: u8,
    pub type2: u8,
    pub catch_rate: u8,
    pub moves: (u8, u8, u8, u8),
    pub ot_id: u16,

    pub exp: (u8, u8, u8),
    pub hp_ev: u16,
    pub attack_ev: u16,
    pub defense_ev: u16,
    pub speed_ev: u16,
    pub special_ev: u16,
    pub individual_values: (u8, u8),
    pub move_pp: (u8, u8, u8, u8),

    pub level: u8,
    pub max_hp: u16,
    pub attack: u16,
    pub defense: u16,
    pub speed: u16,
    pub special: u16,
}

impl PokemonData {
    pub fn test_data() -> PokemonData {
        PokemonData {
            species: pokeid::WEEDLE,
            hp: 10,
            unknown: 0,
            status: status::NONE,
            type1: types::BUG,
            type2: types::NORMAL,
            catch_rate: 0,
            moves: (moves::POUND, moves::NONE, moves::NONE, moves::NONE),
            ot_id: 0x1234,

            exp: (0, 0, 0),
            hp_ev: 0,
            attack_ev: 0,
            defense_ev: 0,
            speed_ev: 0,
            special_ev: 0,
            individual_values: (0, 0),
            move_pp: (20, 0, 0, 0),

            level: 10,
            max_hp: 20,
            attack: 10,
            defense: 10,
            speed: 10,
            special: 10,
        }
    }
}