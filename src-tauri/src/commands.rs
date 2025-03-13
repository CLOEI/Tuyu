use std::{process::{Command, Output}, sync::Mutex};

use adb_client::{ADBDeviceExt, ADBServer};
use tauri::{AppHandle, Manager};
use which::which;

use crate::utils::{get_app_detail_from_apk, get_app_detail_from_dir, get_app_detail_from_xapk, get_scrcpy, run_java_tool, AppDetail};

pub struct AppData {
    pub adb_server: Mutex<ADBServer>,
}

#[derive(serde::Serialize)]
pub struct Device {
    pub id: String,
    pub model: String,
    pub state: String,
}

#[tauri::command]
pub fn get_list(handle: AppHandle, device_id: String, path: String) -> Vec<String> {
    let data = handle.state::<AppData>();
    let mut adb_server = data.adb_server.lock().unwrap();
    let mut device = adb_server.get_device_by_name(&device_id).expect("Can't get device by name");
    let mut output = Vec::new();
    device.shell_command(&["ls", "-1", "-F", &format!("\"{}\"", &path)], &mut output).unwrap();
    let folder_data = String::from_utf8_lossy(&output);
    folder_data.lines().map(|line| line.to_string()).collect::<Vec<String>>()
}

#[tauri::command]
pub fn execute_scrcpy(device_id: String) {
    let scrcpy = get_scrcpy().expect("scrcpy not found");
    Command::new(scrcpy)
        .args(&["-s", &device_id])
        .spawn()
        .expect("Failed to start scrcpy");
}

#[tauri::command]
pub fn get_adb_devices(handle: AppHandle) -> Vec<Device> {
    let data = handle.state::<AppData>();
    let mut adb_server = data.adb_server.lock().unwrap();
    let devices = adb_server.devices().expect("Can't fetch devices from ADB");

    devices.iter().map(|data| {
        let id = data.identifier.clone();
        let mut device = adb_server.get_device_by_name(&id).expect("Can't get device by name");
        let mut output = Vec::new();
        let mut model = "".to_string();
        if device.shell_command(&["getprop", "ro.product.marketname"], &mut output).is_ok() {
            model = String::from_utf8_lossy(&output).trim().to_string();
        }
        Device {
            id,
            model,
            state: data.state.to_string()
            
        }
    }).collect::<Vec<Device>>()
}

#[tauri::command]
pub fn sign_apk(handle: AppHandle, apk_path: String) {
    run_java_tool(
        handle,
        "apksigner",
        &["sign", "--ks-key-alias", "tuyu", "--ks-pass", "pass:tuyu123", "--ks", "binaries/tuyu.keystore", &apk_path],
        "App signed successfully".to_string(),
        "Failed to sign app".to_string(),
    );
}

#[tauri::command]
pub fn merge_xapk(handle: AppHandle, xapk_path: String, name: String) {
    let output_path = format!("compiled/{}.apk", name);
    run_java_tool(
        handle,
        "apkeditor",
        &["m", "-i", &xapk_path, "-o", &output_path, "-f"],
        "XAPK merged successfully".to_string(),
        "Failed to merge XAPK".to_string(),
    )
}

#[tauri::command]
pub fn extract_app(handle: AppHandle, app_path: String, name: String) {
    let output_path = format!("decompiled/{}", name);
    run_java_tool(
        handle,
        "apktool",
        &["d", &app_path, "-o", &output_path, "-f"],
        "App decompiled successfully".to_string(),
        "Failed to decompile app".to_string(),
    );
}

#[tauri::command]
pub fn compile_app(handle: AppHandle, app_path: String, name: String) {
    let output_path = format!("compiled/{}.apk", name);
    run_java_tool(
        handle,
        "apktool",
        &["b", &app_path, "-o", &output_path],
        "App compiled successfully".to_string(),
        "Failed to compile app".to_string(),
    );
}

#[tauri::command]
pub fn get_app_detail(app_path: String) -> Option<AppDetail> {
    let path = std::path::Path::new(&app_path);
    
    if path.is_dir() {
        return get_app_detail_from_dir(app_path)
    } else {
        match path.extension() {
            Some(ext) => match ext.to_str().unwrap() {
                "apk" => return get_app_detail_from_apk(app_path),
                "xapk" => return get_app_detail_from_xapk(app_path),
                _ => panic!("Not Implemented!")
            }
            None => panic!("Not Implemented!")
        }
    }
}

#[tauri::command]
pub fn get_java() -> Option<String> {
    which("java").ok().map(|path| path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn get_adb() -> Option<String> {
    which("adb").ok().map(|path| path.to_string_lossy().to_string())
}