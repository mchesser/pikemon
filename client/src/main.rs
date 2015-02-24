#![feature(core, box_syntax, old_path, old_io, env)]

extern crate "rustc-serialize" as rustc_serialize;
extern crate interface;
extern crate network_common;
extern crate sdl2;
extern crate gb_emu;

use std::old_io::{File, TcpStream};
use std::sync::mpsc::channel;

use gb_emu::emulator::Emulator;
use gb_emu::cart;

use net::{NetworkManager, ClientManager};
use save::LocalSaveWrapper;

mod client;
mod game;
mod timer;
mod net;
mod font;
mod chat;
mod save;

fn main() {
    let args = std::env::args();

    let socket = match args.skip(1).next() {
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

    let mut emulator = box Emulator::new();

    let cart = File::open(&Path::new("Pokemon Red.gb")).read_to_end().unwrap();
    let save_path = Path::new("Pokemon Red.sav");

    let save_file = Box::new(LocalSaveWrapper { path: save_path }) as Box<cart::SaveFile>;
    emulator.load_cart(&cart, Some(save_file));
    emulator.start();

    let client_manager = ClientManager::new(id, local_update_sender, global_update_receiver);

    if let Err(e) = client::run(client_manager, emulator) {
        println!("Pikemon encountered an error and was forced to close. ({})", e);
    }
}
