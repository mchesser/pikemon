extern crate common;
extern crate sdl2;
extern crate gb_emu;

use std::collections::HashMap;
use std::io::TcpStream;
use std::comm::{channel, Sender, Receiver};
use std::io::File;
use gb_emu::emulator::Emulator;
use common::PlayerData;

use net::{NetworkManager, ClientDataManager};

mod client;
mod timer;
mod net;

fn main() {
    let mut socket = TcpStream::connect("127.0.0.1:8080").unwrap();
    let id = socket.read_le_u32().unwrap();

    let (local_update_sender, local_update_receiver) = channel();
    let (global_update_sender, global_update_receiver) = channel();

    let network_manager = NetworkManager {
        socket: socket,
        local_update_receiver: local_update_receiver,
        global_update_sender: global_update_sender,
    };
    spawn(move|| net::handle_network(network_manager));

    let mut emulator = box Emulator::new(|_cpu, _mem| net::collision_manager());
    let cart = File::open(&Path::new("Pokemon Red.gb")).read_to_end().unwrap();
    emulator.load_cart(cart.as_slice());
    emulator.start();

    let client_data_manager = ClientDataManager {
        other_players: HashMap::new(),
        last_state: PlayerData::new(id),
        local_update_sender: local_update_sender,
        global_update_receiver: global_update_receiver,
    };
    client::run(client_data_manager, emulator);
}
