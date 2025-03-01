mod commands;
mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_java,
            commands::get_adb,
            commands::get_app_detail,
            commands::extract_app,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
