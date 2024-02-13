use crate::LOCAL_SERVER;
use serde::Serialize;
use common::{ServerTrait,MapProtocol};
use tauri::Runtime;

#[derive(Serialize)]
pub struct TrItemInfo {
    uid: u128,
    local_addr: String,
    local_port: u16,
    remote_host: String,
    remote_port: u16,
    protocol: String,
}

#[tauri::command]
pub async fn get_status<R: Runtime>(app: tauri::AppHandle<R>, window: tauri::Window<R>,uid:u128) -> Result<TrItemInfo, String> {
    unsafe {
        let mut instance = LOCAL_SERVER.as_ref().unwrap().lock().await;
        let item_info=instance.get_tcp_map_with_uid(uid);
        match item_info {
            Some(info) => {
                let tr_info = TrItemInfo {
                    uid: info.uid,
                    local_addr: info.local_addr,
                    local_port: info.local_port,
                    remote_host: info.remote_addr,
                    remote_port: info.remote_port,
                    protocol: match info.protocol {
                        MapProtocol::TCP => "tcp".to_string(),
                        MapProtocol::UDP => "udp".to_string(),
                    },
                };
                Ok(tr_info)
            }
            None => Err("No such item".to_string()),
        }
    }

}

#[tauri::command]
pub async fn get_status_list<R: Runtime>(app: tauri::AppHandle<R>, window: tauri::Window<R>) -> Result<Vec::<TrItemInfo>, String> {
    let instance;
    unsafe {
        instance = LOCAL_SERVER.as_ref().unwrap().lock().await;
    }
    let info_list = instance.get_tcp_map_list();
    let mut tr_info_list = Vec::new();
    for info in info_list {
        let tr_info = TrItemInfo {
            uid: info.uid,
            local_addr: info.local_addr,
            local_port: info.local_port,
            remote_host: info.remote_addr,
            remote_port: info.remote_port,
            protocol: match info.protocol {
                MapProtocol::TCP => "tcp".to_string(),
                MapProtocol::UDP => "udp".to_string(),
            },
        };
        tr_info_list.push(tr_info);
    }
    Ok(tr_info_list)

}