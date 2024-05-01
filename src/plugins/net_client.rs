use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use ewebsock::{Options, WsEvent, WsMessage, WsReceiver, WsSender};

use crate::plugins::net_listener;
use crate::plugins::root::TickEvent;
use crate::plugins::system_sets::{StartupSystems, UpdateSystems};

#[derive(Component)]
pub enum ClientNetComponent {
    NotConnected,
    Connecting(Arc<Mutex<(WsSender, WsReceiver)>>),
    Connected(Arc<Mutex<(WsSender, WsReceiver)>>),
}

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, init.in_set(StartupSystems::AfterRoot))
        .add_systems(
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
) {
    let net = net_q.single_mut().into_inner();

    let mut new_net = None;
    match net {
        ClientNetComponent::NotConnected => {
            virtual_time.pause();

            let addr = format!("ws://{}", net_listener::HOST);

            if let Ok((sender, receiver)) = ewebsock::connect(addr, Options::default()) {
                println!("Opened connection...");
                new_net = Some(ClientNetComponent::Connecting(Arc::new(Mutex::new((
                    sender, receiver,
                )))));
            }
        }
        ClientNetComponent::Connecting(sr_pair) => {
            virtual_time.pause();

            match sr_pair.lock().unwrap().1.try_recv() {
                Some(WsEvent::Opened) => {
                    new_net = Some(ClientNetComponent::Connected(sr_pair.clone()));
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
                    let decoded = rmp_serde::from_slice::<TickEvent>(&payload);
                    if let Ok(event) = decoded {
                        tick_events.send(event.as_remote());
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
) {
    let net = net_q.single_mut();

    if let ClientNetComponent::Connected(sr_pair) = net.into_inner() {
        for event in tick_events.read() {
            if event.local {
                let payload = rmp_serde::to_vec(event).unwrap();
                sr_pair.lock().unwrap().0.send(WsMessage::Binary(payload));
            }
        }
    }
}
