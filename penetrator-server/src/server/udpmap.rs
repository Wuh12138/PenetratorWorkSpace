use mio::net::{TcpStream, UdpSocket};

pub struct UdpMap {
    control_channel: TcpStream,
    port: u16,
}

impl UdpMap {
    pub fn new(control_channel: TcpStream, port: u16) -> Self {
        Self {
            control_channel,
            port,
        }
    }
}

impl super::MapTrait for UdpMap {
    fn update_once(&mut self) -> std::io::Result<()> {
        Ok(())
    }
    fn destroy(self) -> std::io::Result<()> {
        Ok(())
    }
    fn is_valid(&self) -> bool {
        true
    }
    fn get_info(&self) -> common::ItemInfo {
        todo!()
    }
}
