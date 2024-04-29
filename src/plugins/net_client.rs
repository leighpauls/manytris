use std::sync::Mutex;

use bevy::prelude::*;
use ewebsock::{Options, WsEvent, WsMessage, WsReceiver, WsSender};

use crate::plugins::net_listener;
use crate::plugins::root::TickEvent;
use crate::plugins::system_sets::{StartupSystems, UpdateSystems};

#[derive(Component)]
pub struct ClientNetComponent {
    sender: WsSender,
    receiver: Mutex<WsReceiver>,
}

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, init.in_set(StartupSystems::AfterRoot))
        .add_systems(
            Update,
            (
                update_client_net_receive.in_set(UpdateSystems::LocalEventProducers),
                update_client_net_send.in_set(UpdateSystems::EventSenders),
            ),
        );
}

fn init(mut commands: Commands) {
    let addr = format!("ws://{}", net_listener::HOST);

    let (sender, receiver) =
        ewebsock::connect(addr, Options::default()).expect("Failed to create websocket");

    commands.spawn(ClientNetComponent {
        sender,
        receiver: Mutex::new(receiver),
    });
}

fn update_client_net_receive(
    mut net_q: Query<&mut ClientNetComponent>,
    mut tick_events: EventWriter<TickEvent>,
) {
    let mut net = net_q.single_mut();
    while let Some(event) = net.receiver.lock().unwrap().try_recv() {
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
            }
            WsEvent::Closed => {
                println!("Connection was closed");
            }
        }
    }
}

fn update_client_net_send(
    mut net_q: Query<&mut ClientNetComponent>,
    mut tick_events: EventReader<TickEvent>,
) {
    let mut net = net_q.single_mut();

    for event in tick_events.read() {
        if event.local {
            let payload = rmp_serde::to_vec(event).unwrap();
            net.sender.send(WsMessage::Binary(payload));
        }
    }
}
