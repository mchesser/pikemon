use common::{NetworkEvent, PlayerData, PlayerId};
use interface::GameData;

use std::cell::RefCell;
use std::thread::Thread;
use std::sync::mpsc::{Sender, Receiver};
use std::io::{TcpStream, BufferedReader};

use rustc_serialize::json;

pub struct NetworkManager {
    pub socket: TcpStream,
    pub local_update_receiver: Receiver<NetworkEvent>,
    pub global_update_sender: Sender<NetworkEvent>,
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
        let packet = json::encode(&local_update_receiver.recv().unwrap());

        // TODO: better error handling
        let _ = sender_socket.write_str(&*packet);
        let _ = sender_socket.write_char('\n');
    }
}

pub struct ClientDataManager<'a> {
    pub id: PlayerId,
    pub game_data: &'a RefCell<GameData>,
    pub last_state: PlayerData,
    pub local_update_sender: Sender<NetworkEvent>,
    pub global_update_receiver: Receiver<NetworkEvent>,
}

impl<'a> ClientDataManager<'a> {
    pub fn send_update(&mut self, new_state: PlayerData) {
        if self.last_state != new_state {
            self.last_state = new_state;
            self.local_update_sender.send(NetworkEvent::Update(self.id, new_state));
        }
    }

    pub fn recv_update(&mut self) {
        match self.global_update_receiver.try_recv() {
            Ok(NetworkEvent::Update(id, data)) => self.handle_update(id, data),
            Ok(NetworkEvent::PlayerQuit(id)) => self.handle_quit(id),

            Ok(_) => unimplemented!(),
            _ => {},
        }
    }

    fn handle_update(&mut self, id: PlayerId, data: PlayerData) {
        self.game_data.borrow_mut().other_players.insert(id, data);
    }

    fn handle_quit(&mut self, id: PlayerId) {
        self.game_data.borrow_mut().other_players.remove(&id);
    }
}
