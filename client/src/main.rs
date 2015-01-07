#![feature(slicing_syntax)]

extern crate "rustc-serialize" as rustc_serialize;
extern crate common;
extern crate sdl2;
extern crate gb_emu;

use std::cell::RefCell;
use std::io::{File, TcpStream, BufferedReader};
use std::thread::Thread;
use std::sync::mpsc::channel;

use rustc_serialize::json;

use gb_emu::emulator::Emulator;
use gb_emu::cart;
use common::{NetworkEvent, PlayerData, PlayerId};
use common::error::{NetworkError, NetworkResult};

use interface::GameData;
use net::{NetworkManager, ClientDataManager};
use save::LocalSaveWrapper;

mod client;
mod timer;
mod net;
mod sprite;
mod interface;
mod save;

fn main() {
    let ip_addr = &*std::os::args()[1];
    let socket = TcpStream::connect((ip_addr, 8080)).unwrap();
    let id = get_id_from_server(socket.clone()).unwrap();

    let (local_update_sender, local_update_receiver) = channel();
    let (global_update_sender, global_update_receiver) = channel();

    let network_manager = NetworkManager {
        socket: socket,
        local_update_receiver: local_update_receiver,
        global_update_sender: global_update_sender,
    };
    Thread::spawn(move|| net::handle_network(network_manager)).detach();

    let game_data = RefCell::new(GameData::new());
    let mut emulator = box Emulator::new(|cpu, mem| {
        interface::sprite_check_hack(cpu, mem, &mut *game_data.borrow_mut());
        interface::display_text_hack(cpu, mem, &mut *game_data.borrow_mut());
    });

    let cart = File::open(&Path::new("Pokemon Red.gb")).read_to_end().unwrap();
    let save_path = Path::new("Pokemon Red.sav");

    let save_file = box LocalSaveWrapper { path: save_path } as Box<cart::SaveFile>;
    emulator.load_cart(cart.as_slice(), Some(save_file));
    emulator.start();

    let client_data_manager = ClientDataManager {
        id: id,
        game_data: &game_data,
        last_state: PlayerData::new(),
        local_update_sender: local_update_sender,
        global_update_receiver: global_update_receiver,
    };

    if let Err(e) = client::run(client_data_manager, emulator) {
        println!("Pikemon encountered an error and was forced to close. ({})", e);
    }
}

fn get_id_from_server(socket: TcpStream) -> NetworkResult<PlayerId> {
    let mut line_reader = BufferedReader::new(socket);
    let line = try!(line_reader.read_line());

    if let Ok(NetworkEvent::PlayerJoin(id)) = json::decode(&*line) {
        return Ok(id);
    }

    Err(NetworkError::DecodeError)
}
