#[deriving(Copy, Show, PartialEq)]
#[repr(C)]
pub struct PlayerData {
    pub player_id: u32,
    pub map_id: u8,
    pub pos_x: u8,
    pub pos_y: u8,
    pub direction: u8,
}

pub type PlayerDataPacket = [u8, ..8];

impl PlayerData {
    pub fn new(id: u32) -> PlayerData {
        PlayerData {
            player_id: id,
            map_id: 0xFF,
            pos_x: 0xFF,
            pos_y: 0xFF,
            direction: 0xFF,
        }
    }

    pub fn to_bytes(&self) -> PlayerDataPacket {
        let id = self.player_id;
        [id as u8, (id >> 8) as u8, (id >> 16) as u8, (id >> 24) as u8,
            self.map_id, self.pos_x, self.pos_y, self.direction]
    }

    pub fn from_bytes(b: PlayerDataPacket) -> PlayerData {
        let id = b[0] as u32 | b[1] as u32 << 8 | b[2] as u32 << 16 | b[3] as u32 <<  24;

        PlayerData {
            player_id: id,
            map_id: b[4],
            pos_x: b[5],
            pos_y: b[6],
            direction: b[7],
        }
    }
}
