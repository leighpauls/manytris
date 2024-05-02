use std::error::Error;
use std::io;
use std::net::{TcpListener, TcpStream};
use tungstenite::stream::MaybeTlsStream;
use tungstenite::WebSocket;
use url::Url;

const LISTEN_HOST: &'static str = "127.0.0.1:9988";
const TARGET_HOST: &'static str = "127.0.0.1:9989";

fn main() {
    let listener = TcpListener::bind(LISTEN_HOST).unwrap();
    listener.set_nonblocking(true).unwrap();

    let mut connections = vec![];

    loop {
        if let Err(e) = try_accept_connections(&listener, &mut connections) {
            eprintln!("Error accepting connection: {}", e);
        }

        let mut i = 0;

        while i < connections.len() {
            let mut remove = false;
            let conn = &mut connections[i];
            if let Err(e) = replay_messages(&mut conn.target_socket, &mut conn.listen_socket) {
                eprintln!("Error replaying target to client: {}", e);
                remove = true;
            }
            if let Err(e) = replay_messages(&mut conn.listen_socket, &mut conn.target_socket) {
                eprintln!("Error replaying client to target: {}", e);
                remove = true;
            }

            if remove {
                connections.remove(i);
            } else {
                i += 1;
            }
        }
    }
}

fn replay_messages(
    source: &mut WebSocket<TcpStream>,
    dest: &mut WebSocket<TcpStream>,
) -> Result<(), tungstenite::Error> {
    loop {
        match source.read() {
            Err(tungstenite::error::Error::Io(ref e)) if e.kind() == io::ErrorKind::WouldBlock => {
                return Ok(());
            }
            result => {
                let message = result?;
                dest.send(message)?;
            }
        }
    }
}

struct ProxyConnection {
    listen_socket: WebSocket<TcpStream>,
    target_socket: WebSocket<TcpStream>,
}

fn try_accept_connections(
    listener: &TcpListener,
    sockets: &mut Vec<ProxyConnection>,
) -> Result<(), Box<dyn Error>> {
    loop {
        match listener.accept() {
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                return Ok(());
            }
            result => {
                let (stream, _) = result?;
                sockets.push(new_proxy_connection(stream)?);
            }
        }
    }
}

fn new_proxy_connection(listen_stream: TcpStream) -> Result<ProxyConnection, Box<dyn Error>> {
    // TODO: replicate the actual request
    listen_stream.set_nonblocking(true)?;
    let listen_socket = tungstenite::accept(listen_stream)?;

    // TODO: use socket2 to make this connect() non-blocking
    let target_stream = TcpStream::connect(TARGET_HOST)?;
    target_stream.set_nonblocking(true)?;

    let target_uri = format!("ws://{}", TARGET_HOST);
    let (target_socket, _response) = tungstenite::client(target_uri, target_stream)?;

    Ok(ProxyConnection {
        listen_socket,
        target_socket,
    })
}
