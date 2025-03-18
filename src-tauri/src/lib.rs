use std::sync::Mutex;

use adb_client::ADBServer;
use tauri::Manager;

mod commands;
mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let mut adb_server = ADBServer::default();
            adb_server.set_adb_path(utils::get_adb());
            app.manage(commands::AppData { 
                adb_server: Mutex::new(adb_server)
             });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_java,
            commands::get_adb,
            commands::get_app_detail,
            commands::extract_app,
            commands::compile_app,
            commands::merge_xapk,
            commands::sign_apk,
            commands::get_adb_devices,
            commands::execute_scrcpy,
            commands::get_list,
            commands::hook_shell
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
