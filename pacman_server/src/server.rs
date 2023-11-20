use std::{
    net::{Ipv4Addr, SocketAddrV4, TcpListener, UdpSocket},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{channel, Sender},
        Arc, Mutex,
    },
    time::Duration,
};

use pacman_communication::{Connection, ClientServerMessage, PacmanMessage, PacmanBinary};

type ConnectionListRef = Arc<Mutex<Vec<Connection>>>;

fn start_listeners(
    port: u16,
    connections: ConnectionListRef,
    send: Sender<ClientServerMessage>,
    keep_running: Arc<AtomicBool>,
) {
    {
        // Udp Listener
        let (connections, send, keep_running) =
            (connections.clone(), send.clone(), keep_running.clone());
        std::thread::spawn(move || {
            let mut buf = [0; 9001];
            let listener = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port)).unwrap();
            listener.set_nonblocking(true).unwrap();
            while keep_running.load(Ordering::Relaxed) {
                loop {
                    match listener.recv(&mut buf) {
                        Ok(bytes) => {
                            let buf = &buf[0..bytes];
                            if let PacmanMessage::ClientServer(ClientServerMessage{ id: conn, message: msg }) = PacmanMessage::from_bytes(buf) {
                            }
                        }
                        Err(err) => {
                            if err.kind() == std::io::ErrorKind::WouldBlock {
                                break;
                            } else {
                                eprintln!("Unknown error: {err:?}");
                            }
                        }
                    }
                }
                std::thread::sleep(Duration::from_micros(100));
            }
        });
    }
    {
        let (connections, send, keep_running) =
            (connections.clone(), send.clone(), keep_running.clone());
        std::thread::spawn(move || {
            let mut buf = [0; 9001];
            let listener =
                TcpListener::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port)).unwrap();
            std::thread::sleep(Duration::from_millis(1));
        });
    }
}

pub fn run(port: u16, keep_running: Arc<AtomicBool>) {
    let connections: ConnectionListRef = Arc::new(Mutex::new(Vec::new()));

    let (send, recv) = channel();

    start_listeners(port, connections.clone(), send.clone(), keep_running);
}
