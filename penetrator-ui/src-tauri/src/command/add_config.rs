use crate::LOCAL_SERVER;
use penetrator::control_flow::rule::Rule;
use serde::{Deserialize, Serialize};
use tauri::Runtime;

#[derive(Serialize,Deserialize)]
pub struct TrConfig{
    name: String,
    password: String,
    port_to_pub: u16,
    protocol: String,

    local_addr: String,
    local_port: u16,
    remote_host: String,
    remote_port: u16
}


#[tauri::command]
pub async fn new_config<R: Runtime>(app: tauri::AppHandle<R>, window: tauri::Window<R>,config:TrConfig) -> Result<(), String> {

    unsafe{
        let mut instance=LOCAL_SERVER.as_ref().unwrap().lock().await;
        let rule=Rule{
            name:config.name,
            password:config.password,
            port_to_pub:config.port_to_pub,
            protocol:config.protocol
        };
        let local_addr=config.local_addr;
        let local_port=config.local_port;
        let remote_host=config.remote_host;
        let remote_port=config.remote_port;

        instance.add_tcpmap_with_rule(rule, local_addr, local_port, remote_host, remote_port);
        

    }
    Ok(())
}