use std::{
    io::Read,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener, UdpSocket},
    sync::mpsc::channel,
};

use clap::{Parser, ValueEnum};
use pacman_communication::{server_client, Connection, PacmanMessage};

#[derive(Debug, Clone, PartialEq, ValueEnum)]
#[clap(rename_all = "kebab_case")]
enum Protocol {
    Tcp,
    Udp,
}

pub mod client;

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
    let (send, recv) = channel::<server_client::Message>();
    let (server, listener_addr);
    match args.protocol {
        Protocol::Tcp => {
            server = Connection::Tcp(args.server_addr);
            let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)).unwrap();
            listener_addr = Connection::Tcp(listener.local_addr().unwrap());
            std::thread::spawn(move || {
                let mut buf = [0u8; 9001];
                for stream in listener.incoming() {
                    let Ok(mut stream) = stream else { continue; };
                    let Ok(amt) = stream.read(&mut buf) else { continue; };
                    if let Ok(msg) = PacmanMessage::from_bytes(&buf[..amt]) {
                        send.send(msg).unwrap();
                    }
                }
            });
        }
        Protocol::Udp => {
            server = Connection::Udp(args.server_addr);
            let listener = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)).unwrap();
            listener_addr = Connection::Udp(listener.local_addr().unwrap());
            std::thread::spawn(move || {
                let mut buf = [0u8; 9001];
                loop {
                    let Ok(amt) = listener.recv(&mut buf) else { continue; };
                    if let Ok(msg) = PacmanMessage::from_bytes(&buf[..amt]) {
                        send.send(msg).unwrap();
                    }
                }
            });
        }
    }
    client::run(server, listener_addr, recv);
}
