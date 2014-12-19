use common::PlayerData;
use std::comm::{channel, Sender, Receiver};
use std::io::TcpStream;
use std::collections::HashMap;

pub struct NetworkManager {
    pub socket: TcpStream,
    pub local_update_receiver: Receiver<PlayerData>,
    pub global_update_sender: Sender<PlayerData>,
}

pub fn handle_network(network_manager: NetworkManager) {
    let mut receiver_socket = network_manager.socket.clone();
    let global_update_sender = network_manager.global_update_sender;

    spawn(move|| {
        let mut buffer = [0_u8, ..8];

        loop {
            match receiver_socket.read_at_least(8, &mut buffer) {
                Ok(_) => {
                    let packet = PlayerData::from_bytes(buffer);
                    global_update_sender.send(packet);
                },

                Err(e) => {
                    panic!("Disconnected from server: {}", e);
                },
            }
        }
    });

    let local_update_receiver = network_manager.local_update_receiver;
    let mut sender_socket = network_manager.socket;
    loop {
        let packet = local_update_receiver.recv().to_bytes();
        sender_socket.write(&packet);
    }
}

pub struct ClientDataManager {
    pub other_players: HashMap<u32, PlayerData>,
    pub last_state: PlayerData,
    pub local_update_sender: Sender<PlayerData>,
    pub global_update_receiver: Receiver<PlayerData>,
}

impl ClientDataManager {
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
        self.other_players.insert(update.player_id, update);
    }
}

pub fn collision_manager() {
    // Currently unimplemented
}
