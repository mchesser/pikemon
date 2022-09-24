use std::{fs::File, io::prelude::*, net::TcpStream, path::Path};

use gb_emu::emulator::Emulator;

use crate::{
    net::{ClientManager, NetworkManager},
    save::LocalSaveWrapper,
};

mod border;
mod chat;
mod client;
mod common;
mod font;
mod game;
mod menu;
mod net;
mod save;

#[macroquad::main("Pikemon")]
async fn main() {
    let socket = match std::env::args().nth(1) {
        Some(ip_addr) => TcpStream::connect((&*ip_addr, 8080)).unwrap(),
        // Assume localhost if there was no argument specified
        None => TcpStream::connect(("localhost", 8080)).unwrap(),
    };

    let (local_update_sender, local_update_receiver) = crossbeam_channel::unbounded();
    let (global_update_sender, global_update_receiver) = crossbeam_channel::unbounded();

    let network_manager = NetworkManager { socket, local_update_receiver, global_update_sender };
    let id = net::handle_network(network_manager).unwrap();

    let mut emulator = Box::new(Emulator::new());

    let cart = {
        let mut data = vec![];
        let mut f = match File::open("Pokemon Red.gb") {
            Ok(f) => f,
            Err(e) => panic!("Error opening 'Pokemon Red.gb': {}", e),
        };
        f.read_to_end(&mut data).unwrap();
        data
    };
    let save_path = Path::new("Pokemon Red.sav");

    let save_file = Box::new(LocalSaveWrapper { path: save_path });
    emulator.load_cart(&cart, Some(save_file));
    emulator.start();

    let client_manager = ClientManager::new(id, local_update_sender, global_update_receiver);

    if let Err(e) = client::run(client_manager, emulator).await {
        println!("Pikemon encountered an error and was forced to close. ({})", e);
    }
}
