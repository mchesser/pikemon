extern crate rustc_serialize;
extern crate interface;
extern crate network_common;
extern crate sdl2;
extern crate gb_emu;

use std::io::prelude::*;
use std::path::Path;
use std::fs::File;

use std::net::TcpStream;
use std::sync::mpsc::channel;

use gb_emu::emulator::Emulator;
use gb_emu::cart;

use net::{NetworkManager, ClientManager};
use save::LocalSaveWrapper;

mod common;
mod client;
mod game;
mod net;
mod border;
mod font;
mod chat;
mod menu;
mod save;

fn main() {
    let socket = match std::env::args().nth(1) {
        Some(ip_addr) => TcpStream::connect((&*ip_addr, 8080)).unwrap(),
        // Assume localhost if there was no argument specified
        None => TcpStream::connect(("localhost", 8080)).unwrap(),
    };

    let (local_update_sender, local_update_receiver) = channel();
    let (global_update_sender, global_update_receiver) = channel();

    let network_manager = NetworkManager {
        socket: socket,
        local_update_receiver: local_update_receiver,
        global_update_sender: global_update_sender,
    };
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

    let save_file = Box::new(LocalSaveWrapper { path: save_path }) as Box<cart::SaveFile>;
    emulator.load_cart(&cart, Some(save_file));
    emulator.start();

    let client_manager = ClientManager::new(id, local_update_sender, global_update_receiver);

    if let Err(e) = client::run(client_manager, emulator) {
        println!("Pikemon encountered an error and was forced to close. ({})", e);
    }
}
