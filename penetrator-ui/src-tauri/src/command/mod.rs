pub mod add_config;
pub mod get_status;



#[tauri::command]
pub async fn test()->String{
    return "this is a test".to_string();
}



#[macro_export]
macro_rules! all_command {
    ($($x:expr)*) => {
        tauri::generate_handler![
            penetrator_ui::command::test,
            penetrator_ui::command::add_config::new_config,
            penetrator_ui::command::get_status::get_status,
            penetrator_ui::command::get_status::get_status_list

        ]


    };
}