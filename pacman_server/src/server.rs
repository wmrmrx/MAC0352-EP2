use std::{
    net::{Ipv4Addr, SocketAddrV4, TcpListener, UdpSocket},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{channel, Sender},
        Arc, Mutex,
    },
    time::{Duration, SystemTime, UNIX_EPOCH}, collections::BTreeMap, io::Read,
};

use pacman_communication::{Connection, ClientServerMessage, PacmanMessage};
use crate::database::Database;

struct ConnectionData {
    last_heartbeat: Duration,
    user: Option<String>
}

type ConnectionListRef = Arc<Mutex<BTreeMap<Connection, ConnectionData>>>;

fn start_listeners(
    port: u16,
    send: Sender<PacmanMessage>,
    keep_running: Arc<AtomicBool>,
) {
    {
        // Udp Listener
        let (send, keep_running) =
            (send.clone(), keep_running.clone());
        std::thread::spawn(move || {
            let mut buf = [0; 9001];
            let listener = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port)).unwrap();
            listener.set_nonblocking(true).unwrap();
            while keep_running.load(Ordering::Relaxed) {
                match listener.recv(&mut buf) {
                    Ok(bytes) => {
                        let buf = &buf[0..bytes];
                        send.send(PacmanMessage::from_bytes(buf)).unwrap();
                    }
                    Err(err) => {
                        if err.kind() != std::io::ErrorKind::WouldBlock {
                            eprintln!("Unknown error: {err:?}");
                        }
                    }
                }
                std::thread::sleep(Duration::from_micros(100));
            }
        });
    }
    {
        std::thread::spawn(move || {
            let mut buf = [0; 9001];
            let listener =
                TcpListener::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port)).unwrap();
            while keep_running.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        stream.set_nonblocking(true).unwrap();
                        match stream.read(&mut buf) {
                            Ok(bytes) => {
                                let buf = &buf[0..bytes];
                                send.send(PacmanMessage::from_bytes(buf)).unwrap();
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
                    Err(err) => {
                        if err.kind() != std::io::ErrorKind::WouldBlock {
                            eprintln!("Unknown error: {err:?}");
                        }
                    }
                }
            }
            std::thread::sleep(Duration::from_micros(100));
        });
    }
}

pub fn run(port: u16, keep_running: Arc<AtomicBool>) {
    let connections: ConnectionListRef = Arc::new(Mutex::new(BTreeMap::new()));

    let database = Database::new();

    // UDP and TCP listeners are abstracted into the same interface, where both of them send messages
    // received through this channel
    let (send, recv) = channel();

    start_listeners(port, send.clone(), keep_running);
}
