use mio::net::TcpStream;
use mio::{Interest, Token};
use std::io::{Read, Write};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

pub mod authentification;
pub mod control_flow;
pub mod rule;

pub const BUF_SIZE: usize = 1024 * 100;
pub const TIMEOUT: std::time::Duration = std::time::Duration::from_micros(50);
pub const KEEP_ALIVE_INTERVAL: std::time::Duration = std::time::Duration::from_secs(30);

pub struct RestData {
    pub indxs: usize,
    pub indxe: usize,
    pub buf: Box<[u8; BUF_SIZE]>,
}

impl RestData {
    pub fn new() -> Self {
        Self {
            indxs: 0,
            indxe: 0,
            buf: Box::new([0; BUF_SIZE]),
        }
    }
}

pub fn get_token_and_buf(
    tcp_list: &mut Vec<Option<TcpStream>>,
    rest_token_list: &mut Vec<Token>,
    buf_list: &mut Vec<Box<RestData>>,
) -> Token {
    if rest_token_list.is_empty() {
        let token = Token(tcp_list.len() as usize);
        tcp_list.push(None);
        buf_list.push(Box::new(RestData {
            indxs: 0,
            indxe: 0,
            buf: Box::new([0; BUF_SIZE]),
        }));
        token
    } else {
        let token = rest_token_list.pop().unwrap();
        buf_list[token.0 as usize].indxs = 0;
        buf_list[token.0 as usize].indxe = 0;
        token
    }
}


pub fn fo_to(
    tcp_list: &mut Vec<Option<TcpStream>>,
    recv_token: Token,
    send_token: Token,
    buf: &mut RestData,
    poll: &mut mio::Poll,
) -> bool {
    let mut is_success = true;
    let the_index = recv_token.0 as usize;
    let another_index = send_token.0 as usize;
    'l: loop {
        while buf.indxs < buf.indxe {
            match tcp_list[another_index]
                .as_mut()
                .unwrap()
                .write(&buf.buf[buf.indxs..buf.indxe])
            {
                Ok(size) => {
                    buf.indxs += size;
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::WouldBlock {

                        poll.registry()
                            .reregister(
                                tcp_list[the_index].as_mut().unwrap(),
                                recv_token,
                                Interest::READABLE,
                            )
                            .unwrap();

                        break 'l;
                    } else {
                        is_success = false;
                        break 'l;
                    }
                }
            }
        }

        buf.indxs = 0;
        buf.indxe = 0;

        match tcp_list[the_index].as_mut().unwrap().read(&mut *buf.buf) {
            Ok(recv_size) => {
                if recv_size == 0 {
                    is_success = false;
                    break 'l;
                }

                buf.indxe = recv_size;
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    break 'l;
                } else {
                    is_success = false;
                    break 'l;
                }
            }
        }
    }
    is_success
}

pub enum MapProtocol {
    TCP,
    UDP,
}

pub struct ItemInfo {
    pub uid: u128,

    pub local_addr: String,
    pub local_port: u16,
    pub remote_addr: String,
    pub remote_port: u16,
    pub protocol: MapProtocol,
}

pub struct ForwardItem<T>
where
    T: MapTrait + Send + 'static,
{
    pub uid: u128,
    pub item: T,
}

pub enum ForwardControlMsg<T>
where
    T: MapTrait + Send + 'static,
{
    Add(ForwardItem<T>),
    Remove(u128),
    GetInfo(u128),
    GetInfoList,
}

pub enum ForwardControlResponse {
    Info(ItemInfo),
    InfoList(Vec<ItemInfo>),
    Null,
    Ok,
    Err,
}

pub trait MapTrait {
    fn update_once(&mut self) -> std::io::Result<()>;
    fn destroy(self) -> std::io::Result<()>;
    fn get_info(&self) -> ItemInfo;
    fn is_valid(&self) -> bool;
}

pub fn forward<T>(
    receiver: Receiver<ForwardControlMsg<T>>,
    sender: Sender<ForwardControlResponse>,
) -> thread::JoinHandle<()>
where
    T: MapTrait + Send + 'static,
{
    let handle = thread::spawn(move || {
        let mut list = Vec::new();
        let mut uid_index_map = std::collections::HashMap::<u128, usize>::new();
        let mut invalid_list = Vec::new();
        let receiver = receiver;

        loop {
            if list.is_empty() {
                match receiver.recv() {
                    Ok(item) => match item {
                        ForwardControlMsg::Add(item) => {
                            uid_index_map.insert(item.uid, 0);
                            list.push(item);
                        }
                        _ => {
                            sender.send(ForwardControlResponse::Null).unwrap();
                            continue;
                        }
                    },
                    Err(_) => {}
                }
            }

            for (i, item) in list.iter_mut().enumerate() {
                if item.item.is_valid() {
                    item.item.update_once().unwrap();
                } else {
                    invalid_list.push(i);
                }
            }

            while let Some(i) = invalid_list.pop() {

                let uid = match list.last() {
                    Some(item) => item.uid,
                    None => break,
                };
                *uid_index_map.get_mut(&uid).unwrap() = i;
                uid_index_map.remove(&list[i].uid);
                list.swap_remove(i); 
            }

            match receiver.try_recv() {
                Ok(item) => match item {
                    ForwardControlMsg::Add(item) => {
                        uid_index_map.insert(item.uid, list.len());
                        list.push(item);
                    }
                    ForwardControlMsg::Remove(uid) => {
                        let index = match uid_index_map.get(&uid) {
                            Some(index) => *index,
                            None => continue,
                        };
                        invalid_list.push(index);
                    }
                    ForwardControlMsg::GetInfo(uid) => {
                        if let Some(index) = uid_index_map.get(&uid) {
                            let mut item_info = list[*index].item.get_info();
                            item_info.uid = list[*index].uid;
                            sender
                                .send(ForwardControlResponse::Info(item_info))
                                .unwrap();
                        }
                    }
                    ForwardControlMsg::GetInfoList => {
                        let mut info_list = Vec::new();
                        for item in &list {
                            let mut item_info = item.item.get_info();
                            item_info.uid = item.uid;
                            info_list.push(item_info);
                        }
                        sender
                            .send(ForwardControlResponse::InfoList(info_list))
                            .unwrap();
                    }
                },
                Err(_) => {}
            }
        }
    });
    handle
}

pub trait ServerTrait {
    fn get_tcp_map_list(&self) -> Vec<ItemInfo>;
    fn get_udp_map_list(&self) -> Vec<ItemInfo>;
    fn get_tcp_map_with_uid(&self, uid: u128) -> Option<ItemInfo>;
    fn get_udp_map_with_uid(&self, uid: u128) -> Option<ItemInfo>;
    fn remove_tcp_map(&mut self, uid: u128) -> std::io::Result<()>;
    fn remove_udp_map(&mut self, uid: u128) -> std::io::Result<()>;
}

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}
