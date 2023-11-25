use std::{
    io::Read,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener, UdpSocket},
    sync::{mpsc::channel, atomic::{AtomicBool, Ordering}, Arc}, time::Duration,
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
    loop {
        let keep_running = Arc::new(AtomicBool::new(true));
        println!("Starting a new client!");
        let (send, recv) = channel::<server_client::Message>();
        let (server, listener_addr);
        match args.protocol {
            Protocol::Tcp => {
                let keep_running = keep_running.clone();
                server = Connection::Tcp(args.server_addr);
                let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)).unwrap();
                listener.set_nonblocking(true).unwrap();
                listener_addr = Connection::Tcp(listener.local_addr().unwrap());
                std::thread::spawn(move || {
                    let mut buf = [0u8; 9001];
                    while keep_running.load(Ordering::Relaxed) {
                        std::thread::sleep(Duration::from_millis(33));
                        let Ok((mut stream, _)) = listener.accept() else { continue; };
                        let Ok(amt) = stream.read(&mut buf) else { continue; };
                        let Ok(msg) = PacmanMessage::from_bytes(&buf[..amt]) else { continue; };
                        send.send(msg).unwrap();
                    }
                });
            }
            Protocol::Udp => {
                let keep_running = keep_running.clone();
                server = Connection::Udp(args.server_addr);
                let listener = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)).unwrap();
                listener.set_nonblocking(true).unwrap();
                listener_addr = Connection::Udp(listener.local_addr().unwrap());
                std::thread::spawn(move || {
                    let mut buf = [0u8; 9001];
                    while keep_running.load(Ordering::Relaxed) {
                        std::thread::sleep(Duration::from_millis(33));
                        let Ok(amt) = listener.recv(&mut buf) else { continue; };
                        let Ok(msg) = PacmanMessage::from_bytes(&buf[..amt]) else { continue; };
                        send.send(msg).unwrap();
                    }
                });
            }
        }
        client::run(server, listener_addr, recv, keep_running);
        println!("Client was terminated. Trying to connect to server again in 10 seconds...");
        std::thread::sleep(Duration::from_secs(10));
    }
}
