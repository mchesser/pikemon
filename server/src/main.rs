#![feature(std_misc)]

extern crate rustc_serialize;
extern crate network_common;
extern crate interface;

use rustc_serialize::json;

use std::thread;
use std::collections::HashMap;

use std::io::prelude::*;
use std::io::{self, BufReader};

use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{channel, Sender};

use interface::PlayerId;
use network_common::NetworkEvent;
use network_common::error::NetworkResult;

struct Client {
    id: PlayerId,
    client_stream: TcpStream,
    server_sender: Sender<NetworkEvent>,
}

fn run_server(bind_addr: &str) -> NetworkResult<()> {
    let listener = try!(TcpListener::bind(bind_addr));

    let (new_client_sender, new_client_receiver) = channel();
    let (packet_sender, packet_receiver) = channel();

    thread::spawn(move|| {
        let _ = acceptor(listener, new_client_sender, packet_sender);
    });

    let mut clients = HashMap::new();
    loop {
        select! {
            player_packet = packet_receiver.recv() => {
                let message = try!(player_packet);
                match message {
                    NetworkEvent::FullUpdate(sender_id, _) |
                    NetworkEvent::MovementUpdate(sender_id, _) |
                    NetworkEvent::Chat(sender_id, _) => {
                        for (&client_id, client_stream) in &mut clients {
                            if client_id != sender_id {
                                send_to_client(client_stream, &message).unwrap();
                            }
                        }
                    },

                    NetworkEvent::PlayerQuit(id) => {
                        clients.remove(&id);
                        println!("Player: {} disconnected", id);
                        for (_, client_stream) in &mut clients {
                            send_to_client(client_stream, &message).unwrap();
                        }
                    },

                    NetworkEvent::BattleDataRequest(to, _) |
                    NetworkEvent::BattleDataResponse(to, _) => {
                        send_to_client(clients.get_mut(&to).unwrap(), &message).unwrap();
                    },

                    _ => unimplemented!(),
                }
            },

            // Handle new clients
            packet = new_client_receiver.recv() => {
                let (id, sender) = try!(packet);
                println!("New client connected, id: {}", id);
                clients.insert(id, sender);

                // Tell connected clients that they need to send an update to the new client
                for (_, client_stream) in &mut clients {
                    send_to_client(client_stream, &NetworkEvent::UpdateRequest).unwrap();
                }
            }
        }
    }
}

fn send_to_client(client_stream: &mut TcpStream, message: &NetworkEvent) -> io::Result<usize> {
    let encoded_message = json::encode(&message).unwrap();
    try!(client_stream.write(encoded_message.as_bytes()));
    client_stream.write("\n".as_bytes())
}

fn acceptor(listener: TcpListener, new_client_sender: Sender<(u32, TcpStream)>,
    server_sender: Sender<NetworkEvent>) -> NetworkResult<()>
{
    let mut next_id = 0;

    for stream in listener.incoming() {
        let mut stream = try!(stream);
        if let Err(e) = send_to_client(&mut stream, &NetworkEvent::PlayerJoin(next_id)) {
            println!("Failed to communicate with client: {}", e);
            continue;
        }

        let client = Client {
            id: next_id,
            client_stream: try!(stream.try_clone()),
            server_sender: server_sender.clone(),
        };

        thread::spawn(move|| {
            let _ = client_handler(client);
        });
        try!(new_client_sender.send((next_id, stream)));

        next_id += 1;
    }

    Ok(())
}

fn client_handler(client: Client) -> NetworkResult<()> {
    let mut client_stream = BufReader::new(client.client_stream);
    let mut data = String::new();
    loop {
        match client_stream.read_line(&mut data) {
            Ok(_) => {
                let packet = json::decode(&data).unwrap();
                try!(client.server_sender.send(packet));
            },

            Err(_) => {
                let packet = NetworkEvent::PlayerQuit(client.id);
                try!(client.server_sender.send(packet));
                return Ok(());
            },
        }
        data.clear();
    }
}

fn main() {
    if let Err(e) = run_server("0.0.0.0:8080") {
        println!("Server failed unexpectedly and had to close.\nReason: {}", e);
    }
}
