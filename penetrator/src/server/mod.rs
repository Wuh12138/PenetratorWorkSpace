pub mod tcpmap;
pub mod udpmap;

use common::{authentification::AuthError, rule, ForwardItem, ServerTrait};

use mio::net::TcpStream;

use std::thread;

use common::{forward, ForwardControlMsg, ForwardControlResponse, MapTrait};

use self::{tcpmap::TcpMap, udpmap::UdpMap};

static mut TCP_UID: u128 = 0;
pub struct LocalServer {
    tcpctl_sender: std::sync::mpsc::Sender<ForwardControlMsg<TcpMap>>,
    udpctl_sender: std::sync::mpsc::Sender<ForwardControlMsg<UdpMap>>,

    tcprsp_recver: std::sync::mpsc::Receiver<ForwardControlResponse>,
    udprsp_recver: std::sync::mpsc::Receiver<ForwardControlResponse>,
    tcpmap_handle: thread::JoinHandle<()>,
    udpmap_handle: thread::JoinHandle<()>,
}

impl LocalServer {
    pub fn new() -> Self {
        let (tcpctl_sender, tcpctl_recver) = std::sync::mpsc::channel();
        let (tcprsp_sender, tcprsp_recver) = std::sync::mpsc::channel();
        let (udpctl_sender, udpctl_recver) = std::sync::mpsc::channel();
        let (udprsp_sender, udprsp_recver) = std::sync::mpsc::channel();
        let handle1 = forward(tcpctl_recver, tcprsp_sender);
        let handle2 = forward(udpctl_recver, udprsp_sender);

        Self {
            tcpctl_sender,
            udpctl_sender,
            tcprsp_recver,
            udprsp_recver,
            tcpmap_handle: handle1,
            udpmap_handle: handle2,
        }
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
    )->Result<u128,AuthError> {
        let mut stream =
            TcpStream::connect(format!("{}:{}", remote_host, remote_port).parse().unwrap())
                .unwrap();
        let remote_tcpmap_port = 0;

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
                                poll.registry()
                                .reregister(&mut stream, mio::Token(0), mio::Interest::READABLE)
                                .unwrap();

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

        poll.registry().deregister(&mut stream).unwrap();

        let mut tcpmap = tcpmap::TcpMap::new(
            local_addr,
            local_port,
            remote_host,
            remote_tcpmap_port,
            stream,
        );
        let item = ForwardItem::<TcpMap> {
            uid: unsafe { TCP_UID },
            item: tcpmap,
        };
        unsafe {
            TCP_UID += 1;
        }
        self.tcpctl_sender
            .send(ForwardControlMsg::Add(item))
            .unwrap();
        Ok(unsafe { TCP_UID - 1 })
    }

}

impl ServerTrait for LocalServer {
    fn get_tcp_map_list(&self) -> Vec<common::ItemInfo> {
        self.tcpctl_sender
            .send(ForwardControlMsg::GetInfoList)
            .unwrap();
        match self.tcprsp_recver.recv().unwrap() {
            ForwardControlResponse::InfoList(list) => list,
            _ => vec![],
        }
    }
    fn get_tcp_map_with_uid(&self, uid: u128) -> Option<common::ItemInfo> {
        self.tcpctl_sender
            .send(ForwardControlMsg::GetInfo(uid))
            .unwrap();
        match self.tcprsp_recver.recv().unwrap() {
            ForwardControlResponse::Info(item) => Some(item),
            _ => None,
        }
    }
    fn get_udp_map_list(&self) -> Vec<common::ItemInfo> {
        self.udpctl_sender
            .send(ForwardControlMsg::GetInfoList)
            .unwrap();
        match self.udprsp_recver.recv().unwrap() {
            ForwardControlResponse::InfoList(list) => list,
            _ => vec![],
        }
    }
    fn get_udp_map_with_uid(&self, uid: u128) -> Option<common::ItemInfo> {
        self.udpctl_sender
            .send(ForwardControlMsg::GetInfo(uid))
            .unwrap();
        match self.udprsp_recver.recv().unwrap() {
            ForwardControlResponse::Info(item) => Some(item),
            _ => None,
        }
    }
    fn remove_tcp_map(&mut self, uid: u128) -> std::io::Result<()> {
        self.tcpctl_sender
            .send(ForwardControlMsg::Remove(uid))
            .unwrap();
        Ok(())
    }
    fn remove_udp_map(&mut self, uid: u128) -> std::io::Result<()> {
        self.udpctl_sender
            .send(ForwardControlMsg::Remove(uid))
            .unwrap();
        Ok(())
    }
}