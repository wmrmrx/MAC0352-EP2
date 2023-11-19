use std::{
    net::{SocketAddr, TcpStream, SocketAddrV4, UdpSocket, Ipv4Addr, TcpListener},
    sync::{Arc, Mutex, mpsc::{channel, Sender}, atomic::AtomicBool},
    time::Duration, io::Write,
};

use pacman_communication::Connection;

const TICK: Duration = Duration::from_secs(1);

type ConnectionListRef = Arc<Mutex<Vec<Connection>>>;

fn start_listeners(port: u16, connections: ConnectionListRef, send: Sender<String>, keep_running: Arc<AtomicBool>) {
    {
        // Udp Listener
        let (connections, send, keep_running) = (connections.clone(), send.clone(), keep_running.clone());
        std::thread::spawn(move || {
            log::debug!("Spawned UdpSocket listener thread");
            let listener = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port)).unwrap();
        });
    }
    {
        let (connections, send, keep_running) = (connections.clone(), send.clone(), keep_running.clone());
        std::thread::spawn(move || {
            log::debug!("Spawned TcpListener thread");
            let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port)).unwrap();
        });
    }
}

pub fn run(port: u16) {
    let connections: ConnectionListRef = Arc::new(Mutex::new(Vec::new()));

    let (send, recv) = channel();

    start_listeners(port, connections.clone(), send.clone(), keep_running);
}
