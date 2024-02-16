use std::vec;

use super::*;
use mio::net::TcpStream;



#[derive(Debug)]
pub struct Controller {
    pub stream: TcpStream,
    buf: [u8; BUF_SIZE],
    indxs: usize, // index of start
    indxe: usize, // index of end
}

const BUF_SIZE: usize = 1024 * 5;
const HEAD_MSG_SIZE: usize = 6;

impl Controller {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            buf: [0; BUF_SIZE],
            indxs: 0,
            indxe: 0,
        }
    }

    fn parse_head(head: &[u8]) -> (u16, u32) {
        let flag = u16::from_be_bytes([head[0], head[1]]);
        let size = u32::from_be_bytes([head[2], head[3], head[4], head[5]]);
        (flag, size)
    }

    fn mv_to_head(buf: &mut [u8], indxs: &mut usize, indxe: &mut usize) {
        let sub = *indxe - *indxs;
        for i in 0..sub {
            buf[i] = buf[*indxs + i];
        }
    }

    fn parse_one(buf: &mut [u8], indxs: &mut usize, indxe: &mut usize) -> Option<ControlMsg> {
        if *indxe - *indxs < HEAD_MSG_SIZE {
            Controller::mv_to_head(buf, indxs, indxe);
            return None;
        }

        let (flag, size) = Controller::parse_head(&buf[*indxs..*indxs + HEAD_MSG_SIZE]);
        if *indxe - *indxs < HEAD_MSG_SIZE + size as usize {
            Controller::mv_to_head(buf, indxs, indxe);
            return None;
        }

        let data = buf[*indxs + HEAD_MSG_SIZE..*indxs + HEAD_MSG_SIZE + size as usize].to_vec();
        *indxs += HEAD_MSG_SIZE + size as usize;

        if *indxs == *indxe {
            *indxs = 0;
            *indxe = 0;
        }

        Some(ControlMsg { flag, data })
    }

    pub fn parse(&mut self) -> std::io::Result<Option<Vec<ControlMsg>>> {
        let mut rt_list = vec![];

        loop {
            let size = match self.stream.read(&mut self.buf[self.indxe..]) {
                Ok(size) =>{
                    if size == 0 {
                        dbg!("connection closed");
                        return Ok(None);
                    }else{
                        size
                    }
                }
                Err(e) => match e.kind() {
                    std::io::ErrorKind::WouldBlock => {
                        return Ok(Some(rt_list));
                    }
                    _ => {
                        return Err(e);
                    }
                },
            };
            self.indxe += size;

            loop {
                let msg = Controller::parse_one(&mut self.buf, &mut self.indxs, &mut self.indxe);
                match msg {
                    Some(msg) => rt_list.push(msg),
                    None => break,
                }
            }
        }
    }
}
