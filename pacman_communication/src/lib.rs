use std::{net::{SocketAddrV4, TcpStream, UdpSocket, Ipv4Addr}, io::Write};

#[derive(PartialEq)]
pub enum Connection {
    Udp(SocketAddrV4),
    Tcp(SocketAddrV4), // Using TCP like UDP for simplicity (open a new connection per stream)
}

impl Connection {
    // {0 if UDP, 1 if TCP (1 byte)} - {IP (4 bytes)} - {PORT (2 bytes)}
    pub fn to_bytes(&self) -> [u8; 7] {
        let addr = match self {
            Connection::Udp(addr) => addr,
            Connection::Tcp(addr) => addr
        };
        let mut res = [0u8; 7];
        let ip = addr.ip().octets();
        let port = addr.port().to_be_bytes();
        res[0] = if let Connection::Udp(_) = self { 0 } else { 1 };
        res[1..5].copy_from_slice(&ip);
        res[5..7].copy_from_slice(&port);
        res
    }

    pub fn send(&self, bytes: &[u8]) {
        match self {
            Connection::Udp(addr) => {
                let socket = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)).unwrap();
                socket.send_to(bytes, addr).unwrap();
            }
            Connection::Tcp(addr) => {
                let mut stream = TcpStream::connect(addr).unwrap();
                let _ = stream.write(bytes).unwrap();
            }
        }
    }
}

pub enum ServerClientMessage {
}

pub enum ClientP2PMessage {
}
