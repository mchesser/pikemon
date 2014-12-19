extern crate serialize;

#[deriving(Copy, Show, PartialEq, Encodable, Decodable)]
pub struct PlayerData {
    pub player_id: u32,
    pub map_id: u8,
    pub pos_x: i32,
    pub pos_y: i32,
    pub direction: u8,
}

impl PlayerData {
    pub fn new(id: u32) -> PlayerData {
        PlayerData {
            player_id: id,
            map_id: 0xFF,
            pos_x: 0,
            pos_y: 0,
            direction: 0xFF,
        }
    }
}
