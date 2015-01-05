extern crate "rustc-serialize" as rustc_serialize;
extern crate common;

use rustc_serialize::json;

use std::thread::Thread;
use std::collections::HashMap;
use std::io::{TcpListener, TcpStream, BufferedReader};
use std::io::{Acceptor, Listener};
use std::sync::mpsc::{channel, Sender, Receiver};

use common::PlayerData;

struct Client {
    id: u32,
    stream: TcpStream,
    receiver: Receiver<Packet>,
    sender: Sender<Packet>,
}

enum Packet {
    Update(PlayerData),
    PlayerQuit(u32),
}

fn run_server(bind_addr: &str) {
    let listener = TcpListener::bind(bind_addr).unwrap();

    let (new_client_sender, new_client_receiver) = channel();
    let (packet_sender, packet_receiver) = channel();

    Thread::spawn(move|| acceptor(listener, new_client_sender, packet_sender)).detach();

    let mut clients: HashMap<u32, Sender<Packet>> = HashMap::new();
    loop {
        select! {
            // Handle new player updates and send them to the other clients
            player_packet = packet_receiver.recv() => {
                match player_packet.unwrap() {
                    Packet::Update(data) => {
                        let sender_id = data.player_id;
                        for (&client_id, client_sender) in clients.iter_mut() {
                            if client_id != sender_id {
                                // TODO: Better error handling
                                let _ = client_sender.send(Packet::Update(data));
                            }
                        }
                    },

                    Packet::PlayerQuit(id) => {
                        println!("Player: {} disconnected", id);
                        for (_, client_sender) in clients.iter_mut() {
                            // TODO: Better error handling
                            let _= client_sender.send(Packet::PlayerQuit(id));
                        }
                        clients.remove(&id);
                    },
                }
            },

            // Handle new clients
            packet = new_client_receiver.recv() => {
                let (id, sender) = packet.unwrap();
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

                Thread::spawn(move|| client_handler(client)).detach();
                // TODO: Better error handling
                let _ = new_client_sender.send((next_id, server_sender));

                next_id += 1;
            },
        }
    }
}

fn client_handler(mut client: Client) {
    // Before the client can respond, it needs to know it's id
    let id = client.id;
    if let Err(e) = client.stream.write_le_u32(client.id) {
        println!("Failed to connect with client: {}", e);
    }

    let mut data_reciever = BufferedReader::new(client.stream.clone());
    let mut data_sender = client.stream;
    let sender = client.sender;
    let receiver = client.receiver;

    // Receive data from client
    Thread::spawn(move|| {
        let id = id;

        loop {
            match data_reciever.read_line() {
                Ok(data) => {
                    let packet = Packet::Update(json::decode(&*data).unwrap());
                    // TODO: Better error handling
                    let _ = sender.send(packet);
                },

                Err(_) => {
                    let packet = Packet::PlayerQuit(id);
                    // TODO: Better error handling
                    let _ = sender.send(packet);
                    break;
                },
            }
        }
    }).detach();

    // Send data to client
    loop {
        let packet = match receiver.recv().unwrap() {
            Packet::Update(data) => json::encode(&data),
            Packet::PlayerQuit(_) => break,
        };

        // TODO: properly handle errors here
        let _ = data_sender.write_str(&*packet);
        let _ = data_sender.write_char('\n');
    }
}

fn main() {
    run_server("127.0.0.1:8080");
}
