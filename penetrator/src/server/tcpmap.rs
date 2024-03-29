use crate::server::MapTrait;
use common::control_flow::{ControlMsg, KEEP_ALIVE};
use common::{fo_to, get_token_and_buf, RestData, BUF_SIZE};
use mio::net::TcpStream;
use mio::Token;
use std::collections::HashMap;
use common::control_flow::controller::Controller;

pub struct TcpMap {
    local_addr: String,
    local_port: u16,
    remote_addr: String,
    remote_port: u16,
    //control_stream: TcpStream,
    controller: Controller,
    buf_list: Vec<Box<RestData>>,

    poll: mio::Poll,
    events: mio::Events,

    tcp_pair: HashMap<Token, Token>,
    tcp_list: Vec<Option<TcpStream>>,
    rest_token_list: Vec<Token>,

    // flag
    is_valid: bool,
    last_keep_alive: std::time::Instant,
}

const EVENTS_CAPACITY: usize = 32;

const CONTROL_STREAM_TOKEN: mio::Token = mio::Token(0);
impl TcpMap {
    pub fn new(
        local_addr: String,
        local_port: u16,
        remote_addr: String,
        remote_port: u16,
        control_stream: TcpStream,
        poll: mio::Poll,
    ) -> TcpMap {

        let events = mio::Events::with_capacity(EVENTS_CAPACITY);


        TcpMap {
            local_addr,
            local_port,
            remote_addr,
            remote_port,
            controller: Controller::new(control_stream),
            buf_list: vec![Box::new(RestData {
                indxs: 0,
                indxe: 0,
                buf: Box::new([0; BUF_SIZE]),
            })],

            poll,
            events,

            tcp_pair: HashMap::new(),
            tcp_list: vec![None],
            rest_token_list: Vec::new(),

            is_valid: true,
            last_keep_alive: std::time::Instant::now(),
        }
    }

    fn handle_control_msg(
        msg: ControlMsg,
        local_addr: &String,
        local_port: &u16,
        remote_addr: &String,
        remote_port: &mut u16,
        tcp_list: &mut Vec<Option<TcpStream>>,
        rest_token_list: &mut Vec<Token>,
        buf_list: &mut Vec<Box<RestData>>,
        tcp_pair: &mut HashMap<Token, Token>,
        _control_stream: &mut TcpStream,
        poll: &mut mio::Poll,
    ) {
        match msg.flag {
            common::control_flow::NOTIFY_NEW_TCP_MAP => {
                let data = msg.data;
                let num = u32::from_be_bytes(data.as_slice().try_into().unwrap());
                for _ in 0..num {
                    let mut stream_to_local = TcpStream::connect(
                        format!("{}:{}", local_addr, local_port).parse().unwrap(),
                    )
                    .unwrap();
                    let mut stream_to_remote = TcpStream::connect(
                        format!("{}:{}", remote_addr, remote_port).parse().unwrap(),
                    )
                    .unwrap();
                    let token_to_local = get_token_and_buf(tcp_list, rest_token_list, buf_list);
                    let token_to_remote = get_token_and_buf(tcp_list, rest_token_list, buf_list);
                    poll.registry()
                        .register(
                            &mut stream_to_local,
                            token_to_local,
                            mio::Interest::READABLE,
                        )
                        .unwrap();
                    poll.registry()
                        .register(
                            &mut stream_to_remote,
                            token_to_remote,
                            mio::Interest::READABLE,
                        )
                        .unwrap();
                    tcp_pair.insert(token_to_local, token_to_remote);
                    tcp_pair.insert(token_to_remote, token_to_local);
                    tcp_list[token_to_local.0 as usize] = Some(stream_to_local);
                    tcp_list[token_to_remote.0 as usize] = Some(stream_to_remote);
                }

                // common::control_flow::ack_new_tcp_map(&mut self.control_stream)
                //     .unwrap();
            }
            common::control_flow::NOTIFY_PORT_TO_CONN => {
                let data = msg.data;
                *remote_port = u16::from_be_bytes(data.as_slice().try_into().unwrap());
                //common::control_flow::ack_notify_port(control_stream).unwrap();
            }
            KEEP_ALIVE => {
                // do nothing
            }
            _ => {}
        }
    }
}

impl MapTrait for TcpMap {
    fn update_once(&mut self) -> std::io::Result<()> {
        //keep alive
        if self.last_keep_alive.elapsed() > common::KEEP_ALIVE_INTERVAL {
            self.controller.keep_alive().unwrap_or_else(|e|{
                dbg!(e);
                self.is_valid=false;
                return;
            });
            self.last_keep_alive = std::time::Instant::now();
        }

        self.poll.poll(&mut self.events, Some(common::TIMEOUT)).unwrap(); 
        for event in self.events.iter() {
            match event.token() {
                CONTROL_STREAM_TOKEN => {

                    self.last_keep_alive = std::time::Instant::now();

                    let msg_vec = match self.controller.parse() {
                        Ok(Some(msg_vec)) => msg_vec,
                        Ok(None) => {
                            self.is_valid = false;
                            return Ok(());
                        }
                        Err(e) => {
                            dbg!(e);
                            self.is_valid = false;
                            return Ok(());
                        }
                    };

                    for msg in msg_vec {
                        TcpMap::handle_control_msg(
                            msg,
                            &self.local_addr,
                            &self.local_port,
                            &self.remote_addr,
                            &mut self.remote_port,
                            &mut self.tcp_list,
                            &mut self.rest_token_list,
                            &mut self.buf_list,
                            &mut self.tcp_pair,
                            &mut self.controller.stream,
                            &mut self.poll,
                        );
                    }



                }
                conn_token => {
                    let is_exist = self.tcp_pair.contains_key(&conn_token);
                    if !is_exist {
                        continue;
                    }
                    let the_token = conn_token;
                    let the_index = the_token.0 as usize;

                    let another_token = self.tcp_pair.get(&the_token).unwrap().clone(); // clone
                    let another_index = another_token.0 as usize;

                    let is_success = fo_to(
                        &mut self.tcp_list,
                        the_token,
                        another_token,
                        &mut self.buf_list[another_index],
                        &mut self.poll,
                    );
                    let is_success2 = fo_to(
                        &mut self.tcp_list,
                        another_token,
                        the_token,
                        &mut self.buf_list[the_index],
                        &mut self.poll,
                    );
                    if !(is_success && is_success2) {
                        self.poll
                            .registry()
                            .deregister(self.tcp_list[the_index].as_mut().unwrap()).unwrap();
                        TcpStream::shutdown(
                            self.tcp_list[the_index].as_mut().unwrap(),
                            std::net::Shutdown::Both,
                        )
                        .unwrap_or_else(|e| {
                            match e.kind() {
                                std::io::ErrorKind::ConnectionReset => {
                                    // do nothing
                                }
                                _ => {
                                    //dbg!(e);
                                }
                            }
                        });
                        self.tcp_list[the_index] = None;
                        self.rest_token_list.push(the_token);
                        self.tcp_pair.remove(&the_token);

                        self.poll
                            .registry()
                            .deregister(self.tcp_list[another_index].as_mut().unwrap()).unwrap();
                        TcpStream::shutdown(
                            self.tcp_list[another_index].as_mut().unwrap(),
                            std::net::Shutdown::Both,
                        )
                        .unwrap_or_else(|e| {
                            match e.kind() {
                                std::io::ErrorKind::ConnectionReset => {
                                    // do nothing
                                }
                                _ => {
                                    //
                                }
                            }
                        });
                        self.tcp_list[another_index] = None;
                        self.rest_token_list.push(another_token);
                        self.tcp_pair.remove(&another_token);
                    }
                }
            }
        }

        Ok(())
    }

    fn destroy(self) -> std::io::Result<()> {
        drop(self.controller.stream);
        for conn in self.tcp_list {
            if let Some(stream) = conn {
                TcpStream::shutdown(&stream, std::net::Shutdown::Both).unwrap();
            }
        }

        Ok(())
    }

    fn is_valid(&self) -> bool {
        self.is_valid
    }

    fn get_info(&self) -> common::ItemInfo {
        common::ItemInfo {
            uid: 0,
            local_addr: self.local_addr.clone(),
            local_port: self.local_port,
            remote_addr: self.remote_addr.clone(),
            remote_port: self.remote_port,
            protocol: common::MapProtocol::TCP,
        }
    }
}
