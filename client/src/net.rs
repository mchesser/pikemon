use common::PlayerData;
use interface::GameData;

use std::cell::RefCell;
use std::thread::Thread;
use std::comm::{Sender, Receiver};
use std::io::{TcpStream, BufferedReader};

use serialize::json;

pub struct NetworkManager {
    pub socket: TcpStream,
    pub local_update_receiver: Receiver<PlayerData>,
    pub global_update_sender: Sender<PlayerData>,
}

pub fn handle_network(network_manager: NetworkManager) {
    let mut receiver_socket = BufferedReader::new(network_manager.socket.clone());
    let global_update_sender = network_manager.global_update_sender;

    Thread::spawn(move|| {
        loop {
            match receiver_socket.read_line() {
                Ok(data) => {
                    let packet = json::decode(&*data).unwrap();
                    global_update_sender.send(packet);
                },

                Err(e) => {
                    println!("Disconnected from server: {}", e);
                    break;
                },
            }
        }
    }).detach();

    let local_update_receiver = network_manager.local_update_receiver;
    let mut sender_socket = network_manager.socket;
    loop {
        let packet = json::encode(&local_update_receiver.recv());

        // TODO: better error handling
        let _ = sender_socket.write_str(&*packet);
        let _ = sender_socket.write_char('\n');
    }
}

pub struct ClientDataManager<'a> {
    pub game_data: &'a RefCell<GameData>,
    pub last_state: PlayerData,
    pub local_update_sender: Sender<PlayerData>,
    pub global_update_receiver: Receiver<PlayerData>,
}

impl<'a> ClientDataManager<'a> {
    pub fn update(&mut self, new_state: PlayerData) {
        if self.last_state != new_state {
            self.last_state = new_state;
            self.local_update_sender.send(new_state);
        }

        match self.global_update_receiver.try_recv() {
            Ok(update) => self.handle_recv(update),
            _ => {},
        }
    }

    fn handle_recv(&mut self, update: PlayerData) {
        // TODO: handle disconnecting players
        self.game_data.borrow_mut().other_players.insert(update.player_id, update);
    }
}
