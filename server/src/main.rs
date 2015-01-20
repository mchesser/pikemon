#![allow(unstable)] // This generates a lot of unnecessary warnings at the moment

extern crate "rustc-serialize" as rustc_serialize;
extern crate common;

use rustc_serialize::json;

use std::thread::Thread;
use std::collections::HashMap;
use std::io::{IoResult, TcpListener, TcpStream, BufferedReader};
use std::io::{Acceptor, Listener};
use std::sync::mpsc::{channel, Sender};

use common::{NetworkEvent, PlayerId};
use common::error::NetworkResult;

struct Client {
    id: PlayerId,
    client_stream: TcpStream,
    server_sender: Sender<NetworkEvent>,
}

fn run_server(bind_addr: &str) -> NetworkResult<()> {
    let listener = try!(TcpListener::bind(bind_addr));

    let (new_client_sender, new_client_receiver) = channel();
    let (packet_sender, packet_receiver) = channel();

    Thread::spawn(move|| {
        let _ = acceptor(listener, new_client_sender, packet_sender);
    });

    let mut clients: HashMap<PlayerId, TcpStream> = HashMap::new();
    loop {
        select! {
            player_packet = packet_receiver.recv() => {
                let message = try!(player_packet);
                match message {
                    NetworkEvent::FullUpdate(sender_id, _) |
                    NetworkEvent::MovementUpdate(sender_id, _) |
                    NetworkEvent::Chat(sender_id, _) => {
                        for (&client_id, client_stream) in clients.iter_mut() {
                            if client_id != sender_id {
                                try!(send_to_client(client_stream, &message));
                            }
                        }
                    },

                    NetworkEvent::PlayerQuit(id) => {
                        clients.remove(&id);
                        println!("Player: {} disconnected", id);
                        for (_, client_stream) in clients.iter_mut() {
                            try!(send_to_client(client_stream, &message));
                        }
                    },

                    NetworkEvent::BattleDataRequest(to, _) |
                    NetworkEvent::BattleDataResponse(to, _) => {
                        try!(send_to_client(&mut clients[to], &message))
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
                for (_, client_stream) in clients.iter_mut() {
                    try!(send_to_client(client_stream, &NetworkEvent::UpdateRequest));
                }
            }
        }
    }
}

fn send_to_client(client_stream: &mut TcpStream, message: &NetworkEvent) -> IoResult<()> {
    let encoded_message = json::encode(&message);
    try!(client_stream.write_str(&*encoded_message));
    client_stream.write_char('\n')
}

fn acceptor(listener: TcpListener, new_client_sender: Sender<(u32, TcpStream)>,
    server_sender: Sender<NetworkEvent>) -> NetworkResult<()>
{
    let mut acceptor = listener.listen();
    let mut next_id = 0;

    for stream in acceptor.incoming() {
        let mut stream = try!(stream);
            if let Err(e) = send_to_client(&mut stream, &NetworkEvent::PlayerJoin(next_id)) {
                println!("Failed to communicate with client: {}", e);
                continue;
            }

            let client = Client {
                id: next_id,
                client_stream: stream.clone(),
                server_sender: server_sender.clone(),
            };

            Thread::spawn(move|| {
                let _ = client_handler(client);
            });
            try!(new_client_sender.send((next_id, stream)));

            next_id += 1;
    }

    Ok(())
}

fn client_handler(client: Client) -> NetworkResult<()> {
    let mut client_stream = BufferedReader::new(client.client_stream.clone());
    loop {
        match client_stream.read_line() {
            Ok(data) => {
                let packet = json::decode(&*data).unwrap();
                try!(client.server_sender.send(packet));
            },

            Err(_) => {
                let packet = NetworkEvent::PlayerQuit(client.id);
                try!(client.server_sender.send(packet));
                return Ok(());
            },
        }
    }
}

fn main() {
    if let Err(e) = run_server("0.0.0.0:8080") {
        println!("Server failed unexpectedly and had to close.\nReason: {:?}", e);
    }
}
