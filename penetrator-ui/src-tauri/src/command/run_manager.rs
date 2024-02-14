use std::io::IsTerminal;

use super::config_manager::TrConfig;
use crate::LOCAL_SERVER;
use common::{ItemInfo, ServerTrait};
use serde::{Deserialize, Serialize};
use tauri::Runtime;
use common::rule::Rule;

#[tauri::command]
pub async fn start_a_map<R: Runtime>(app: tauri::AppHandle<R>, window: tauri::Window<R>,config:TrConfig) -> Result<u128, String> {
    let uid;
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

        uid=instance.add_tcpmap_with_rule(rule, local_addr, local_port, remote_host, remote_port)
        

    }
    if let Ok(uid)=uid{
        Ok(uid)
    }else{
        Err("failed to add tcpmap".to_string())
    }
}


// Since JSON doesn't support BigInt, JavaScript's Number doesn't represent numbers in Rust well, e.g. u64,u128 etc.
#[tauri::command]
pub async fn stop_a_map<R: Runtime>(app: tauri::AppHandle<R>, window: tauri::Window<R>,uid:u32) -> Result<(), String> {
    unsafe {
        let mut instance = LOCAL_SERVER.as_ref().unwrap().lock().await;
        instance.remove_tcp_map(uid as u128).unwrap();
    }
    Ok(())
}

#[derive(Serialize,Deserialize)]
pub struct TrItemInfo{
    pub uid:u32, // Since JSON doesn't support BigInt, JavaScript's Number doesn't represent numbers in Rust well, e.g. u64,u128 etc.
    pub local_addr:String,
    pub local_port:u16,
    pub remote_host:String,
    pub remote_port:u16,
    pub protocol:String
}

#[tauri::command]
pub async fn get_running_item<R: Runtime>(app: tauri::AppHandle<R>, window: tauri::Window<R>) -> Result<Vec<TrItemInfo>, String> {
    let mut rt=vec![];
    unsafe {
        let instance = LOCAL_SERVER.as_ref().unwrap().lock().await;
        let item_list:Vec<ItemInfo>=instance.get_tcp_map_list();
        for item in item_list{
            let rt_item=TrItemInfo{
                uid:item.uid as u32,
                local_addr:item.local_addr,
                local_port:item.local_port,
                remote_host:item.remote_addr,
                remote_port:item.remote_port,
                protocol:match item.protocol{
                    common::MapProtocol::TCP=>"TCP".to_string(),
                    common::MapProtocol::UDP=>"UDP".to_string()
                }
            };
            rt.push(rt_item);
        }

    }
    Ok(rt)
}
