// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]


use penetrator::server;
use tokio::sync::Mutex;
use penetrator_ui::{all_command,LOCAL_SERVER};


#[tokio::main]
async fn main() {
    
    unsafe {
        LOCAL_SERVER = Some(Mutex::new(server::LocalServer::new()));
    }

    tauri::Builder::default()
        .invoke_handler(all_command![])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
