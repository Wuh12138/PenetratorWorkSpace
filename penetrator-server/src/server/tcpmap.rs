use common::control_flow::ControlMsg;
use common::{fo_to, get_token_and_buf, RestData, BUF_SIZE};
use mio::net::{TcpListener, TcpStream};
use mio::{Interest, Token};
use std::collections::{HashMap, VecDeque};
use std::io::{self, Read, Write};

pub struct TcpMap {
    control_channel: TcpStream,
    poll: mio::Poll,
    events: mio::Events,

    lit_pub: TcpListener,
    lit_clt: TcpListener,
    tcp_list: Vec<Option<TcpStream>>,
    rest_token_list: Vec<Token>,
    tcp_pair: HashMap<Token, Token>,

    pub_conn_queue: VecDeque<TcpStream>,

    buf_list: Vec<Box<RestData>>,

    // flag
    is_valid: bool,
}
const CONTROL_CHANNEL_TOKEN: mio::Token = mio::Token(0);
const LIT_PUB_TOKEN: mio::Token = mio::Token(1);
const LIT_CLT_TOKEN: mio::Token = mio::Token(2);
const EVENTS_CAPACITY: usize = 32;

impl TcpMap {
    pub fn new(mut control_channel: TcpStream, pub_port: u16) -> Self {
        let mut poll = mio::Poll::new().unwrap();
        poll.registry()
            .register(
                &mut control_channel,
                CONTROL_CHANNEL_TOKEN,
                Interest::READABLE,
            )
            .unwrap();

        let config = crate::config::CONFIG.lock().unwrap();
        let mut lit_clt =
            TcpListener::bind(format!("{}:{}", config.listen_addr, 0u16).parse().unwrap()).unwrap();
        poll.registry()
            .register(&mut lit_clt, LIT_CLT_TOKEN, Interest::READABLE)
            .unwrap();
        let port = lit_clt.local_addr().unwrap().port();
        common::control_flow::notify_port(&mut control_channel, port).unwrap();

        let mut lit_pub = TcpListener::bind(
            format!("{}:{}", config.listen_addr, pub_port)
                .parse()
                .unwrap(),
        )
        .unwrap();
        poll.registry()
            .register(&mut lit_pub, LIT_PUB_TOKEN, Interest::READABLE)
            .unwrap();

        let events = mio::Events::with_capacity(EVENTS_CAPACITY);

        let mut tcp_list = vec![None, None, None];
        let buf_list = vec![
            Box::new(RestData::new()),
            Box::new(RestData::new()),
            Box::new(RestData::new()),
        ];

        Self {
            control_channel,
            poll,
            events,
            lit_pub,
            lit_clt,
            tcp_list,
            rest_token_list: vec![],
            tcp_pair: HashMap::new(),
            pub_conn_queue: VecDeque::new(),

            buf_list,

            is_valid: true,
        }
    }

    pub fn try_new(mut control_channel: TcpStream, pub_port: u16) -> io::Result<Self> {
        let mut poll = mio::Poll::new()?;
        poll.registry().register(
            &mut control_channel,
            CONTROL_CHANNEL_TOKEN,
            Interest::READABLE,
        )?;

        let config = crate::config::CONFIG.lock().unwrap();
        let mut lit_clt =
            TcpListener::bind(format!("{}:{}", config.listen_addr, 0u16).parse().unwrap()).unwrap();
        poll.registry()
            .register(&mut lit_clt, LIT_CLT_TOKEN, Interest::READABLE)?;

        let port = lit_clt.local_addr().unwrap().port();
        common::control_flow::notify_port(&mut control_channel, port)?;

        let mut lit_pub = TcpListener::bind(
            format!("{}:{}", config.listen_addr, pub_port)
                .parse()
                .unwrap(),
        )?;
        poll.registry()
            .register(&mut lit_pub, LIT_PUB_TOKEN, Interest::READABLE)?;

        let events = mio::Events::with_capacity(EVENTS_CAPACITY);

        let mut tcp_list = vec![None, None, None];
        let buf_list = vec![
            Box::new(RestData::new()),
            Box::new(RestData::new()),
            Box::new(RestData::new()),
        ];

        Ok(Self {
            control_channel,
            poll,
            events,
            lit_pub,
            lit_clt,
            tcp_list,
            rest_token_list: vec![],
            tcp_pair: HashMap::new(),
            pub_conn_queue: VecDeque::new(),

            buf_list,

            is_valid: true,
        })
    }
}

impl super::MapTrait for TcpMap {
    fn update_once(&mut self) -> std::io::Result<()> {
        self.poll.poll(
            &mut self.events,
            Option::Some(std::time::Duration::from_millis(100)),
        )?;
        for event in self.events.iter() {
            match event.token() {
                CONTROL_CHANNEL_TOKEN => {
                    let msg: Option<ControlMsg> =
                        common::control_flow::recv_notify(&mut self.control_channel).ok();
                    match msg {
                        Some(msg) => match msg.flag {
                            common::control_flow::NOTIFY_PORT_TO_NEW_CONN_RESP => {}
                            _ => {}
                        },
                        None => {
                            self.is_valid = false;
                        }
                    }
                }
                LIT_CLT_TOKEN => {
                    while let Ok((mut stream, _)) = self.lit_clt.accept() {
                        let mut pub_conn = self.pub_conn_queue.pop_front().unwrap();

                        let token = get_token_and_buf(
                            &mut self.tcp_list,
                            &mut self.rest_token_list,
                            &mut self.buf_list,
                        );
                        self.poll
                            .registry()
                            .register(&mut stream, token, Interest::READABLE)?;

                        let token_2 = get_token_and_buf(
                            &mut self.tcp_list,
                            &mut self.rest_token_list,
                            &mut self.buf_list,
                        );
                        self.poll.registry().register(
                            &mut pub_conn,
                            token_2,
                            Interest::READABLE,
                        )?;

                        self.tcp_list[token.0 as usize] = Some(stream);
                        self.tcp_list[token_2.0 as usize] = Some(pub_conn);

                        self.tcp_pair.insert(token, token_2);
                        self.tcp_pair.insert(token_2, token);
                    }
                }
                LIT_PUB_TOKEN => {
                    let mut conn_num = 0u32;
                    while let Ok((stream, _)) = self.lit_pub.accept() {
                        self.pub_conn_queue.push_back(stream);
                        conn_num += 1;
                    }
                    if conn_num > 0 {
                        common::control_flow::notify_new_tcp_map_with_num(
                            &mut self.control_channel,
                            conn_num,
                        )?;
                    }
                }
                conn_token => {
                    let is_exist = self.tcp_pair.contains_key(&conn_token);
                    if !is_exist {
                        continue;
                    }

                    let the_token = conn_token;
                    let the_index = the_token.0 as usize;

                    let another_token = self.tcp_pair.get(&conn_token).unwrap().clone(); // clone Token is cheap
                    let another_index = another_token.0 as usize;

                    let is_success = fo_to(
                        &mut self.tcp_list,
                        the_token,
                        another_token,
                        &mut self.buf_list[another_index],
                        &mut self.poll,
                    );
                    let is_success_2 = fo_to(
                        &mut self.tcp_list,
                        another_token,
                        the_token,
                        &mut self.buf_list[the_index],
                        &mut self.poll,
                    );
                    if !(is_success && is_success_2) {
                        self.poll
                            .registry()
                            .deregister(self.tcp_list[the_index].as_mut().unwrap())?;
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
                                    dbg!(e);
                                }
                            }
                        });
                        self.tcp_list[the_index] = None;
                        self.rest_token_list.push(the_token);
                        self.tcp_pair.remove(&the_token);

                        self.poll
                            .registry()
                            .deregister(self.tcp_list[another_index].as_mut().unwrap())?;
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
                                    dbg!(e);
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
    fn destroy(mut self) -> std::io::Result<()> {
        drop(self.poll);
        drop(self.control_channel);
        drop(self.lit_clt);
        drop(self.lit_pub);
        for stream in &mut self.tcp_list {
            if let Some(stream) = stream {
                stream.shutdown(std::net::Shutdown::Both)?;
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
            local_addr: self.lit_pub.local_addr().unwrap().ip().to_string(),
            local_port: self.lit_pub.local_addr().unwrap().port(),
            remote_addr: self.lit_clt.local_addr().unwrap().ip().to_string(),
            remote_port: self.lit_clt.local_addr().unwrap().port(),
            protocol: common::MapProtocol::TCP,
        }
    }
}
