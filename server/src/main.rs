use std::{
    collections::HashMap,
    io::{self, BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    thread,
};

use interface::PlayerId;
use network_common::{
    error::{NetworkError, NetworkResult},
    NetworkEvent,
};

struct Client {
    id: PlayerId,
    client_stream: TcpStream,
    server_sender: crossbeam_channel::Sender<NetworkEvent>,
}

fn run_server(bind_addr: &str) -> NetworkResult<()> {
    let listener = TcpListener::bind(bind_addr)?;

    let (new_client_sender, new_client_receiver) = crossbeam_channel::unbounded();
    let (packet_sender, packet_receiver) = crossbeam_channel::unbounded();

    thread::spawn(move || {
        let _ = acceptor(listener, new_client_sender, packet_sender);
    });

    let mut clients = HashMap::new();
    loop {
        crossbeam_channel::select! {
            recv(&packet_receiver) -> player_packet => {
                let message = player_packet.map_err(|_| NetworkError::RecvError)?;
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
            recv(new_client_receiver) -> packet => {
                let (id, sender) = packet.map_err(|_| NetworkError::RecvError)?;
                println!("New client connected, id: {}", id);
                clients.insert(id, sender);

                // Tell connected clients that they need to send an update to the new client
                for (_, client_stream) in &mut clients {
                    send_to_client(client_stream, &NetworkEvent::UpdateRequest).unwrap();
                }

            },
        }
    }
}

fn send_to_client(client_stream: &mut TcpStream, message: &NetworkEvent) -> io::Result<usize> {
    let encoded_message = serde_json::to_vec(&message).unwrap();
    client_stream.write(&encoded_message)?;
    client_stream.write("\n".as_bytes())
}

fn acceptor(
    listener: TcpListener,
    new_client_sender: crossbeam_channel::Sender<(u32, TcpStream)>,
    server_sender: crossbeam_channel::Sender<NetworkEvent>,
) -> NetworkResult<()> {
    let mut next_id = 0;

    for stream in listener.incoming() {
        let mut stream = stream?;
        if let Err(e) = send_to_client(&mut stream, &NetworkEvent::PlayerJoin(next_id)) {
            println!("Failed to communicate with client: {}", e);
            continue;
        }

        let client = Client {
            id: next_id,
            client_stream: stream.try_clone()?,
            server_sender: server_sender.clone(),
        };

        thread::spawn(move || {
            let _ = client_handler(client);
        });
        new_client_sender.send((next_id, stream)).map_err(|_| NetworkError::SendError)?;

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
                let packet = serde_json::from_str(&data).unwrap();
                client.server_sender.send(packet).map_err(|_| NetworkError::SendError)?;
            }

            Err(_) => {
                let packet = NetworkEvent::PlayerQuit(client.id);
                client.server_sender.send(packet).map_err(|_| NetworkError::SendError)?;
                return Ok(());
            }
        }
        data.clear();
    }
}

fn main() {
    if let Err(e) = run_server("0.0.0.0:8080") {
        println!("Server failed unexpectedly and had to close.\nReason: {}", e);
    }
}
