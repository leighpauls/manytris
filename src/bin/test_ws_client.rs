use ewebsock::{Options, WsEvent, WsMessage};

fn main() {
    println!("opening");
    let (mut sender, reciever) =
        ewebsock::connect(
            "ws://127.0.0.1:9988",
            Options::default()).expect("Failed to connect.");

    println!("sending");
    sender.send(WsMessage::Text("Hello".into()));
    println!("sent");

    loop {
        if let Some(event) = reciever.try_recv() {
            match event {
                WsEvent::Opened => {
                    println!("opened")
                }
                WsEvent::Message(msg) => match msg {
                    WsMessage::Binary(_) => {
                        println!("Received binary")
                    }
                    WsMessage::Text(txt) => {
                        println!("Received: {}", txt)
                    }
                    WsMessage::Unknown(_) => {
                        println!("Received unknown")
                    }
                    _ => {}
                },
                WsEvent::Error(err) => {
                    println!("Error: {}", err);
                    return;
                }
                WsEvent::Closed => {
                    println!("Closed");
                    return;
                }
            }
        }
    }
}
