pub mod command;


use penetrator::server;
use tokio::sync::Mutex;
pub static mut LOCAL_SERVER: Option<Mutex<server::LocalServer>> = None;