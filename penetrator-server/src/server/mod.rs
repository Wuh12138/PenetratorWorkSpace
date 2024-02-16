pub mod tcpmap;
pub mod udpmap;

use crate::{
    config::{self},
    control_flow::{recv_notify, NOTIFY_AUTHEN_RESP},
};
use common::{
    control_flow::notify_authen, rule::Rule, ForwardControlMsg, ForwardControlResponse, ForwardItem, ServerTrait
};
use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token};

use std::{ sync::mpsc::Sender, time::Duration};

use common::{forward, MapTrait};

use self::{tcpmap::TcpMap, udpmap::UdpMap};

struct Pair {
    stream: TcpStream,
    poll: Poll,
    _events: Events,
    pub rule:Option<Rule>,
}
const DEFAULT_CONTROL_TOKEN: Token = Token(0);
impl Pair {
    pub fn new(stream: TcpStream, poll: Poll) -> Self {
        Self { stream, poll, _events: Events::with_capacity(1),rule:None}
    }
    pub fn register(&mut self) {
        self.poll
            .registry()
            .register(&mut self.stream, DEFAULT_CONTROL_TOKEN, Interest::READABLE)
            .unwrap();

        notify_authen(&mut self.stream).unwrap();
    }

    fn is_ok(&mut self) -> Result<(),()> {

        self.poll.poll(&mut self._events, Some(Duration::from_millis(10))).unwrap();
        for event in self._events.iter() {
            if event.token() == DEFAULT_CONTROL_TOKEN {
                let msg = recv_notify(&mut self.stream).unwrap();
                if msg.flag == NOTIFY_AUTHEN_RESP {
                    let rule=Rule::from_u8(&msg.data).unwrap();
                    self.rule=Some(rule);
                }

            }
        }
        Err(())
    }
}

pub struct Server {
    pub listener: TcpListener,
    tcpctl_sender: Option<Sender<ForwardControlMsg<TcpMap>>>,
    udpctl_sender: Option<Sender<ForwardControlMsg<UdpMap>>>,
    tcprsp_recver: Option<std::sync::mpsc::Receiver<ForwardControlResponse>>,
    udprsp_recver: Option<std::sync::mpsc::Receiver<ForwardControlResponse>>,
    tcpmap_handle: Option<std::thread::JoinHandle<()>>,
    udpmap_handle: Option<std::thread::JoinHandle<()>>,
}

const EVENTS_CAPACITY: usize = 32;

static mut TCP_UID: u128 = 0;
static mut UDP_UID: u128 = 0;
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
        Self {
            listener,
            tcpctl_sender: None,
            udpctl_sender: None,
            tcprsp_recver: None,
            udprsp_recver: None,
            tcpmap_handle: None,
            udpmap_handle: None,
        }
    }

    pub fn run(&mut self) {
        let (tcpctl_sender, tcpctl_recver) = std::sync::mpsc::channel();
        let (tcprsp_sender, tcprsp_recver) = std::sync::mpsc::channel();
        let (udpctl_sender, udpctl_recver) = std::sync::mpsc::channel();
        let (udprsp_sender, udprsp_recver) = std::sync::mpsc::channel();
        self.tcpctl_sender = Some(tcpctl_sender.clone());
        self.udpctl_sender = Some(udpctl_sender.clone());
        self.tcprsp_recver = Some(tcprsp_recver);
        self.udprsp_recver = Some(udprsp_recver);

        let handle1 = forward(tcpctl_recver, tcprsp_sender);
        let handle2 = forward(udpctl_recver, udprsp_sender);

        self.tcpmap_handle = Some(handle1);
        self.udpmap_handle = Some(handle2);

        let mut poll = Poll::new().unwrap();
        let mut events = Events::with_capacity(EVENTS_CAPACITY);
        let server_token = Token(0);

        poll.registry()
            .register(&mut self.listener, server_token, Interest::READABLE)
            .unwrap();

        let mut pair_list: Vec<Pair> = Vec::new();

        loop {
            poll.poll(&mut events, Some(Duration::from_millis(20))).unwrap();
            for event in events.iter() {
                match event.token() {
                    Token(0) => {
                        while let Ok((stream, _)) = self.listener.accept() {
                            let mut pair = Pair::new(stream, Poll::new().unwrap());
                            pair.register();
                            pair_list.push(pair);
                        }
                    }
                    _ => {}
                }
            }
            let mut ok_list=vec![];
            for (i,pair) in pair_list.iter_mut().enumerate() {
                if let Ok(())=pair.is_ok(){
                    ok_list.push(i);
                }
            }
            for i in ok_list.iter().rev(){
                let pair=pair_list.swap_remove(*i);
                let rule=pair.rule.unwrap();
                Self::distribute_connection(
                    tcpctl_sender.clone(),
                    udpctl_sender.clone(),
                    pair.stream,
                    pair.poll,
                    rule,
                );
            }   





        }
    }

    // fn handle_control_msg(
    //     stream: TcpStream,
    //     msg: common::control_flow::ControlMsg,
    //     tcpctl_sender: Sender<ForwardControlMsg<TcpMap>>,
    //     udpctl_sender: Sender<ForwardControlMsg<UdpMap>>,
    // ) {
    //     match msg.flag {
    //         NOTIFY_AUTHEN_RESP => {
    //             let rule = config::Rule::from_u8(&msg.data).unwrap();
    //             let is_ok = crate::authentification::check(&rule).unwrap();
    //             if is_ok {
    //                 Self::distribute_connection(tcpctl_sender, udpctl_sender, stream, rule);
    //             }
    //         }
    //         _ => {}
    //     }
    // }

    pub fn distribute_connection(
        tcpmap_sender: Sender<ForwardControlMsg<TcpMap>>,
        udpctl_sender: Sender<ForwardControlMsg<UdpMap>>,
        stream: TcpStream,
        poll:Poll,
        rule: config::Rule,
    ) {
        let protocol = rule.protocol.as_str();
        match protocol {
            "tcp" => {
                let tcpmap = match tcpmap::TcpMap::try_new(stream,poll, rule.port_to_pub) {
                    Ok(tcpmap) => tcpmap,
                    Err(e) => {
                        dbg!("Error:{}", e);
                        return;
                    }
                };
                let item = ForwardItem::<TcpMap> {
                    uid: unsafe { TCP_UID },
                    item: tcpmap,
                };
                unsafe {
                    TCP_UID += 1;
                }
                tcpmap_sender.send(ForwardControlMsg::Add(item)).unwrap();
            }
            "udp" => {
                let udpmap = udpmap::UdpMap::new(stream, rule.port_to_pub);
                let item = ForwardItem::<UdpMap> {
                    uid: unsafe { UDP_UID },
                    item: udpmap,
                };
                unsafe {
                    UDP_UID += 1;
                }
                udpctl_sender.send(ForwardControlMsg::Add(item)).unwrap();
            }
            _ => {}
        }
    }
}

impl ServerTrait for Server {
    fn get_tcp_map_list(&self) -> Vec<common::ItemInfo> {
        self.tcpctl_sender
            .as_ref()
            .unwrap()
            .send(ForwardControlMsg::GetInfoList)
            .unwrap();
        let list = self.tcprsp_recver.as_ref().unwrap().recv().unwrap();
        match list {
            ForwardControlResponse::InfoList(list) => list,
            _ => vec![],
        }
    }
    fn get_tcp_map_with_uid(&self, uid: u128) -> Option<common::ItemInfo> {
        self.tcpctl_sender
            .as_ref()
            .unwrap()
            .send(ForwardControlMsg::GetInfo(uid))
            .unwrap();

        let item = self.tcprsp_recver.as_ref().unwrap().recv().unwrap();
        match item {
            ForwardControlResponse::Info(item) => Some(item),
            _ => None,
        }
    }

    fn get_udp_map_list(&self) -> Vec<common::ItemInfo> {
        self.udpctl_sender
            .as_ref()
            .unwrap()
            .send(ForwardControlMsg::GetInfoList)
            .unwrap();

        let list = self.udprsp_recver.as_ref().unwrap().recv().unwrap();
        match list {
            ForwardControlResponse::InfoList(list) => list,
            _ => vec![],
        }
    }

    fn get_udp_map_with_uid(&self, uid: u128) -> Option<common::ItemInfo> {
        self.udpctl_sender
            .as_ref()
            .unwrap()
            .send(ForwardControlMsg::GetInfo(uid))
            .unwrap();

        let item = self.udprsp_recver.as_ref().unwrap().recv().unwrap();
        match item {
            ForwardControlResponse::Info(item) => Some(item),
            _ => None,
        }
    }

    fn remove_tcp_map(&mut self, uid: u128) -> std::io::Result<()> {
        self.tcpctl_sender
            .as_ref()
            .unwrap()
            .send(ForwardControlMsg::Remove(uid))
            .unwrap();
        Ok(())
    }

    fn remove_udp_map(&mut self, uid: u128) -> std::io::Result<()> {
        self.udpctl_sender
            .as_ref()
            .unwrap()
            .send(ForwardControlMsg::Remove(uid))
            .unwrap();
        Ok(())
    }
}
