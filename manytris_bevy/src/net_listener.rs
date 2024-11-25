use crate::cli_options::HostConfig;
use crate::game_container::GameContainer;
use crate::net_game_control_manager::{
    ConnectionId, ConnectionTarget, ReceiveControlEventFromClient, SendControlEventToClient,
};
use crate::net_listener::ListenResult::{DropSocket, NewMessage};
use crate::net_protocol::NetMessage;
use crate::root::TickEvent;
use crate::states;
use crate::states::PlayingState;
use crate::system_sets::UpdateSystems;
use bevy::prelude::*;
use std::collections::BTreeMap;
use std::error::Error;
use std::io;
use std::net::{TcpListener, TcpStream};
use tungstenite::{Message, WebSocket};

#[derive(Component)]
pub struct ServerListenerComponent {
    listener: TcpListener,
    sockets: BTreeMap<ConnectionId, WebSocket<TcpStream>>,
}

#[derive(Resource)]
pub struct NetListenerConfig(pub HostConfig);

pub fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(PlayingState::Playing),
        init_listener.run_if(states::is_server),
    )
    .add_systems(
        OnExit(PlayingState::Playing),
        teardown_listener.run_if(states::is_server),
    )
    .add_systems(
        Update,
        (
            listener_system.in_set(UpdateSystems::LocalEventProducers),
            sender_system.in_set(UpdateSystems::EventSenders),
        )
            .run_if(in_state(PlayingState::Playing))
            .run_if(states::is_server),
    )
    .add_event::<SendControlEventToClient>()
    .add_event::<ReceiveControlEventFromClient>();
}

fn init_listener(mut commands: Commands, config: Res<NetListenerConfig>) {
    let NetListenerConfig(HostConfig { host, port }) = config.as_ref();
    let listener = TcpListener::bind(format!("{host}:{port}")).unwrap();
    listener.set_nonblocking(true).unwrap();

    commands.spawn(ServerListenerComponent {
        listener,
        sockets: BTreeMap::new(),
    });
}

fn teardown_listener(
    mut commands: Commands,
    listener_q: Query<Entity, With<ServerListenerComponent>>,
) {
    commands.entity(listener_q.single()).despawn();
}

fn listener_system(
    mut listener_q: Query<&mut ServerListenerComponent>,
    mut tick_writer: EventWriter<TickEvent>,
    mut control_writer: EventWriter<ReceiveControlEventFromClient>,
) {
    let listener = listener_q.single_mut().into_inner();

    if let Err(e) = accept_new_connections(&listener.listener, &mut listener.sockets) {
        eprintln!("Error while accepting new sockets: {e}");
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
    mut tick_event_reader: EventReader<TickEvent>,
    mut control_event_reader: EventReader<SendControlEventToClient>,
    q_game_container: Query<&GameContainer>,
) {
    let mut listener = listener_q.single_mut();
    let game_container = q_game_container.single();

    let mut payloads: Vec<(ConnectionTarget, Vec<u8>)> = vec![];
    payloads.extend(control_event_reader.read().map(|sce| {
        (
            sce.to_connection,
            rmp_serde::to_vec(&NetMessage::ServerControl(sce.event.clone())).unwrap(),
        )
    }));

    // Events made locally by the server go to all clients.
    // Events made by a client go to all except the original client.
    payloads.extend(tick_event_reader.read().filter_map(|te| {
        let Some(from_connection) = game_container.connection_for_game(&te.mutation.game_id) else {
            return None;
        };
        Some((
            ConnectionTarget::AllExcept(if te.local {
                None
            } else {
                Some(from_connection)
            }),
            rmp_serde::to_vec(&NetMessage::Tick(te.mutation.clone())).unwrap(),
        ))
    }));

    for (target, bytes) in payloads {
        let sockets = match target {
            ConnectionTarget::To(conn) => {
                vec![listener.sockets.get_mut(&conn).unwrap()]
            }
            ConnectionTarget::AllExcept(None) => listener.sockets.values_mut().collect(),
            ConnectionTarget::AllExcept(Some(except)) => listener
                .sockets
                .iter_mut()
                .filter(|(conn, _)| **conn != except)
                .map(|(_, soc)| soc)
                .collect(),
        };

        let m = Message::Binary(bytes);
        for s in sockets {
            s.send(m.clone()).unwrap();
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
                println!("New incomming connection.");
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
                eprintln!("Error reading from websocket, dropping thread: {e}");
                return DropSocket;
            }
            Ok(Message::Binary(buf)) => match rmp_serde::from_slice(&buf) {
                Ok(nm) => result.push(nm),
                Err(e) => eprintln!("Unable to read message: {e}"),
            },
            Ok(Message::Close(cf)) => {
                println!("Connection closed, reason: {cf:?}");
                return DropSocket;
            }
            Ok(m) => {
                eprintln!("Unexpected message: {m:?}");
            }
        }
    }
    NewMessage(result)
}
