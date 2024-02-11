use mio::net::TcpStream;
use mio::{Interest, Token};
use std::io::{Read, Write};
use std::time::Instant;

pub mod control_flow;
pub mod rule;

pub const BUF_SIZE: usize = 1024 * 100;

pub struct RestData {
    pub sent_size: usize,
    pub size: usize,
    pub buf: Box<[u8; BUF_SIZE]>,
}

impl RestData {
    pub fn new() -> Self {
        Self {
            sent_size: 0,
            size: 0,
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
            sent_size: 0,
            size: 0,
            buf: Box::new([0; BUF_SIZE]),
        }));
        token
    } else {
        let token = rest_token_list.pop().unwrap();
        buf_list[token.0 as usize].sent_size = 0;
        buf_list[token.0 as usize].size = 0;
        token
    }
}

// static mut TOTAL_TIME: u128 = 0;
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
        while buf.sent_size < buf.size {
            match tcp_list[another_index]
                .as_mut()
                .unwrap()
                .write(&buf.buf[buf.sent_size..buf.size])
            {
                Ok(size) => {
                    buf.sent_size += size;
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::WouldBlock {
                        poll.registry()
                            .reregister(
                                tcp_list[another_index].as_mut().unwrap(),
                                send_token,
                                Interest::READABLE | Interest::WRITABLE,
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

        buf.sent_size = 0;
        buf.size = 0;

        match tcp_list[the_index].as_mut().unwrap().read(&mut *buf.buf) {
            Ok(recv_size) => {
                if recv_size == 0 {
                    is_success = false;
                    break 'l;
                }

                buf.size = recv_size;
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    poll.registry()
                        .reregister(
                            tcp_list[another_index].as_mut().unwrap(),
                            send_token,
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
    is_success
}

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}
