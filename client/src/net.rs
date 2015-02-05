use std::mem;
use std::thread::Thread;
use std::sync::mpsc::{Sender, Receiver};
use std::old_io::{TcpStream, BufferedReader};

use rustc_serialize::json;

use common::{NetworkEvent, MovementData, PlayerData, PlayerId};
use common::error::{NetworkError, NetworkResult};

use interface::{self, NetworkRequest, InterfaceState};
use interface::{text, extract};
use game::Game;

pub struct NetworkManager {
    pub socket: TcpStream,
    pub local_update_receiver: Receiver<NetworkEvent>,
    pub global_update_sender: Sender<NetworkEvent>,
}

pub fn handle_network(network_manager: NetworkManager) -> NetworkResult<PlayerId> {
    let mut receiver_socket = BufferedReader::new(network_manager.socket.clone());

    let join_line = try!(receiver_socket.read_line());
    let player_id = match json::decode(&*join_line) {
        Ok(NetworkEvent::PlayerJoin(id)) => id,
        _ => return Err(NetworkError::DecodeError),
    };

    let global_update_sender = network_manager.global_update_sender;
    Thread::spawn(move|| {
        loop {
            match receiver_socket.read_line() {
                Ok(data) => {
                    let packet = json::decode(&*data).unwrap();
                    // TODO: better error handling
                    let _ = global_update_sender.send(packet);
                },

                Err(e) => {
                    println!("Disconnected from server: {}", e);
                    break;
                },
            }
        }
    });

    let local_update_receiver = network_manager.local_update_receiver;
    let mut sender_socket = network_manager.socket;
    Thread::spawn(move|| {
        loop {
            let packet = json::encode(&local_update_receiver.recv().unwrap()).unwrap();

            // TODO: better error handling
            let _ = sender_socket.write_str(&*packet);
            let _ = sender_socket.write_char('\n');
        }
    });

    Ok(player_id)
}


pub struct ClientManager {
    id: PlayerId,
    last_state: PlayerData,
    full_update: Option<PlayerData>,
    movement_update: Option<MovementData>,
    update_sender: Sender<NetworkEvent>,
    update_receiver: Receiver<NetworkEvent>,
}

impl ClientManager {
    pub fn new(id: PlayerId, update_sender: Sender<NetworkEvent>,
        update_receiver: Receiver<NetworkEvent>) -> ClientManager
    {
        ClientManager {
            id: id,
            last_state: PlayerData::new(),
            full_update: None,
            movement_update: None,
            update_sender: update_sender,
            update_receiver: update_receiver,
        }
    }

    pub fn update_player(&mut self, new_data: &PlayerData) {
        if self.last_state.movement_data != new_data.movement_data {
            self.last_state.movement_data = new_data.movement_data;
            self.movement_update = Some(new_data.movement_data);
        }

        if self.last_state != *new_data {
            self.last_state = new_data.clone();
            self.full_update = Some(new_data.clone());
        }
    }

    pub fn send_update(&mut self, game: &mut Game) -> NetworkResult<()> {
        if self.movement_update.is_some() {
            let update_data = mem::replace(&mut self.movement_update, None).unwrap();
            try!(self.update_sender.send(NetworkEvent::MovementUpdate(self.id, update_data)));
        }

        if self.full_update.is_some() {
            let update_data = mem::replace(&mut self.full_update, None).unwrap();
            try!(self.update_sender.send(NetworkEvent::FullUpdate(self.id, update_data)));
        }

        if game.chat_box.message_ready {
            try!(self.send_message(game));
        }

        match game.interface_data.borrow().network_request {
            NetworkRequest::None => {},
            NetworkRequest::Battle(id) => {
                println!("Requesting battle");
                try!(self.update_sender.send(NetworkEvent::BattleDataRequest(id, self.id)));
            },
        }

        game.interface_data.borrow_mut().network_request = NetworkRequest::None;
        Ok(())
    }

    pub fn recv_update(&mut self, game: &mut Game) -> NetworkResult<()> {
        let interface_data = &mut *game.interface_data.borrow_mut();
        loop {
            match self.update_receiver.try_recv() {
                Ok(NetworkEvent::FullUpdate(id, update_data)) => {
                    interface_data.players.insert(id, update_data);
                },

                Ok(NetworkEvent::MovementUpdate(id, update_data)) => {
                    if let Some(player) = interface_data.players.get_mut(&id) {
                        player.movement_data = update_data;
                    }
                },

                Ok(NetworkEvent::PlayerQuit(id)) => {
                    println!("Player: {} quit.", id);
                    interface_data.players.remove(&id);
                },

                Ok(NetworkEvent::BattleDataRequest(_, id)) => {
                    println!("Responding to battle request");
                    let data = extract::battle_data(&game.emulator.mem);
                    try!(self.update_sender.send(NetworkEvent::BattleDataResponse(id, data)));
                },

                Ok(NetworkEvent::BattleDataResponse(_, battle_data)) => {
                    interface_data.state = InterfaceState::Normal;
                    let enemy_id = interface_data.last_interaction;
                    if let Some(enemy) = interface_data.players.get(&enemy_id) {
                        interface::set_battle(&mut game.emulator.mem, enemy, battle_data);
                    }
                },

                Ok(NetworkEvent::UpdateRequest) => {
                    println!("Responding to update request");
                    let update_data = game.player_data.clone();
                    try!(self.update_sender.send(NetworkEvent::FullUpdate(self.id, update_data)));
                },

                Ok(NetworkEvent::Chat(id, msg)) => {
                    let player_name = match interface_data.players.get(&id) {
                        Some(player) => player.name.clone(),
                        None => text::Encoder::new("UNKNOWN").collect(),
                    };
                    game.chat_box.add_message(player_name, text::Encoder::new(&*msg).collect());
                },

                Ok(_) => unimplemented!(),
                _ => break,
            }
        }

        Ok(())
    }

    pub fn send_message(&mut self, game: &mut Game) -> NetworkResult<()> {
        let msg = game.chat_box.get_message_buffer();
        let user_name = game.player_data.name.clone();

        game.chat_box.add_message(user_name, text::Encoder::new(&*msg).collect());
        try!(self.update_sender.send(NetworkEvent::Chat(self.id, msg)));

        Ok(())
    }
}
