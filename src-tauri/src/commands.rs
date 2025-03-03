use tauri::AppHandle;
use which::which;

use crate::utils::{get_app_detail_from_apk, get_app_detail_from_dir, run_apktool_command, AppDetail};

#[tauri::command]
pub fn extract_app(handle: AppHandle, app_path: String, name: String) {
    let output_path = format!("decompiled/{}", name);
    run_apktool_command(
        handle,
        &["d", &app_path, "-o", &output_path, "-f"],
        "App decompiled successfully".to_string(),
        "Failed to decompile app".to_string(),
    );
}

#[tauri::command]
pub fn compile_app(handle: AppHandle, app_path: String, name: String) {
    let output_path = format!("compiled/{}.apk", name);
    run_apktool_command(
        handle,
        &["b", &app_path, "-o", &output_path],
        "App compiled successfully".to_string(),
        "Failed to compile app".to_string(),
    );
}

#[tauri::command]
pub fn get_app_detail(app_path: String) -> Option<AppDetail> {
    let path = std::path::Path::new(&app_path);
    
    if path.is_dir() {
        return get_app_detail_from_dir(app_path);
    } else {
        match path.extension() {
            Some(ext) => match ext.to_str().unwrap() {
                "apk" => return get_app_detail_from_apk(app_path),
                "xapk" => {},
                _ => panic!("Not Implemented!")
            }
            None => panic!("Not Implemented!")
        }
    }

    None
}

#[tauri::command]
pub fn get_java() -> Option<String> {
    which("java").ok().map(|path| path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn get_adb() -> Option<String> {
    which("adb").ok().map(|path| path.to_string_lossy().to_string())
}