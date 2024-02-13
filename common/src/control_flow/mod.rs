use mio::net::TcpStream;
use std::io::{Read, Write};

pub use crate::rule;

pub const NOTIFY_AUTHEN: u16 = 0x0001u16;
pub const NOTIFY_AUTHEN_RESP: u16 = 0x0002u16;
pub const NOTIFY_PORT_TO_CONN: u16 = 0x0003u16;
pub const NOTIFY_PORT_TO_NEW_CONN_RESP: u16 = 0x0004u16;

pub const NOTIFY_NEW_TCP_MAP: u16 = 0x0005u16;
pub const NOTIFY_NEW_TCP_MAP_RESP: u16 = 0x0006u16;

#[derive(Debug)]
pub struct ControlMsg {
    pub flag: u16,
    pub data: Vec<u8>,
}

pub fn notify(stream: &mut TcpStream, msg: &ControlMsg) -> std::io::Result<()> {
    let mut data = vec![];
    data.extend(&msg.flag.to_be_bytes());
    data.extend((msg.data.len() as u32).to_be_bytes());
    data.extend(&msg.data);
    stream.write(&data)?;
    Ok(())
}

pub fn recv_notify(stream: &mut TcpStream) -> std::io::Result<ControlMsg> {
    let mut flag = [0u8; 2];
    stream.read_exact(&mut flag)?;
    let flag = u16::from_be_bytes(flag);
    let size = {
        let mut size = [0u8; 4];
        stream.read_exact(&mut size)?;
        u32::from_be_bytes(size)
    };
    let mut data = vec![0u8; size as usize];
    stream.read_exact(&mut data)?;

    Ok(ControlMsg { flag, data })
}

//server
pub fn notify_authen(stream: &mut TcpStream) -> std::io::Result<()> {
    let msg = ControlMsg {
        flag: NOTIFY_AUTHEN,
        data: vec![],
    };
    notify(stream, &msg)
}

// client
pub fn ack_authen(stream: &mut TcpStream, rule: &rule::Rule) -> std::io::Result<()> {
    let data = rule.to_u8().unwrap();
    let msg = ControlMsg {
        flag: NOTIFY_AUTHEN_RESP,
        data,
    };
    notify(stream, &msg)
}

// server
pub fn notify_port(stream: &mut TcpStream, port: u16) -> std::io::Result<()> {
    let data = port.to_be_bytes();
    let msg = ControlMsg {
        flag: NOTIFY_PORT_TO_CONN,
        data: data.to_vec(),
    };
    notify(stream, &msg)
}

// client
pub fn recv_port_notify(stream: &mut TcpStream) -> std::io::Result<u16> {
    let msg = recv_notify(stream)?;
    let port = u16::from_be_bytes(msg.data.as_slice().try_into().unwrap());
    Ok(port)
}

pub fn ack_notify_port(stream: &mut TcpStream) -> std::io::Result<()> {
    let msg = ControlMsg {
        flag: NOTIFY_PORT_TO_NEW_CONN_RESP,
        data: vec![],
    };
    notify(stream, &msg)
}

// server
pub fn notify_new_tcp_map(stream: &mut TcpStream) -> std::io::Result<()> {
    let msg = ControlMsg {
        flag: NOTIFY_NEW_TCP_MAP,
        data: vec![],
    };
    notify(stream, &msg)
}

pub fn notify_new_tcp_map_with_num(stream: &mut TcpStream, num: u32) -> std::io::Result<()> {
    let data = num.to_be_bytes();
    let msg = ControlMsg {
        flag: NOTIFY_NEW_TCP_MAP,
        data: data.to_vec(),
    };
    notify(stream, &msg)
}

// client
pub fn ack_new_tcp_map(stream: &mut TcpStream) -> std::io::Result<()> {
    let msg = ControlMsg {
        flag: NOTIFY_NEW_TCP_MAP_RESP,
        data: vec![],
    };
    notify(stream, &msg)
}
