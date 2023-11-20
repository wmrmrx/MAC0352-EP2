use std::{
    collections::BTreeMap,
    io::Read,
    net::{Ipv4Addr, SocketAddrV4, TcpListener, UdpSocket},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{channel, Sender},
        Arc, Mutex,
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::database::Database;
use pacman_communication::{
    ClientServerMessage, ClientServerMessageEnum, Connection, LoginRequest, LoginResponse,
    PacmanMessage, ServerClientMessage,
};

fn current_time() -> Duration {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
}

#[derive(Clone)]
enum GameStatus {
    Pacman,
    Ghost,
    Idle,
}

#[derive(Clone)]
struct ConnectionData {
    last_heartbeat: Duration,
    user: Option<String>,
    status: GameStatus,
}

impl ConnectionData {
    fn exists(
        connections: &Mutex<BTreeMap<Connection, ConnectionData>>,
        connection: &Connection,
    ) -> bool {
        connections.lock().unwrap().get(connection).is_some()
    }

    fn add_new(connections: &Mutex<BTreeMap<Connection, ConnectionData>>, connection: &Connection) {
        connections.lock().unwrap().insert(
            *connection,
            ConnectionData {
                last_heartbeat: current_time(),
                user: None,
                status: GameStatus::Idle,
            },
        );
    }

    fn remove(connections: &Mutex<BTreeMap<Connection, ConnectionData>>, connection: &Connection) {
        connections.lock().unwrap().remove(&connection);
    }

    fn set_heartbeat(
        connections: &Mutex<BTreeMap<Connection, ConnectionData>>,
        connection: &Connection,
        time: Duration,
    ) {
        connections
            .lock()
            .unwrap()
            .get_mut(connection)
            .unwrap()
            .last_heartbeat = time;
    }

    fn set_user(
        connections: &Mutex<BTreeMap<Connection, ConnectionData>>,
        connection: &Connection,
        user: Option<String>,
    ) {
        connections
            .lock()
            .unwrap()
            .get_mut(connection)
            .unwrap()
            .user = user;
    }

    fn set_status(
        connections: &Mutex<BTreeMap<Connection, ConnectionData>>,
        connection: &Connection,
        status: GameStatus,
    ) {
        connections
            .lock()
            .unwrap()
            .get_mut(connection)
            .unwrap()
            .status = status;
    }
}

type ConnectionListRef = Arc<Mutex<BTreeMap<Connection, ConnectionData>>>;

fn start_listeners(port: u16, send: Sender<PacmanMessage>, keep_running: Arc<AtomicBool>) {
    {
        // Udp Listener
        let (send, keep_running) = (send.clone(), keep_running.clone());
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
    let database = Database::new();

    let connections: ConnectionListRef = Arc::new(Mutex::new(BTreeMap::new()));

    // UDP and TCP listeners are abstracted into the same interface, where both of them send messages
    // received through this channel
    let (send, recv) = channel();

    start_listeners(port, send.clone(), keep_running);
    loop {
        let msg = match recv.recv() {
            Ok(msg) => msg,
            Err(err) => break,
        };
        let (conn, msg) = if let PacmanMessage::ClientServer(ClientServerMessage {
            connection: conn,
            message: msg,
        }) = msg
        {
            (conn, msg)
        } else {
            eprintln!("Server received message not meant for it!");
            continue;
        };

        match msg {
            ClientServerMessageEnum::ConnectRequest(request) => {
                if ConnectionData::exists(&connections, &conn) {
                    continue;
                }
                ConnectionData::add_new(&connections, &conn);
            }
            ClientServerMessageEnum::Heartbeat => {
                if !ConnectionData::exists(&connections, &conn) {
                    continue;
                }
                ConnectionData::set_heartbeat(&connections, &conn, current_time());
            }
            ClientServerMessageEnum::LoginRequest(request) => {
                if !ConnectionData::exists(&connections, &conn) {
                    continue;
                }
                let response = database.login_request(request);
                if let LoginResponse::LoggedIn = response {
                    ConnectionData::set_user(&connections, &conn, Some(request.user))
                }
                conn.send(PacmanMessage::ServerClient(
                    ServerClientMessage::LoginResponse(response),
                ));
            }
            ClientServerMessageEnum::LogoutRequest => {
                if !ConnectionData::exists(&connections, &conn) {
                    continue;
                }
                ConnectionData::set_user(&connections, &conn, None);
                conn.send(PacmanMessage::ServerClient(
                    ServerClientMessage::LogoutResponse,
                ));
            }
        }
    }
}
