use std::collections::BTreeMap;
use std::error::Error;
use std::io;
use std::net::{TcpListener, TcpStream};

use bevy::prelude::*;
use bevy::tasks::futures_lite::AsyncSeek;
use tungstenite::{Message, WebSocket};

use crate::cli_options::HostConfig;
use crate::plugins::net_game_control_manager::{
    ConnectionId, ReceiveControlEventFromClient, SendControlEventToClient,
};
use crate::plugins::net_listener::ListenResult::{DropSocket, NewMessage};
use crate::plugins::net_protocol::NetMessage;
use crate::plugins::root::TickEvent;
use crate::plugins::system_sets::UpdateSystems;

#[derive(Component)]
pub struct ServerListenerComponent {
    listener: TcpListener,
    sockets: BTreeMap<ConnectionId, WebSocket<TcpStream>>,
}

#[derive(Resource)]
pub struct NetListenerConfig(pub HostConfig);

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, init_listener).add_systems(
        Update,
        (
            listener_system.in_set(UpdateSystems::LocalEventProducers),
            sender_system.in_set(UpdateSystems::EventSenders),
        ),
    );
}

fn init_listener(mut commands: Commands, config: Res<NetListenerConfig>) {
    let listener = TcpListener::bind(format!("{}:{}", config.0.host, config.0.port)).unwrap();
    listener.set_nonblocking(true).unwrap();

    commands.spawn(ServerListenerComponent {
        listener,
        sockets: BTreeMap::new(),
    });
}

fn listener_system(
    mut listener_q: Query<&mut ServerListenerComponent>,
    mut tick_writer: EventWriter<TickEvent>,
    mut control_writer: EventWriter<ReceiveControlEventFromClient>,
) {
    let listener = listener_q.single_mut().into_inner();

    if let Err(e) = accept_new_connections(&listener.listener, &mut listener.sockets) {
        eprintln!("Error while accepting new sockets: {}", e);
    }

    let mut remove_connections = vec![];
    for (connection_id, socket) in &mut listener.sockets {
        match listen_to_socket(socket) {
            DropSocket => {
                remove_connections.push(connection_id.clone());
            }
            NewMessage(msgs) => {
                for m in msgs {
                    match m {
                        NetMessage::Tick(tm) => {
                            tick_writer.send(TickEvent::new_remote(tm));
                        }
                        NetMessage::ClientControl(event) => {
                            control_writer.send(ReceiveControlEventFromClient {
                                event,
                                from_connection: connection_id.clone(),
                            });
                        }
                        NetMessage::ServerControl(_) => {
                            eprintln!("Unexpected server control message");
                        }
                    }
                }
            }
        }
    }
    remove_connections.iter().for_each(|cid| {
        listener.sockets.remove(cid);
    });
}

fn sender_system(
    mut listener_q: Query<&mut ServerListenerComponent>,
    mut event_reader: EventReader<TickEvent>,
    mut control_reader: EventReader<SendControlEventToClient>,
) {
    let mut listener = listener_q.single_mut();

    let tick_event_payloads: Vec<Vec<u8>> = event_reader
        .read()
        .filter(|e| e.local)
        .map(|e| rmp_serde::to_vec(&NetMessage::Tick(e.mutation.clone())).unwrap())
        .collect();

    let mut control_payloads_by_connection_id: BTreeMap<ConnectionId, Vec<Vec<u8>>> =
        BTreeMap::new();
    for ce in control_reader.read() {
        let payload_list = control_payloads_by_connection_id
            .entry(ce.to_connection)
            .or_default();
        payload_list.push(rmp_serde::to_vec(&NetMessage::ServerControl(ce.event.clone())).unwrap())
    }

    for (connection_id, socket) in &mut listener.sockets {
        for p in control_payloads_by_connection_id
            .get(connection_id)
            .unwrap_or(&vec![])
        {
            socket.send(Message::Binary(p.clone())).unwrap();
        }

        for p in &tick_event_payloads {
            socket.send(Message::Binary(p.clone())).unwrap();
        }
    }
}

fn accept_new_connections(
    listener: &TcpListener,
    sockets: &mut BTreeMap<ConnectionId, WebSocket<TcpStream>>,
) -> Result<(), Box<dyn Error>> {
    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                s.set_nonblocking(true)?;
                sockets.insert(ConnectionId::new(), tungstenite::accept(s)?);
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                break;
            }
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}

enum ListenResult {
    DropSocket,
    NewMessage(Vec<NetMessage>),
}

fn listen_to_socket(web_socket: &mut WebSocket<TcpStream>) -> ListenResult {
    let mut result = vec![];
    loop {
        match web_socket.read() {
            Err(tungstenite::error::Error::Io(ref e)) if e.kind() == io::ErrorKind::WouldBlock => {
                break;
            }
            Err(e) => {
                eprintln!("Error reading from websocket, dropping thread: {}", e);
                return DropSocket;
            }
            Ok(Message::Binary(buf)) => match rmp_serde::from_slice(&buf) {
                Ok(nm) => result.push(nm),
                Err(e) => eprintln!("Unable to read message: {}", e),
            },
            Ok(Message::Close(cf)) => {
                println!("Connection closed, reason: {:?}", cf);
                return DropSocket;
            }
            Ok(m) => {
                eprintln!("Unexpected message: {:?}", m);
            }
        }
    }
    NewMessage(result)
}
