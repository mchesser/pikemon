extern crate serialize;

#[deriving(Copy, Show, PartialEq, Encodable, Decodable)]
pub struct PlayerData {
    pub player_id: u32,
    pub map_id: u8,
    pub pos_x: i32,
    pub pos_y: i32,
    pub map_x: u8,
    pub map_y: u8,
    pub sprite_index: u8,
    pub direction: u8,
}

impl PlayerData {
    pub fn new(id: u32) -> PlayerData {
        PlayerData {
            player_id: id,
            map_id: 0xFF,
            pos_x: 0,
            pos_y: 0,
            map_x: 0xFF,
            map_y: 0xFF,
            sprite_index: 0xFF,
            direction: 0xFF,
        }
    }
}
