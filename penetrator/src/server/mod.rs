pub mod tcpmap;
pub mod udpmap;

use crate::server::tcpmap::TcpMap;
use common::rule;
use mio::net::UdpSocket;
use mio::net::{TcpListener, TcpStream};
use std::process::exit;
use std::sync::mpsc::Receiver;
use std::thread;
use std::thread::sleep;
use std::time::Duration;

trait MapTrait {
    fn update_once(&mut self) -> std::io::Result<()>;
    //fn destroy(&mut self) -> std::io::Result<()>;
    fn is_valid(&self) -> bool;
}

pub struct LocalServer {
    tcpmap_sender: std::sync::mpsc::Sender<tcpmap::TcpMap>,
    udpmap_sender: std::sync::mpsc::Sender<udpmap::UdpMap>,
}

impl LocalServer {
    pub fn new() -> Self {
        let (tcpmap_sender, tcpmap_recver) = std::sync::mpsc::channel();
        let (udpmap_sender, udpmap_recver) = std::sync::mpsc::channel();
        Self::forward(tcpmap_recver);
        Self::forward(udpmap_recver);

        Self {
            tcpmap_sender,
            udpmap_sender,
        }
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

                for i in invalid_list.iter().rev() {
                    list.swap_remove(*i);
                }

                match receiver.try_recv() {
                    Ok(item) => {
                        list.push(item);
                    }
                    Err(_) => {
                        continue;
                    }
                }
            }
        });
        handle
    }

    pub fn add_tcpmap(
        &mut self,
        local_addr: String,
        local_port: u16,
        remote_addr: String,
        remote_port: u16,
    ) {
    }
    pub fn add_tcpmap_with_rule(
        &mut self,
        rule: rule::Rule,
        local_addr: String,
        local_port: u16,
        remote_host: String,
        remote_port: u16,
    ) {
        let mut stream =
            TcpStream::connect(format!("{}:{}", remote_host, remote_port).parse().unwrap())
                .unwrap();
        let mut remote_tcpmap_port = 0;

        let mut poll = mio::Poll::new().unwrap();
        let mut events = mio::Events::with_capacity(16);
        poll.registry()
            .register(&mut stream, mio::Token(0), mio::Interest::READABLE)
            .unwrap();
        'outer: loop {
            poll.poll(&mut events, None).unwrap();
            for event in events.iter() {
                match event.token() {
                    mio::Token(0) => match common::control_flow::recv_notify(&mut stream) {
                        Ok(msg) => match msg.flag {
                            common::control_flow::NOTIFY_AUTHEN => {
                                common::control_flow::ack_authen(&mut stream, &rule).unwrap();
                                break 'outer;
                            }
                            _ => {}
                        },
                        Err(_) => {
                            break;
                        }
                    },
                    _ => {
                        println!("unexpected event");
                    }
                };
            }
        }

        poll.poll(&mut events, Some(Duration::from_millis(100)))
            .unwrap();

        poll.registry().deregister(&mut stream).unwrap();

        let mut tcpmap = tcpmap::TcpMap::new(
            local_addr,
            local_port,
            remote_host,
            remote_tcpmap_port,
            stream,
        );
        self.tcpmap_sender.send(tcpmap).unwrap();
    }
}
