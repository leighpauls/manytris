use ewebsock::{Options, WsEvent, WsMessage};
use manytris::game_state::{DownType, TickMutation};
use manytris::plugins::root::TickEvent;
use std::thread;
use std::time::Duration;

fn main() {
    println!("opening");
    let (mut sender, reciever) =
        ewebsock::connect("ws://127.0.0.1:9988", Options::default()).expect("Failed to connect.");

    let payload =
        rmp_serde::to_vec(&TickEvent(TickMutation::DownInput(DownType::Gravity))).unwrap();

    println!("sending");
    sender.send(WsMessage::Binary(payload));

    loop {
        if let Some(event) = reciever.try_recv() {
            match event {
                WsEvent::Opened => {
                    println!("closing");
                    sender.close().unwrap();

                    return;
                }
                _ => {}
            }
        } else {
            thread::sleep(Duration::from_millis(100));
        }
    }
}
