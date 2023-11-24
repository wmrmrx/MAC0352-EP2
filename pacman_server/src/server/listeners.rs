use std::{
    io::Read,
    net::{Ipv4Addr, SocketAddrV4, TcpListener, UdpSocket},
    sync::mpsc::Sender,
    time::Duration,
};

use pacman_communication::{client_server, PacmanMessage};

pub fn start(port: u16, send: Sender<client_server::Message>) {
    const TICK: Duration = Duration::from_millis(1);
    {
        // Udp Listener
        let send = send.clone();
        std::thread::spawn(move || {
            let mut buf = [0; 9001];
            let listener = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port)).unwrap();
            loop {
                match listener.recv(&mut buf) {
                    Ok(amt) => {
                        let Ok(msg) = PacmanMessage::from_bytes(&buf[..amt]) else { continue; };
                        send.send(msg).unwrap();
                    }
                    Err(err) => {
                        if err.kind() != std::io::ErrorKind::WouldBlock {
                            eprintln!("Unknown error: {err:?}");
                        }
                    }
                }
            }
        });
    }
    {
        std::thread::spawn(move || {
            let mut buf = [0; 9001];
            let listener =
                TcpListener::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port)).unwrap();
            for stream in listener.incoming() {
                match stream {
                    Ok(mut stream) => {
                        let send = send.clone();
                        std::thread::spawn(move || {
                            let mut buf = [0; 9001];
                            loop {
                                let Ok(amt) = stream.read(&mut buf) else { break; };
                                if let Ok(msg) = PacmanMessage::from_bytes(&buf[..amt]) {
                                    send.send(msg).unwrap();
                                }
                            }
                        });
                    }
                    Err(err) => {
                        eprintln!("Unknown error: {err}");
                    }
                }
            }
        });
    }
}
