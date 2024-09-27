use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use ewebsock::{Options, WsEvent, WsMessage, WsReceiver, WsSender};

use crate::cli_options::HostConfig;
use crate::plugins::net_game_control_manager::{ClientControlEvent, ServerControlEvent};
use crate::plugins::net_protocol::NetMessage;
use crate::plugins::root::TickEvent;
use crate::plugins::system_sets::UpdateSystems;

#[derive(Component)]
pub enum ClientNetComponent {
    NotConnected,
    Connecting(Arc<Mutex<(WsSender, WsReceiver)>>),
    Connected(Arc<Mutex<(WsSender, WsReceiver)>>),
}

#[derive(Resource)]
pub struct NetClientConfig(pub HostConfig);

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, init).add_systems(
        Update,
        (
            update_client_connect.in_set(UpdateSystems::LocalEventProducers),
            update_client_net_receive.in_set(UpdateSystems::LocalEventProducers),
            update_client_net_send.in_set(UpdateSystems::EventSenders),
        ),
    );
}

fn init(mut commands: Commands) {
    commands.spawn(ClientNetComponent::NotConnected);
}

fn update_client_connect(
    mut net_q: Query<&mut ClientNetComponent>,
    mut virtual_time: ResMut<Time<Virtual>>,
    mut control_events: EventWriter<ClientControlEvent>,
    config: Res<NetClientConfig>,
) {
    let net = net_q.single_mut().into_inner();

    let mut new_net = None;
    match net {
        ClientNetComponent::NotConnected => {
            virtual_time.pause();

            let addr = format!("ws://{}:{}", config.0.host, config.0.port);

            if let Ok((sender, receiver)) = ewebsock::connect(addr, Options::default()) {
                println!("Opening connection...");
                new_net = Some(ClientNetComponent::Connecting(Arc::new(Mutex::new((
                    sender, receiver,
                )))));
            }
        }
        ClientNetComponent::Connecting(sr_pair) => {
            virtual_time.pause();

            match sr_pair.lock().unwrap().1.try_recv() {
                Some(WsEvent::Opened) => {
                    println!("Connected!");
                    new_net = Some(ClientNetComponent::Connected(sr_pair.clone()));
                    control_events.send(ClientControlEvent::JoinRequest);
                }
                Some(e) => {
                    eprintln!("Unexpected connecting message: {:?}", e);
                    new_net = Some(ClientNetComponent::NotConnected);
                }
                None => {}
            }
        }
        ClientNetComponent::Connected(_) => {
            virtual_time.unpause();
        }
    }

    if let Some(n) = new_net {
        *net = n;
    }
}

fn update_client_net_receive(
    mut net_q: Query<&mut ClientNetComponent>,
    mut tick_events: EventWriter<TickEvent>,
    mut control_events: EventWriter<ServerControlEvent>,
) {
    let net = net_q.single_mut().into_inner();

    let mut disconnected = false;

    if let ClientNetComponent::Connected(sr_pair) = net {
        while let Some(event) = sr_pair.lock().unwrap().1.try_recv() {
            match event {
                WsEvent::Opened => {
                    println!("Connection Opened");
                }
                WsEvent::Message(WsMessage::Binary(payload)) => {
                    let decoded = rmp_serde::from_slice::<NetMessage>(&payload);
                    match decoded.unwrap() {
                        NetMessage::Tick(tm) => {
                            tick_events.send(TickEvent::new_remote(tm));
                        }
                        NetMessage::ServerControl(sce) => {
                            control_events.send(sce);
                        }
                        NetMessage::ClientControl(_) => {
                            eprintln!("Unexpected client control message");
                        }
                    }
                }
                WsEvent::Message(msg) => {
                    println!("Unexpected message: {:?}", msg);
                }
                WsEvent::Error(err) => {
                    eprintln!("Network error {}", err);
                    disconnected = true;
                }
                WsEvent::Closed => {
                    println!("Connection was closed");
                    disconnected = true;
                }
            }
        }
    }

    if disconnected {
        *net = ClientNetComponent::NotConnected;
    }
}

fn update_client_net_send(
    mut net_q: Query<&mut ClientNetComponent>,
    mut tick_events: EventReader<TickEvent>,
    mut control_events: EventReader<ClientControlEvent>,
) {
    let net = net_q.single_mut();

    if let ClientNetComponent::Connected(sr_pair) = net.into_inner() {
        let send_func = |nm: NetMessage| {
            println!("Sending {:?}", nm);
            let payload = rmp_serde::to_vec(&nm).unwrap();
            sr_pair.lock().unwrap().0.send(WsMessage::Binary(payload));
        };

        control_events
            .read()
            .map(|ce| NetMessage::ClientControl(ce.clone()))
            .for_each(send_func);

        tick_events
            .read()
            .filter(|te| te.local)
            .map(|e| NetMessage::Tick(e.mutation.clone()))
            .for_each(send_func);
    }
}
