use common::MapTrait;

pub struct UdpMap {}
impl MapTrait for UdpMap {
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
        todo!("Not implemented")
    }
}
