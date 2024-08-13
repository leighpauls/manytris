use std::error::Error;
use std::io;
use std::net::{TcpListener, TcpStream};

use crate::cli_options::HostConfig;
use bevy::prelude::*;
use tungstenite::{Message, WebSocket};

use crate::plugins::net_listener::ListenResult::{DropSocket, NewMessage};
use crate::plugins::net_protocol::NetMessage;
use crate::plugins::root::{ReceiveControlEvent, SendControlEvent, TickEvent};
use crate::plugins::system_sets::UpdateSystems;

#[derive(Component)]
pub struct ServerListenerComponent {
    listener: TcpListener,
    sockets: Vec<WebSocket<TcpStream>>,
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
        sockets: vec![],
    });
}

fn listener_system(
    mut listener_q: Query<&mut ServerListenerComponent>,
    mut tick_writer: EventWriter<TickEvent>,
    mut control_writer: EventWriter<ReceiveControlEvent>,
) {
    let listener = listener_q.single_mut().into_inner();

    if let Err(e) = accept_new_connections(&listener.listener, &mut listener.sockets) {
        eprintln!("Error while accepting new sockets: {}", e);
    }

    let mut i = 0;
    while i < listener.sockets.len() {
        match listen_to_socket(&mut listener.sockets[i]) {
            DropSocket => {
                listener.sockets.remove(i);
            }
            NewMessage(msgs) => {
                for m in msgs {
                    match m {
                        NetMessage::Tick(tm) => {
                            tick_writer.send(TickEvent::new_remote(tm));
                        }
                        NetMessage::Control(ce) => {
                            control_writer.send(ReceiveControlEvent(ce));
                        }
                    }
                }
                i += 1;
            }
        }
    }
}

fn sender_system(
    mut listener_q: Query<&mut ServerListenerComponent>,
    mut event_reader: EventReader<TickEvent>,
    mut control_reader: EventReader<SendControlEvent>,
) {
    let mut listener = listener_q.single_mut();

    let payloads: Vec<Vec<u8>> = control_reader
        .read()
        .map(|SendControlEvent(ce)| NetMessage::Control(ce.clone()))
        .chain(
            event_reader
                .read()
                .filter(|e| e.local)
                .map(|e| NetMessage::Tick(e.mutation.clone())),
        )
        .map(|nm| rmp_serde::to_vec(&nm).unwrap())
        .collect();

    for socket in &mut listener.sockets {
        for p in &payloads {
            // TODO: drop socket on error?
            socket.send(Message::Binary(p.clone())).unwrap();
        }
    }
}

fn accept_new_connections(
    listener: &TcpListener,
    sockets: &mut Vec<WebSocket<TcpStream>>,
) -> Result<(), Box<dyn Error>> {
    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                s.set_nonblocking(true)?;
                sockets.push(tungstenite::accept(s)?);
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
