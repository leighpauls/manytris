use std::net::TcpListener;
use std::thread::spawn;
use tungstenite::Message;

fn main() {
    let server = TcpListener::bind("127.0.0.1:9988").expect("Failed to bind server socket");
    println!("Listening");
    for stream in server.incoming() {
        println!("New connection");
        spawn(move || {
            let mut websocket = tungstenite::accept(stream.expect("Stream failed to open"))
                .expect("Failed to accept websocket");

            loop {
                let msg = websocket.read().expect("Read failed");
                match msg {
                    Message::Text(txt) => {
                        println!("Received: {}", txt);
                    }
                    Message::Binary(bytes) => {
                        println!("Received: {} binary bytes", bytes.len());
                    }
                    Message::Close(_) => {
                        println!("Closed");
                        return;
                    }
                    _ => {}
                }
            }
        });
    }
}
