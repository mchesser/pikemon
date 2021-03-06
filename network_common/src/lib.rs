extern crate rustc_serialize;
extern crate interface;

use interface::data::{PlayerData, MovementData, BattleData};

pub mod error;

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
    BattleDataResponse(PlayerId, BattleData),
    ServerFailure,
}
