pub mod tcpmap;
pub mod udpmap;

use crate::{
    config::{self},
    control_flow:: NOTIFY_AUTHEN_RESP,
};
use common::{
    control_flow::{controller::Controller, notify_authen}, rule::Rule, ForwardControlMsg, ForwardControlResponse, ForwardItem, ServerTrait
};
use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token};

use std::{ collections::VecDeque, sync::mpsc::Sender, time::Duration};

use common::{forward, MapTrait};

use self::{tcpmap::TcpMap, udpmap::UdpMap};

struct Pair {
    controller: Controller,
    poll: Poll,
    _events: Events,
    pub rule:Option<Rule>,
}
const DEFAULT_CONTROL_TOKEN: Token = Token(0);
impl Pair {
    pub fn new(stream: TcpStream, poll: Poll) -> Self {
        Self { controller:Controller::new(stream), poll, _events: Events::with_capacity(1),rule:None}
    }
    pub fn register(&mut self) {
        self.poll
            .registry()
            .register(&mut self.controller.stream, DEFAULT_CONTROL_TOKEN, Interest::READABLE)
            .unwrap();

        notify_authen(&mut self.controller.stream).unwrap();
    }
    
    // Err(1) connection closed
    // Err(2) no response
    fn is_ok(&mut self) -> Result<(),u8> {

        self.poll.poll(&mut self._events, Some(Duration::from_millis(10))).unwrap();
        for event in self._events.iter() {
            if event.token() == DEFAULT_CONTROL_TOKEN {
                let msg_vec = match self.controller.parse() {
                    Ok(Some(msg_vec)) => msg_vec,
                    Ok(None) =>return  Err(1),
                    Err(e) => {
                        dbg!(e);
                        return Err(1);
                    }
                };
                for msg in msg_vec {
                    if msg.flag == NOTIFY_AUTHEN_RESP {
                        let rule = Rule::from_u8(&msg.data).unwrap();
                        self.rule=Some(rule);
                        return Ok(());
                    }
                }

            }
        }
        Err(2)
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
static mut _UDP_UID: u128 = 0;
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

        let mut pair_que=VecDeque::new();

        loop {
            poll.poll(&mut events, Some(Duration::from_millis(20))).unwrap();
            for event in events.iter() {
                match event.token() {
                    Token(0) => {
                        while let Ok((stream, _)) = self.listener.accept() {
                            let mut pair = Pair::new(stream, Poll::new().unwrap());
                            pair.register();
                            pair_que.push_back(pair);
                        }
                    }
                    _ => {}
                }
            }
            
            for _ in 0.. pair_que.len(){
                let mut pair=pair_que.pop_front().unwrap();
                match pair.is_ok(){
                    Ok(())=>{
                        let rule=pair.rule.unwrap();
                        Self::distribute_connection(
                            tcpctl_sender.clone(),
                            udpctl_sender.clone(),
                            pair.controller,
                            pair.poll,
                            rule,
                        );
                    }
                    Err(1)=>{
                        // connection closed
                    }
                    Err(2)=>{
                        // no response
                        pair_que.push_back(pair);
                    }
                    _=>{}
                }
            }


        }
    }



    pub fn distribute_connection(
        tcpmap_sender: Sender<ForwardControlMsg<TcpMap>>,
        _udpctl_sender: Sender<ForwardControlMsg<UdpMap>>,
        controller: Controller,
        poll:Poll,
        rule: config::Rule,
    ) {
        let protocol = rule.protocol.as_str();
        match protocol {
            "tcp" => {
                let tcpmap = match tcpmap::TcpMap::try_new(controller,poll, rule.port_to_pub) {
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
                // let udpmap = udpmap::UdpMap::new(stream, rule.port_to_pub);
                // let item = ForwardItem::<UdpMap> {
                //     uid: unsafe { UDP_UID },
                //     item: udpmap,
                // };
                // unsafe {
                //     UDP_UID += 1;
                // }
                // udpctl_sender.send(ForwardControlMsg::Add(item)).unwrap();
                todo!("udp");
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
