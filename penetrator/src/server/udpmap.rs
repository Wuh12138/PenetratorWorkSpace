use crate::server::MapTrait;

pub struct UdpMap {}
impl MapTrait for UdpMap {
    fn update_once(&mut self) -> std::io::Result<()> {
        Ok(())
    }
    fn is_valid(&self) -> bool {
        true
    }
}
