use std::{io::{BufRead, BufReader, Read, Write}, os, process::{Command, Output, Stdio}, sync::{mpsc, Arc, Mutex}, thread};

use adb_client::{ADBDeviceExt, ADBServer};
use tauri::{AppHandle, Emitter, Manager};
use which::which;

use crate::utils::{get_app_detail_from_apk, get_app_detail_from_dir, get_app_detail_from_xapk, get_scrcpy, parse_ls_output, run_java_tool, AppDetail, Directory};

pub struct AppData {
    pub adb_server: Mutex<ADBServer>,
}

#[derive(serde::Serialize)]
pub struct Device {
    pub id: String,
    pub model: String,
    pub state: String,
}

struct Reader {
    device_name: String,
    app_handle: AppHandle,
}
struct Writer {}
impl Read for Reader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        println!("{}:{} $ {}", self.device_name, "/", String::from_utf8_lossy(buf));
        self.app_handle.emit("shell_output", buf).unwrap();
        Ok(0)
    }
}

impl Write for Writer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        Ok(0)
    }

    fn write_all(&mut self, mut buf: &[u8]) -> std::io::Result<()> {
        println!("{}", String::from_utf8_lossy(buf));
        Ok(())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[tauri::command]
pub fn hook_shell(handle: AppHandle, device_id: String) {
    let mut device = handle.state::<AppData>().adb_server.lock().unwrap().get_device_by_name(&device_id).expect("Can't get device by name");
    let mut output = Vec::new();
    device.shell_command(&["getprop", "ro.product.product.device"], &mut output).unwrap();
    let device_name = String::from_utf8_lossy(&output).trim().to_string();

    let mut reader = Reader {
        device_name: device_name.clone(),
        app_handle: handle.clone(),
    };
    let writer = Writer {};
    device.shell(&mut reader, Box::new(writer)).expect("Shell command failed");
}

#[tauri::command]
pub fn pwd(handle: AppHandle, device_id: String) -> String {
    let mut device = handle.state::<AppData>().adb_server.lock().unwrap().get_device_by_name(&device_id).expect("Can't get device by name");
    let mut output = Vec::new();
    device.shell_command(&["pwd"], &mut output).unwrap();

    String::from_utf8_lossy(&output).trim().to_string()
}

#[tauri::command]
pub fn get_list(handle: AppHandle, device_id: String, path: String) -> Vec<Directory> {
    let data = handle.state::<AppData>();
    let mut adb_server = data.adb_server.lock().unwrap();
    let mut device = adb_server.get_device_by_name(&device_id).expect("Can't get device by name");
    let mut output = Vec::new();
    device.shell_command(&["ls", "-1", "-l", &format!("\"{}\"", &path)], &mut output).unwrap();
    let folder_data = String::from_utf8_lossy(&output);
    parse_ls_output(&folder_data)
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
        let mut product_device = "".to_string();
        if device.shell_command(&["getprop", "ro.product.marketname"], &mut output).is_ok() {
            model = String::from_utf8_lossy(&output).trim().to_string();
        }
        output.clear();
        if device.shell_command(&["getprop", "ro.product.product.device"], &mut output).is_ok() {
            product_device = String::from_utf8_lossy(&output).trim().to_string();
        }

        Device {
            id,
            model: if model.is_empty() { product_device.clone() } else { model },
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