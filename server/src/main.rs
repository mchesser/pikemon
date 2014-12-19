extern crate common;

use std::collections::HashMap;
use std::io::{TcpListener, TcpStream};
use std::io::{Acceptor, Listener};
use std::comm::{channel, Sender, Receiver};

use common::PlayerData;

struct Client {
    id: u32,
    stream: TcpStream,
    receiver: Receiver<Packet>,
    sender: Sender<Packet>,
}

#[deriving(Show)]
enum Packet {
    Update(PlayerData),
    PlayerQuit(u32),
}

fn run_server(bind_addr: &str) {
    let listener = TcpListener::bind(bind_addr).unwrap();

    let (new_client_sender, new_client_receiver) = channel();
    let (packet_sender, packet_receiver) = channel();

    spawn(move|| acceptor(listener, new_client_sender, packet_sender));

    let mut clients: HashMap<u32, Sender<Packet>> = HashMap::new();
    loop {
        select! {
            // Handle new player updates and send them to the other clients
            player_packet = packet_receiver.recv() => {
                println!("Received: {}", player_packet);
                match player_packet {
                    Packet::Update(data) => {
                        let sender_id = data.player_id;
                        for (&client_id, client_sender) in clients.iter_mut() {
                            if client_id != sender_id {
                                client_sender.send(Packet::Update(data));
                            }
                        }
                    },

                    Packet::PlayerQuit(id) => {
                        println!("Player: {} disconnected", id);
                        for (_, client_sender) in clients.iter_mut() {
                            client_sender.send(Packet::PlayerQuit(id));
                        }
                        clients.remove(&id);
                    },
                }
            },

            // Handle new clients
            (id, sender) = new_client_receiver.recv() => {
                println!("New client connected, id: {}", id);
                clients.insert(id, sender);
            }
        }
    }
}

fn acceptor(listener: TcpListener, new_client_sender: Sender<(u32, Sender<Packet>)>,
    packet_sender: Sender<Packet>)
{
    let mut acceptor = listener.listen();
    let mut next_id = 0;

    for stream in acceptor.incoming() {
        match stream {
            Err(e) => println!("Connection failed: {}", e),

            Ok(stream) => {
                let (server_sender, client_receiver) = channel();
                let client = Client {
                    id: next_id,
                    stream: stream,
                    receiver: client_receiver,
                    sender: packet_sender.clone(),
                };

                spawn(move|| client_handler(client));
                new_client_sender.send((next_id, server_sender));

                next_id += 1;
            },
        }
    }
}

fn client_handler(mut client: Client) {
    // Before the client can respond, it needs to know it's id
    let id = client.id;
    client.stream.write_le_u32(client.id);

    let mut data_reciever = client.stream.clone();
    let mut data_sender = client.stream;
    let sender = client.sender;
    let receiver = client.receiver;

    // Receive data from client
    spawn(move|| {
        let id = id;
        let mut buffer = [0_u8, ..8];

        loop {
            match data_reciever.read_at_least(8, &mut buffer) {
                Ok(_) => {
                    let packet = Packet::Update(PlayerData::from_bytes(buffer));
                    sender.send(packet);
                },

                Err(e) => {
                    let packet = Packet::PlayerQuit(id);
                    sender.send(packet);
                    break;
                },
            }
        }
    });

    // Send data to client
    loop {
        let packet = match receiver.recv() {
            Packet::Update(data) => data.to_bytes(),
            Packet::PlayerQuit(_) => break,
        };
        data_sender.write(&packet);
    }
}

fn main() {
    run_server("127.0.0.1:8080");
}
