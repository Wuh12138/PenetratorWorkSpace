pub mod tcpmap;
pub mod udpmap;
use crate::authentification;
use crate::{
    config::{self},
    control_flow::{recv_notify, NOTIFY_AUTHEN_RESP},
};
use common::control_flow::{notify_authen, notify_port, NOTIFY_PORT_TO_NEW_CONN_RESP};
use mio::{
    event::Source,
    net::{TcpListener, TcpStream},
};
use mio::{Events, Interest, Poll, Token};
use std::fmt::format;
use std::sync::mpsc::Receiver;
use std::{
    sync::mpsc::{channel, Sender},
    thread,
};

use self::{tcpmap::TcpMap, udpmap::UdpMap};

pub trait MapTrait {
    fn update_once(&mut self) -> std::io::Result<()>;
    fn destroy(self) -> std::io::Result<()>;
    fn is_valid(&self) -> bool;
}

pub struct Server {
    pub listener: TcpListener,
}

const EVENTS_CAPACITY: usize = 32;

impl Server {
    pub fn new() -> Self {
        let glconfig = crate::config::CONFIG.lock().unwrap();
        let listener = TcpListener::bind(
            format!("{}:{}", glconfig.listen_addr, glconfig.port)
                .parse()
                .unwrap(),
        )
        .unwrap();
        drop(glconfig);
        Self { listener }
    }

    fn forward<T>(receiver: Receiver<T>) -> thread::JoinHandle<()>
    where
        T: MapTrait + Send + 'static,
    {
        let mut handle = thread::spawn(move || {
            let mut list = Vec::new();
            let mut invalid_list = Vec::new();
            let mut receiver = receiver;
            loop {
                if list.is_empty() {
                    match receiver.recv() {
                        Ok(item) => {
                            list.push(item);
                        }
                        Err(_) => {
                            break;
                        }
                    }
                }

                for (i, item) in list.iter_mut().enumerate() {
                    if item.is_valid() {
                        item.update_once().unwrap();
                    } else {
                        //item.destroy().unwrap();
                        invalid_list.push(i);
                    }
                }

                while let Some(i) = invalid_list.pop() {
                    list.swap_remove(i);
                }

                match receiver.try_recv() {
                    Ok(item) => {
                        list.push(item);
                    }
                    Err(_) => {}
                }
            }
        });
        handle
    }

    pub fn run(mut self) {
        let (tcpmap_sender, tcpmap_recver) = channel::<TcpMap>();
        let (udpmap_sender, udpmap_recver) = channel::<UdpMap>();

        Self::forward(tcpmap_recver);
        // Self::forward(udpmap_recver);

        let mut poll = Poll::new().unwrap();
        let mut events = Events::with_capacity(EVENTS_CAPACITY);
        let server_token = Token(0);
        let mut token_record = 0u32;
        let mut rest_token_list = vec![];
        poll.registry()
            .register(&mut self.listener, server_token, Interest::READABLE)
            .unwrap();

        let mut token_socket_map = std::collections::HashMap::new();

        loop {
            poll.poll(&mut events, None).unwrap();
            for event in events.iter() {
                match event.token() {
                    Token(0) => {
                        while let Ok((mut stream, _)) = self.listener.accept() {
                            let token = if rest_token_list.is_empty() {
                                token_record += 1;
                                Token(token_record as usize)
                            } else {
                                Token(rest_token_list.pop().unwrap() as usize)
                            };
                            poll.registry()
                                .register(&mut stream, token, Interest::READABLE)
                                .unwrap();

                            notify_authen(&mut stream).unwrap();
                            token_socket_map.insert(token, stream);
                        }
                    }

                    token => {
                        let mut stream = token_socket_map.remove(&token).unwrap();
                        let msg: Option<common::control_flow::ControlMsg> =
                            recv_notify(&mut stream).ok();
                        match msg {
                            Some(msg) => {
                                poll.registry().deregister(&mut stream).unwrap();
                                rest_token_list.push(token.0);

                                let whether_send = Server::handle_control_msg(
                                    stream,
                                    msg,
                                    tcpmap_sender.clone(),
                                    udpmap_sender.clone(),
                                );
                            }
                            None => {
                                poll.registry().deregister(&mut stream).unwrap();
                                rest_token_list.push(token.0);
                                TcpStream::shutdown(&stream, std::net::Shutdown::Both).unwrap();
                            }
                        }
                    }
                }
            }
        }
    }

    fn handle_control_msg(
        mut stream: TcpStream,
        msg: common::control_flow::ControlMsg,
        tcpmap_sender: Sender<TcpMap>,
        udpmap_sender: Sender<UdpMap>,
    ) {
        match msg.flag {
            NOTIFY_AUTHEN_RESP => {
                let rule = config::Rule::from_u8(&msg.data).unwrap();
                let is_ok = crate::authentification::check(&rule).unwrap();
                if is_ok {
                    Self::distribute_connection(tcpmap_sender, udpmap_sender, stream, rule);
                }
            }
            _ => {}
        }
    }

    pub fn distribute_connection(
        tcpmap_sender: Sender<TcpMap>,
        udpmap_sender: Sender<UdpMap>,
        mut stream: TcpStream,
        rule: config::Rule,
    ) {
        let protocol = rule.protocol.as_str();
        match protocol {
            "tcp" => {
                let tcpmap = tcpmap::TcpMap::new(stream, rule.port_to_pub);
                tcpmap_sender.send(tcpmap).unwrap();
            }
            "udp" => {
                let udpmap = udpmap::UdpMap::new(stream, rule.port_to_pub);
                udpmap_sender.send(udpmap).unwrap();
            }
            _ => {}
        }
    }
}
