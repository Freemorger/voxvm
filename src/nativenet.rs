use std::net::{TcpListener, TcpStream, UdpSocket};

use crate::misclib::show_runtime_err;

pub struct NetController {
    connections: Vec<NetConnection>    
}

impl NetController {
    pub fn new() -> NetController {
        NetController { connections: Vec::new() }
    }

    pub fn openconn(&mut self, ntype: NetConnType, addr: &str)
        -> Result<usize, NCError> {
        let nc: NetConnection = match ntype {
            NetConnType::NewTcpS() => {
                let tcps: TcpStream = match TcpStream::connect(addr) {
                    Ok(v) => v,
                    Err(e) => {return Err(NCError::Native(e));}
                };
                NetConnection::new(NetConnType::TcpS(tcps))
            }
            NetConnType::NewTcpL() => {
                let tcpl: TcpListener = match TcpListener::bind(addr) {
                    Ok(v) => v,
                    Err(e) => {return Err(NCError::Native(e));}
                };
                NetConnection::new(NetConnType::TcpL(tcpl))
            }
            NetConnType::NewUdpS() => {
                let udps: UdpSocket = match UdpSocket::bind(addr) {
                    Ok(v) => v,
                    Err(e) => {return Err(NCError::Native(e));}
                };
                NetConnection::new(NetConnType::UdpS(udps))
            }
            _ => {
                return Err(NCError::InvalidType());
            }
        }

        self.connections.push(nc);
        Ok(self.connections.len().saturating_sub(1))
    }
}

pub enum NCError {
    Native(std::io::Error),
    InvalidType(),
}

pub struct NetConnection {
    conn: NetConnType,
}

impl NetConnection {
    pub fn new(ntype: NetConnType) -> NetConnection {
        NetConnection { conn: ntype }
    }
}

pub enum NetConnType {
    TcpS(TcpStream),
    TcpL(TcpListener),
    UdpS(UdpSocket),
    NewTcpS(),
    NewTcpL(),
    NewUdpS(),
}
