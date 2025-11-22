use std::{io::{Read, Write}, net::{SocketAddr, TcpListener, TcpStream, UdpSocket}};

use crate::{misclib::{show_runtime_err, u8_slice_to_u16_vec, vec16_into_vec8}, registers::Register, vm::VM};

#[derive(Debug)]
pub struct NetController {
    connections: Vec<NetConnection>    
}

impl NetController {
    pub fn new() -> NetController {
        NetController { connections: Vec::new() }
    }

    fn tryaddr(s: &str) -> Result<SocketAddr, NCError> {
            let saddr: SocketAddr = match s.parse() {
                    Ok(v) => {return Ok(v);},
                    Err(e) => {
                        return Err(NCError::Parse());
                    }
                };
    }

    pub fn openconn(&mut self, ntype: NetConnType, addr: &str)
        -> Result<usize, NCError> {
        let nc: NetConnection = match ntype {
            NetConnType::NewTcpS() => {
                let tcps: TcpStream = match TcpStream::connect(addr) {
                    Ok(v) => v,
                    Err(e) => {return Err(NCError::Native(e));}
                }; 

                NetConnection::new(
                    NetConnType::TcpS(tcps), 
                    NetController::tryaddr(addr)?    
                )
            }
            NetConnType::NewTcpL() => {
                let tcpl: TcpListener = match TcpListener::bind(addr) {
                    Ok(v) => v,
                    Err(e) => {return Err(NCError::Native(e));}
                };
                NetConnection::new(
                    NetConnType::TcpL(tcpl), 
                    NetController::tryaddr(addr)?
                )
            }
            NetConnType::NewUdpS() => {
                let udps: UdpSocket = match UdpSocket::bind(addr) {
                    Ok(v) => v,
                    Err(e) => {return Err(NCError::Native(e));}
                };
                NetConnection::new(
                    NetConnType::UdpS(udps),
                    NetController::tryaddr(addr)?
                )
            }
            _ => {
                return Err(NCError::InvalidType());
            }
        };

        self.connections.push(nc);
        Ok(self.connections.len().saturating_sub(1))
    }
}

#[derive(Debug)]
pub enum NCError {
    Native(std::io::Error),
    InvalidType(),
    Parse(),
}

#[derive(Debug)]
pub struct NetConnection {
    conn: NetConnType,
    pub addr: SocketAddr
}

impl NetConnection {
    pub fn new(ntype: NetConnType, addr: SocketAddr) -> NetConnection {
        NetConnection { conn: ntype, addr: addr }
    }
}

#[derive(Debug)]
pub enum NetConnType {
    TcpS(TcpStream),
    TcpL(TcpListener),
    UdpS(UdpSocket),
    NewTcpS(),
    NewTcpL(),
    NewUdpS(),
}


/// Ncall 0x20
/// r1 is conn type (1 - tcpstream, 2 - tcplistener,
/// 3 - udpsocket)
/// r2 is heap ptr to addr
/// r3 is count
/// returns idx into r0 
pub fn ncall_nc_bind(vm: &mut VM) {
    let conn_type_idx = vm.registers[1].as_u64();

    let conn_type: NetConnType = match conn_type_idx {
        1 => NetConnType::NewTcpS(),
        2 => NetConnType::NewTcpL(),
        3 => NetConnType::NewUdpS(),
        other => {
            show_runtime_err(vm, &format!("Invalid connection type: {}", other));
            vm.exceptions_active.push(crate::exceptions::Exception::InvalidDataType);
            return;
        }
    };

    let src_ptr = vm.registers[2].as_u64();
    let count = vm.registers[3].as_u64();

    let addr_bytes: Vec<u8> = match vm.heap.read(src_ptr, count) {
        Ok(v) => v,
        Err(()) => {
            show_runtime_err(vm, "Can't read from heap");
            vm.exceptions_active.push(crate::exceptions::Exception::HeapReadFault);
            return;
        }
    };

    let addr = String::from_utf16_lossy(
        &u8_slice_to_u16_vec(&addr_bytes)
    );

    let idx: usize = match vm.nc.openconn(conn_type, &addr) {
        Ok(v) => v,
        Err(e) => {
            show_runtime_err(vm, &format!("Error opening connection: {:#?}", e));
            vm.exceptions_active.push(crate::exceptions::Exception::NativeFault);
            return;
        }
    };

    vm.registers[0] = Register::uint(idx as u64);
}

/// ncall 0x21 
/// r1 is net conn idx 
pub fn ncall_nc_close(vm: &mut VM) {
    let nind: usize = vm.registers[1].as_u64() as usize;

    if nind >= vm.nc.connections.len() {
        show_runtime_err(vm, "Net conn idx >= nc conns len");
        vm.exceptions_active.push(crate::exceptions::Exception::NativeFault);
        return;
    }

    vm.nc.connections.remove(nind);
}

/// ncall 0x22 
/// r1 is net conn idx 
/// only for tcp listener
/// returns nind for new tcpstream into r0 
pub fn ncall_nc_accept(vm: &mut VM) {
    let nind: usize = vm.registers[1].as_u64() as usize;
    
    let conn: &NetConnection = match vm.nc.connections.get(nind) {
        Some(v) => v,
        None => {
            show_runtime_err(vm, "Net conn idx is invalid");
            vm.exceptions_active.push(crate::exceptions::Exception::NativeFault);
            return;
        }
    };

    let mut res_idx: usize = 0;
    match &conn.conn {
        NetConnType::TcpL(tl) => {
            let new_tcps = match tl.accept() {
                Ok(v) => v,
                Err(e) => {
                    show_runtime_err(vm, &format!("Error accepting connection: {}", e.to_string()));
                    vm.exceptions_active.push(crate::exceptions::Exception::NativeFault);
                    return;
                }
            };

            let newconn = NetConnection::new(
                NetConnType::TcpS(new_tcps.0), new_tcps.1
            );
            vm.nc.connections.push(newconn);
            res_idx = vm.nc.connections.len().saturating_sub(1);
        },
        _ => {
            show_runtime_err(vm, "`accept` is not implemented for not-tcplistener types");
            vm.exceptions_active.push(crate::exceptions::Exception::NativeFault);
            return;
        }
    }

    vm.registers[0] = Register::uint(res_idx as u64);
}

/// ncall 0x23 
/// r1 is nind 
/// r2 is heap ptr to data 
/// r3 is count 
pub fn ncall_nc_write(vm: &mut VM) {
    let nind: usize  = vm.registers[1].as_u64() as usize;
    let from_ptr: u64 = vm.registers[2].as_u64();
    let count:  u64 = vm.registers[3].as_u64();

    let conn: &mut NetConnection = match vm.nc.connections.get_mut(nind) {
        Some(v) => v,
        None => {
            show_runtime_err(vm, "Net conn idx is invalid");
            vm.exceptions_active.push(crate::exceptions::Exception::NativeFault);
            return;
        }
    };

    let data: Vec<u8> = match vm.heap.read(from_ptr, count) {
        Ok(b) => b,
        Err(()) => {
            show_runtime_err(vm, "Error while reading heap data for net write!");
            vm.exceptions_active.push(crate::exceptions::Exception::HeapReadFault);
            return;
        }
    };

    let mut count_written: usize = 0;
    match &mut conn.conn {
        NetConnType::TcpS(ts) => {
            match ts.write(&data) {
                Ok(c) => {
                    count_written = c;
                }
                Err(e) => {
                    show_runtime_err(vm, &format!("While writing data over network: {}", e.to_string()));
                    vm.exceptions_active.push(crate::exceptions::Exception::NativeFault);
                    return;
                }
            }
        }
        NetConnType::UdpS(us) => {
            match us.send(&data) {
                Ok(c) => {
                    count_written = c;
                }
                Err(e) => {
                    show_runtime_err(vm, &format!("While writing data over network: {}", e.to_string()));
                    vm.exceptions_active.push(crate::exceptions::Exception::NativeFault);
                    return;
                }
            }
        }
        other => {
            eprintln!("{:#?} can't write data!", other);
            vm.exceptions_active.push(crate::exceptions::Exception::NativeFault);
            return;

        }
        
    }
    vm.registers[0] = Register::uint(count_written as u64);
}

// ncall 0x24 
// r1 is nind 
// r2 is dst heap ptr 
// r3 is max to read 
// in case of udps, it will also add some 
// addr utf16be bytes into the begging. 
// make sure to have enough space
pub fn ncall_nc_read(vm: &mut VM) {
    let nind: usize = vm.registers[1].as_u64() as usize;
    let dst_ptr: u64 = vm.registers[2].as_u64();
    let maxc: usize = vm.registers[3].as_u64() as usize;

    let conn: &mut NetConnection = match vm.nc.connections.get_mut(nind) {
        Some(v) => v,
        None => {
            show_runtime_err(vm, "Net conn idx is invalid");
            vm.exceptions_active.push(crate::exceptions::Exception::NativeFault);
            return;
        }
    };
    let mut buf = vec![0u8; maxc];
    let mut readc: usize = 0;
    let mut from_addr: Option<SocketAddr> = None;

    match &mut conn.conn {
        NetConnType::TcpS(ts) => {
            match ts.read(&mut buf) {
                Ok(v) => {
                    readc = v;
                }
                Err(e) => {
                    eprintln!("Error while reading from tcp stream: {}", e.to_string());
                    vm.exceptions_active.push(crate::exceptions::Exception::NativeFault);
                    return;
                }
            }
        }
        NetConnType::UdpS(us) => {
            match us.recv_from(&mut buf) {
                Ok(dat) => {
                    readc = dat.0;
                    from_addr = Some(dat.1);
                },
                Err(e) => {
                    eprintln!("Error while reading from tcp stream: {}", e.to_string());
                    vm.exceptions_active.push(crate::exceptions::Exception::NativeFault);
                    return;
                }
            }
        }
        other => {
            eprintln!("{:#?} can't write data!", other);
            vm.exceptions_active.push(crate::exceptions::Exception::NativeFault);
            return;
        }
    }

    if let Some(ad) = from_addr {
        let addr_dbytes: Vec<u16> = ad.to_string().encode_utf16().collect();
        let addr_bytes: Vec<u8> = vec16_into_vec8(addr_dbytes);
        buf.splice(0..0, addr_bytes); 
    }

    let buf_len = buf.len();
    if let Err(()) = vm.heap.write(dst_ptr, buf) {
        show_runtime_err(vm, "Can't write heap!");
        vm.exceptions_active.push(crate::exceptions::Exception::HeapWriteFault);
        return;
    }

    vm.registers[0] = Register::uint(readc as u64);
}

// ncall 0x25 
// r1 is nind 
// r2 is dst heap ptr 
// writes conn addr into heap 
// returns written  bytes count into r0
pub fn ncall_nc_getaddr(vm: &mut VM) {
    let nind: usize = vm.registers[1].as_u64() as usize;
    let dst_ptr: u64 = vm.registers[2].as_u64();

    let conn: &NetConnection = match vm.nc.connections.get(nind) {
        Some(v) => v,
        None => {
            show_runtime_err(vm, "Net conn idx is invalid");
            vm.exceptions_active.push(crate::exceptions::Exception::NativeFault);
            return;
        }
    };

    let addr_dbytes: Vec<u16> = 
        conn.addr.to_string().encode_utf16().collect();
    let addr_bytes: Vec<u8> = vec16_into_vec8(addr_dbytes);
    let bcount: usize = addr_bytes.len();

    if let Err(()) = vm.heap.write(dst_ptr, addr_bytes) {
        show_runtime_err(vm, "Can't write heap!");
        vm.exceptions_active.push(crate::exceptions::Exception::HeapWriteFault);
        return;
    }

    vm.registers[0] = Register::uint(bcount as u64);
}
