use serde::{Deserialize, Serialize};
use tauri::Runtime;
use bincode;


#[derive(Serialize,Deserialize)]
pub struct TrConfig{
    pub name: String,
    pub password: String,
    pub port_to_pub: u16,
    pub protocol: String,

    pub local_addr: String,
    pub local_port: u16,
    pub remote_host: String,
    pub remote_port: u16
}


const CONFIG_ITEM_PATH:&str="config.ini";



#[tauri::command]
pub async fn update_config_list<R: Runtime>(app: tauri::AppHandle<R>, window: tauri::Window<R>,new_config_list:Vec::<TrConfig>) -> Result<(), String> {
    let biniry=bincode::serialize(&new_config_list).unwrap();
    std::fs::write(CONFIG_ITEM_PATH,biniry).unwrap();
    Ok(())
}

#[tauri::command]
pub async fn get_config_list<R: Runtime>(app: tauri::AppHandle<R>, window: tauri::Window<R>) -> Result<Vec<TrConfig>, String> {
    let biniry=std::fs::read(CONFIG_ITEM_PATH).unwrap_or_else(|e|{
        let biniry=bincode::serialize(&Vec::<TrConfig>::new()).unwrap();
        std::fs::write(CONFIG_ITEM_PATH,&biniry).unwrap();
        biniry
    });
    let config_list:Vec<TrConfig>=bincode::deserialize(&biniry).unwrap();
    Ok(config_list)
}