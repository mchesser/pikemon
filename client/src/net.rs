use std::{
    io::{prelude::*, BufReader},
    mem,
    net::TcpStream,
    thread,
};

use crossbeam_channel::{Receiver, Sender};
use interface::{
    self,
    data::{MovementData, PlayerData},
    extract, text, InterfaceState, NetworkRequest,
};
use network_common::{
    error::{NetworkError, NetworkResult},
    NetworkEvent, PlayerId,
};

use crate::game::Game;

pub struct NetworkManager {
    pub socket: TcpStream,
    pub local_update_receiver: Receiver<NetworkEvent>,
    pub global_update_sender: Sender<NetworkEvent>,
}

pub fn handle_network(network_manager: NetworkManager) -> NetworkResult<PlayerId> {
    let mut receiver_socket = BufReader::new(network_manager.socket.try_clone()?);
    let mut data_buffer = String::new();

    receiver_socket.read_line(&mut data_buffer)?;
    let player_id = match serde_json::from_str(&data_buffer) {
        Ok(NetworkEvent::PlayerJoin(id)) => id,
        _ => return Err(NetworkError::DecodeError),
    };
    data_buffer.clear();

    let global_update_sender = network_manager.global_update_sender;
    thread::spawn(move || {
        loop {
            match receiver_socket.read_line(&mut data_buffer) {
                Ok(_) => {
                    let packet = serde_json::from_str(&data_buffer).unwrap();
                    // TODO: better error handling
                    let _ = global_update_sender.send(packet);
                }

                Err(e) => {
                    println!("Disconnected from server: {}", e);
                    break;
                }
            }
            data_buffer.clear();
        }
    });

    let local_update_receiver = network_manager.local_update_receiver;
    let mut sender_socket = network_manager.socket;
    thread::spawn(move || {
        loop {
            let packet = serde_json::to_vec(&local_update_receiver.recv().unwrap()).unwrap();

            // TODO: better error handling
            let _ = sender_socket.write(&packet);
            let _ = sender_socket.write(b"\n");
        }
    });

    Ok(player_id)
}

pub struct ClientManager {
    id: PlayerId,
    last_state: Option<PlayerData>,
    full_update: Option<PlayerData>,
    movement_update: Option<MovementData>,
    update_sender: Sender<NetworkEvent>,
    update_receiver: Receiver<NetworkEvent>,
}

impl ClientManager {
    pub fn new(
        id: PlayerId,
        update_sender: Sender<NetworkEvent>,
        update_receiver: Receiver<NetworkEvent>,
    ) -> ClientManager {
        ClientManager {
            id,
            last_state: None,
            full_update: None,
            movement_update: None,
            update_sender,
            update_receiver,
        }
    }

    pub fn update_player(&mut self, new_data: &PlayerData) {
        if let Some(ref last_state) = self.last_state {
            if last_state.movement_data != new_data.movement_data {
                self.movement_update = Some(new_data.movement_data);
            }
        }
        if self.last_state.as_ref() != Some(new_data) {
            self.last_state = Some(new_data.clone());
            self.full_update = Some(new_data.clone());
        }
    }

    pub fn send_update(&mut self, game: &mut Game) -> NetworkResult<()> {
        if self.movement_update.is_some() {
            let update_data = mem::replace(&mut self.movement_update, None).unwrap();
            self.update_sender
                .send(NetworkEvent::MovementUpdate(self.id, update_data))
                .map_err(|_| NetworkError::SendError)?;
        }

        if self.full_update.is_some() {
            let update_data = mem::replace(&mut self.full_update, None).unwrap();
            self.update_sender
                .send(NetworkEvent::FullUpdate(self.id, update_data))
                .map_err(|_| NetworkError::SendError)?;
        }

        if game.chat_box.message_ready {
            self.send_message(game)?;
        }

        match game.interface_data.borrow().network_request {
            NetworkRequest::None => {}
            NetworkRequest::Battle(id) => {
                println!("Requesting battle");
                self.update_sender
                    .send(NetworkEvent::BattleDataRequest(id, self.id))
                    .map_err(|_| NetworkError::SendError)?;
            }
        }

        game.interface_data.borrow_mut().network_request = NetworkRequest::None;
        Ok(())
    }

    pub fn recv_update(&mut self, game: &mut Game) -> NetworkResult<()> {
        let interface_data = &mut game.interface_data.borrow_mut();
        loop {
            match self.update_receiver.try_recv() {
                Ok(NetworkEvent::FullUpdate(id, update_data)) => {
                    interface_data.players.insert(id, update_data);
                }

                Ok(NetworkEvent::MovementUpdate(id, update_data)) => {
                    if let Some(player) = interface_data.players.get_mut(&id) {
                        player.movement_data = update_data;
                    }
                }

                Ok(NetworkEvent::PlayerQuit(id)) => {
                    println!("Player: {} quit.", id);
                    interface_data.players.remove(&id);
                }

                Ok(NetworkEvent::BattleDataRequest(_, id)) => {
                    println!("Responding to battle request");
                    let data = extract::battle_data(&game.emulator.mem);
                    self.update_sender
                        .send(NetworkEvent::BattleDataResponse(id, data))
                        .map_err(|_| NetworkError::SendError)?;
                }

                Ok(NetworkEvent::BattleDataResponse(_, battle_data)) => {
                    interface_data.state = InterfaceState::Normal;
                    let enemy_id = interface_data.last_interaction;
                    if let Some(enemy) = interface_data.players.get(&enemy_id) {
                        interface::set_battle(&mut game.emulator.mem, enemy, battle_data);
                    }
                }

                Ok(NetworkEvent::UpdateRequest) => {
                    println!("Responding to update request");
                    let update_data = game.player_data.clone();
                    self.update_sender
                        .send(NetworkEvent::FullUpdate(self.id, update_data))
                        .map_err(|_| NetworkError::SendError)?;
                }

                Ok(NetworkEvent::Chat(id, msg)) => {
                    let player_name = match interface_data.players.get(&id) {
                        Some(player) => player.name.clone(),
                        None => text::Encoder::new("UNKNOWN").collect(),
                    };
                    game.chat_box.add_message(player_name, text::Encoder::new(&*msg).collect());
                }

                Ok(_) => unimplemented!(),
                _ => break,
            }
        }

        Ok(())
    }

    pub fn send_message(&mut self, game: &mut Game) -> NetworkResult<()> {
        let msg = game.chat_box.get_message_buffer();
        let user_name = game.player_data.name.clone();

        game.chat_box.add_message(user_name, text::Encoder::new(&msg).collect());
        self.update_sender
            .send(NetworkEvent::Chat(self.id, msg))
            .map_err(|_| NetworkError::SendError)?;

        Ok(())
    }
}
