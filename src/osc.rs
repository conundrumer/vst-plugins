use std::error::Error;
use std::fmt;
use std::net::{UdpSocket, SocketAddrV4, Ipv4Addr};

use rosc;
use rosc::{encoder, OscPacket, OscMessage, OscType};


const TO_PORT: u16 = 9001;
const BASE_HOST_PORT: u16 = 9100;

#[derive(Debug)]
pub struct OscSender {
    pub id: u16,
    sock: UdpSocket,
    to_addr: SocketAddrV4,
}

impl OscSender {
    pub fn new() -> Result<Self, String> {
        let home_ip: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);
        let to_addr = SocketAddrV4::new(home_ip, TO_PORT);
        let mut id = 0;
        let sock = (0..100).flat_map(|i| {
            id = i;
            let host_addr = SocketAddrV4::new(home_ip, BASE_HOST_PORT + i);
            UdpSocket::bind(host_addr)
        }).nth(0);

        if let Some(sock) = sock {
            Ok(OscSender { id, sock, to_addr })
        } else {
            Err("No available host ports".into())
        }
    }

    pub fn send(&self, address: String, msg: Vec<OscType>) -> Result<(), Box<Error>> {
        let msg_buf = encoder::encode(&OscPacket::Message(OscMessage {
            addr: address,
            args: Some(msg),
        })).map_err(|e| OscError(e))?;

        self.sock.send_to(&msg_buf, self.to_addr)?;

        Ok(())
    }

}

#[derive(Debug)]
struct OscError(rosc::OscError);
impl fmt::Display for OscError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "OSC error: {:?}", self.0)
    }
}
impl Error for OscError {
    fn description(&self) -> &str {
        "OSC error"
    }
}
