use std::ops::Deref;
use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use ewebsock::{Options, WsEvent, WsMessage, WsReceiver, WsSender};

use crate::cli_options::HostConfig;
use crate::plugins::game_container::LocalGameRoot;
use crate::plugins::net_game_control_manager::{ClientControlEvent, ServerControlEvent};
use crate::plugins::net_protocol::NetMessage;
use crate::plugins::root::TickEvent;
use crate::plugins::states;
use crate::plugins::states::PlayingState;
use crate::plugins::system_sets::UpdateSystems;

pub enum ClientNetComponent {
    NotConnected,
    Connecting(Arc<Mutex<(WsSender, WsReceiver)>>),
    Connected(Arc<Mutex<(WsSender, WsReceiver)>>),
}

#[derive(Resource)]
pub struct NetClientConfig(pub HostConfig);

pub fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(PlayingState::Playing),
        init.run_if(states::is_multiplayer_client),
    )
    .add_systems(
        OnExit(PlayingState::Playing),
        teardown.run_if(states::is_multiplayer_client),
    )
    .add_systems(
        Update,
        (
            update_client_connect.in_set(UpdateSystems::LocalEventProducers),
            update_client_net_receive.in_set(UpdateSystems::LocalEventProducers),
            update_client_net_send.in_set(UpdateSystems::EventSenders),
        )
            .run_if(in_state(PlayingState::Playing))
            .run_if(states::is_multiplayer_client),
    )
    .add_event::<ClientControlEvent>()
    .add_event::<ServerControlEvent>();
}

fn init(world: &mut World) {
    world.insert_non_send_resource(ClientNetComponent::NotConnected);
}

fn teardown(world: &mut World) {
    world.remove_non_send_resource::<ClientNetComponent>();
}

fn update_client_connect(
    mut net: NonSendMut<ClientNetComponent>,
    mut virtual_time: ResMut<Time<Virtual>>,
    mut control_events: EventWriter<ClientControlEvent>,
    config: Res<NetClientConfig>,
    local_game_root: Option<Res<LocalGameRoot>>,
) {
    let mut new_net = None;
    match &net.as_ref() {
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
                    let request = match local_game_root {
                        None => ClientControlEvent::JoinRequest,
                        Some(game_root) => ClientControlEvent::ReconnectRequest(game_root.game_id),
                    };
                    control_events.send(request);
                }
                Some(WsEvent::Error(err_msg)) => {
                    eprintln!("Unexpected connecting message: {err_msg}");
                    new_net = Some(ClientNetComponent::NotConnected);
                }
                Some(WsEvent::Closed) => {
                    eprintln!("Connection closed while trying to connect.");
                    new_net = Some(ClientNetComponent::NotConnected);
                }
                Some(WsEvent::Message(msg)) => {
                    eprintln!("Unexpected message before open: {msg:?}");
                }
                None => {
                }
            }
        }
        ClientNetComponent::Connected(_) => {
            virtual_time.unpause();
        }
    }

    if let Some(n) = new_net {
        *(net.as_mut()) = n;
    }
}

fn update_client_net_receive(
    mut net: NonSendMut<ClientNetComponent>,
    mut tick_events: EventWriter<TickEvent>,
    mut control_events: EventWriter<ServerControlEvent>,
) {
    let mut disconnected = false;

    if let ClientNetComponent::Connected(sr_pair) = net.as_ref() {
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
        *(net.as_mut()) = ClientNetComponent::NotConnected;
    }
}

fn update_client_net_send(
    net: NonSend<ClientNetComponent>,
    mut tick_events: EventReader<TickEvent>,
    mut control_events: EventReader<ClientControlEvent>,
) {
    if let ClientNetComponent::Connected(sr_pair) = &net.deref() {
        let send_func = |nm: NetMessage| {
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
