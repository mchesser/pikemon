use std::cell::RefCell;
use std::thread::Thread;
use std::sync::mpsc::{Sender, Receiver};
use std::io::{TcpStream, BufferedReader};

use rustc_serialize::json;

use common::{NetworkEvent, PlayerData, PlayerId};
use common::data::Party;
use interface::{self, GameData, NetworkRequest, GameState};
use gb_emu::mmu::Memory;

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
            // println!("Sending update");
            self.last_state = new_state;
            self.local_update_sender.send(NetworkEvent::Update(self.id, new_state));
        }

        match self.game_data.borrow_mut().network_request {
            NetworkRequest::None => {},
            NetworkRequest::Battle(id) => {
                println!("Requesting battle");
                self.local_update_sender.send(NetworkEvent::BattleDataRequest(id, self.id));
            },
        }
        self.game_data.borrow_mut().network_request = NetworkRequest::None;
    }

    pub fn recv_update(&mut self, mem: &mut Memory) {
        match self.global_update_receiver.try_recv() {
            Ok(NetworkEvent::Update(id, data)) => self.handle_update(id, data),
            Ok(NetworkEvent::PlayerQuit(id)) => self.handle_quit(id),
            Ok(NetworkEvent::BattleDataRequest(_, id)) => self.send_battle_data(id, mem),
            Ok(NetworkEvent::BattleDataResponse(_, party)) => self.handle_battle_data(party, mem),

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

    fn send_battle_data(&self, to: PlayerId, mem: &mut Memory) {
        println!("Responding to battle request");
        let party = interface::extract_player_party(mem);
        self.local_update_sender.send(NetworkEvent::BattleDataResponse(to, party));
    }

    fn handle_battle_data(&mut self, party: Party, mem: &mut Memory) {
        self.game_data.borrow_mut().game_state = GameState::Normal;
        interface::set_battle(mem, party);
    }
}
