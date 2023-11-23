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
    ChangePasswordResponse, ClientServerMessage, ClientServerMessageEnum, ConnectedUsersResponse,
    Connection, CreatePartyResponse, JoinPartyResponse, LoginResponse, PacmanMessage,
    ServerClientMessage,
};

fn current_time() -> Duration {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
}

#[derive(Clone, PartialEq)]
enum GameStatus {
    Pacman,
    Ghost,
    Idle,
}

// Notice two connections can have the same user, and could even play the same game!
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

    fn get(
        connections: &Mutex<BTreeMap<Connection, ConnectionData>>,
        connection: &Connection,
    ) -> Option<Self> {
        connections.lock().unwrap().get(connection).cloned()
    }

    fn add_new(connections: &Mutex<BTreeMap<Connection, ConnectionData>>, connection: &Connection) {
        connections.lock().unwrap().insert(
            connection.clone(),
            ConnectionData {
                last_heartbeat: current_time(),
                user: None,
                status: GameStatus::Idle,
            },
        );
    }

    fn remove(connections: &Mutex<BTreeMap<Connection, ConnectionData>>, connection: &Connection) {
        ConnectionData::logout(connections, connection);
        connections.lock().unwrap().remove(&connection);
    }

    fn logout(connections: &Mutex<BTreeMap<Connection, ConnectionData>>, connection: &Connection) {
        let mut connections = connections.lock().unwrap();
        let mut finish_game = false;
        if let Some(conn_data) = connections.get_mut(connection) {
            conn_data.user = None;
            if conn_data.status == GameStatus::Pacman {
                // If pacman is quitting the game, finish the game since he's the server in this
                // P2P game
                finish_game = true;
            }
            conn_data.status = GameStatus::Idle;
        }
        if finish_game {
            for conn_data in connections.values_mut() {
                conn_data.status = GameStatus::Idle;
            }
        }
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
            .map(|conn_data| conn_data.last_heartbeat = time);
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
            .map(|conn_data| conn_data.user = user);
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
            .map(|conn_data| conn_data.status = status);
    }

    // Returns a party if it exists, where the first element of the tuple is the Pacman
    fn get_party(
        connections: &Mutex<BTreeMap<Connection, ConnectionData>>,
    ) -> Option<ConnectedUsersResponse> {
        let mut pacman = None;
        let mut ghosts = vec![];
        for (conn, conn_data) in connections.lock().unwrap().iter() {
            match conn_data.status {
                GameStatus::Idle => continue,
                GameStatus::Ghost => ghosts.push((
                    conn.clone(),
                    conn_data.user.clone().unwrap_or("undefined".to_owned()),
                )),
                GameStatus::Pacman => {
                    assert!(pacman.is_none());
                    pacman = Some((
                        conn.clone(),
                        conn_data.user.clone().unwrap_or("undefined".to_owned()),
                    ));
                }
            }
        }
        match pacman {
            None => {
                assert!(ghosts.is_empty());
                None
            }
            Some(pacman) => Some(ConnectedUsersResponse { pacman, ghosts }),
        }
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
            Err(_) => break,
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
            ClientServerMessageEnum::ConnectRequest => {
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
                let response = database.login_request(request.clone());
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
            ClientServerMessageEnum::CreateUserRequest(request) => {
                if !ConnectionData::exists(&connections, &conn) {
                    continue;
                }
                let response = database.create_user(request);
                conn.send(PacmanMessage::ServerClient(
                    ServerClientMessage::CreateUserResponse(response),
                ));
            }
            ClientServerMessageEnum::ChangePasswordRequest(request) => {
                if let Some(conn_data) = ConnectionData::get(&connections, &conn) {
                    let response = if let Some(user) = conn_data.user {
                        database.change_password_request(&user, request)
                    } else {
                        ChangePasswordResponse::NotLoggedIn
                    };
                    conn.send(PacmanMessage::ServerClient(
                        ServerClientMessage::ChangePasswordResponse(response),
                    ));
                }
            }
            ClientServerMessageEnum::CreatePartyRequest => {
                let response = if ConnectionData::get_party(&connections).is_none() {
                    if let Some(conn_data) = ConnectionData::get(&connections, &conn) {
                        if let Some(user) = conn_data.user {
                            log::info!("User {user} created a game as pacman!");
                            ConnectionData::set_status(&connections, &conn, GameStatus::Pacman);
                            CreatePartyResponse::Success
                        } else {
                            CreatePartyResponse::AlreadyExists
                        }
                    } else {
                        CreatePartyResponse::NotLoggedIn
                    }
                } else {
                    CreatePartyResponse::AlreadyExists
                };
                conn.send(PacmanMessage::ServerClient(
                    ServerClientMessage::CreatePartyResponse(response),
                ));
            }
            ClientServerMessageEnum::JoinPartyRequest => {
                let response = if ConnectionData::get_party(&connections).is_none() {
                    JoinPartyResponse::DoesNotExists
                } else {
                    if let Some(conn_data) = ConnectionData::get(&connections, &conn) {
                        if let Some(user) = conn_data.user {
                            log::info!("User {user} joined a game as a ghost!");
                            ConnectionData::set_status(&connections, &conn, GameStatus::Ghost);
                            JoinPartyResponse::Success
                        } else {
                            JoinPartyResponse::NotLoggedIn
                        }
                    } else {
                        JoinPartyResponse::NotLoggedIn
                    }
                };
                conn.send(PacmanMessage::ServerClient(
                    ServerClientMessage::JoinPartyResponse(response),
                ));
            }
            ClientServerMessageEnum::ConnectedUsersRequest => {
                conn.send(PacmanMessage::ServerClient(
                    ServerClientMessage::ConnectedUsersResponse(ConnectionData::get_party(
                        &connections,
                    )),
                ));
            }
        }
    }
}
