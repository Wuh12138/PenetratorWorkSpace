
pub mod config_manager;
pub mod run_manager;



#[tauri::command]
pub async fn test()->String{
    return "this is a test".to_string();
}



#[macro_export]
macro_rules! all_command {
    ($($x:expr)*) => {
        tauri::generate_handler![
            penetrator_ui::command::test,
            penetrator_ui::command::config_manager::update_config_list,
            penetrator_ui::command::config_manager::get_config_list,
            penetrator_ui::command::run_manager::start_a_map,
            penetrator_ui::command::run_manager::stop_a_map,
            penetrator_ui::command::run_manager::get_running_item,

        ]


    };
}