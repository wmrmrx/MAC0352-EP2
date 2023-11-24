use std::{
    io::Read,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener, UdpSocket},
    sync::{mpsc::{channel, Sender}, Mutex, Arc},
};

use clap::{Parser, ValueEnum};
use pacman_communication::{Connection, PacmanMessage};

#[derive(Debug, Clone, PartialEq, ValueEnum)]
#[clap(rename_all = "kebab_case")]
enum Protocol {
    Tcp,
    Udp,
}

pub mod state_machine;
pub mod publisher;

#[derive(Parser, Debug)]
#[command(about, long_about = None)]
struct Args {
    #[arg(short, long)]
    server_addr: SocketAddr,
    #[arg(short, long)]
    protocol: Protocol,
}

fn main() {
    let args = Args::parse();
    let (send, recv) = channel::<PacmanMessage>();
    let server;
    let listener_addr;
    let subscribers = Arc::new(Mutex::new(Vec::<Sender<PacmanMessage>>::new()));
    match args.protocol {
        Protocol::Tcp => {
            server = Connection::Tcp(args.server_addr);
            let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)).unwrap();
            listener_addr = Connection::Tcp(listener.local_addr().unwrap());
            let subscribers = subscribers.clone();
            std::thread::spawn(move || {
                let mut buf = [0u8; 9001];
                for stream in listener.incoming() {
                    let Ok(mut stream) = stream else { continue; };
                    let Ok(amt) = stream.read(&mut buf) else { continue; };
                    let subscribers: &mut Vec<Sender<PacmanMessage>> = subscribers.lock().unwrap().as_mut();
                    *subscribers = (*subscribers).into_iter().filter(|send| {
                        send.send(PacmanMessage::from_bytes(&buf[..amt])).is_ok()
                    }).collect();
                }
            });
        }
        Protocol::Udp => {
            server = Connection::Udp(args.server_addr);
            let listener = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)).unwrap();
            listener_addr = Connection::Udp(listener.local_addr().unwrap());
            let subscribers = subscribers.clone();
            std::thread::spawn(move || {
                let mut buf = [0u8; 9001];
                loop {
                    let Ok(amt) = listener.recv(&mut buf) else { continue; };
                    let subscribers: &mut Vec<Sender<PacmanMessage>> = subscribers.lock().unwrap().as_mut();
                    *subscribers = (*subscribers).into_iter().filter(|send| {
                        send.send(PacmanMessage::from_bytes(&buf[..amt])).is_ok()
                    }).collect();
                }
            });
        }
    }
    let publisher = publisher::Publisher::new(subscribers.as_ref());
    state_machine::run(server, connection, publisher);
}
