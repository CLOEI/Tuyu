use std::{fs::File, io::{BufRead, Read}};

use tauri::Emitter;
use which::which;
use zip::ZipArchive;
use base64::{engine::general_purpose, Engine};

use crate::utils::get_aapt2;

#[derive(Debug, serde::Serialize)]
pub struct AppDetail {
    name: String,
    package_name: String,
    version: String,
    min_sdk: String,
    target_sdk: String,
    is_32bit: bool,
    is_64bit: bool,
    icon_base64: Option<String>,
}

#[tauri::command]
pub fn extract_app(handle: tauri::AppHandle, app_path: String, name: String) {
    let apktool_path = std::path::Path::new("binaries/apktool.jar");
    if !apktool_path.exists() {
        handle.emit("log", "apktool.jar not found").unwrap();
        return;
    }

    let mut cmd = std::process::Command::new("java")
        .args(&["-jar", apktool_path.to_str().unwrap(), "d", &app_path, "-o", format!("decompiled/{}", name).as_str(), "-f"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("failed to execute apktool");

    let stdout = cmd.stdout.take().expect("failed to get stdout");
    let stderr = cmd.stderr.take().expect("failed to get stderr");

    let handle_clone = handle.clone();
    std::thread::spawn(move || {
        let mut reader = std::io::BufReader::new(stdout);
        let mut buffer = String::new();
        while reader.read_line(&mut buffer).unwrap() > 0 {
            handle_clone.emit("log-info", buffer.trim()).unwrap();
            buffer.clear();
        }
    });

    let handle_clone = handle.clone();
    std::thread::spawn(move || {
        let mut reader = std::io::BufReader::new(stderr);
        let mut buffer = String::new();
        while reader.read_line(&mut buffer).unwrap() > 0 {
            handle_clone.emit("log-error", buffer.trim()).unwrap();
            buffer.clear();
        }
    });

    std::thread::spawn(move || {
        let status = cmd.wait().expect("failed to wait on child");
        if status.success() {
            handle.emit("log-info", "I: App decompiled successfully").unwrap();
        } else {
            handle.emit("log-error", "E: Failed to decompile app").unwrap();
        }
    });

}

#[tauri::command]
pub fn get_app_detail(app_path: String) -> AppDetail {
    let path = std::path::Path::new(&app_path);
    
    if path.is_dir() {
        panic!("Not Implemented!")
    } else {
        match path.extension() {
            Some(ext) => match ext.to_str().unwrap() {
                "apk" => {},
                "xapk" => {},
                _ => panic!("Not Implemented!")
            }
            None => panic!("Not Implemented!")
        }
    }

    let aapt2_path = get_aapt2().expect("aapt2 not found");
    let output = std::process::Command::new(aapt2_path)
        .args(&["dump", "badging", &path.to_str().unwrap()])
        .output()
        .expect("failed to execute aapt2");

    let output = String::from_utf8(output.stdout).expect("failed to convert output to string");

    let mut package_name = String::new();
    let mut version = String::new();
    let mut min_sdk = String::new();
    let mut target_sdk = String::new();
    let mut name = String::new();
    let mut is_32bit = false;
    let mut is_64bit = false;
    let mut icon_base64 = None;

    let mut icon_path = String::new();
    for line in output.lines() {
        if line.starts_with("package:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            package_name = parts.get(1).unwrap().split("=").nth(1).unwrap().replace("'", "");
            version = parts.get(3).unwrap().split("=").nth(1).unwrap().replace("'", "");
        } else if line.starts_with("sdkVersion:") {
            min_sdk = line.split(':').nth(1).unwrap().trim().replace("'", "");
        } else if line.starts_with("targetSdkVersion:") {
            target_sdk = line.split(':').nth(1).unwrap().trim().replace("'", "");
        } else if line.starts_with("application-label:") {
            name = line.split(':').nth(1).unwrap().trim().replace("'", "");
        } else if line.starts_with("application:") {
            if let Some(start) = line.find("icon='") {
                let end = line[start + 6..].find('\'').unwrap_or(line.len());
                icon_path = line[start + 6..start + 6 + end].to_string();
            }
        } else if line.starts_with("native-code:") {
            let native_code = line.split(':').nth(1).unwrap().trim().replace("'", "");
            let archs: Vec<&str> = native_code.split_whitespace().collect();
            is_32bit = archs.contains(&"armeabi-v7a");
            is_64bit = archs.contains(&"arm64-v8a");
        }
    }

    if !icon_path.is_empty() {
        if let Ok(file) = File::open(&path) {
            if let Ok(mut archive) = ZipArchive::new(file) {
                if let Ok(mut icon_file) = archive.by_name(&icon_path) {
                    let mut icon_data = Vec::new();
                    if icon_file.read_to_end(&mut icon_data).is_ok() {
                        icon_base64 = Some(general_purpose::STANDARD.encode(&icon_data));
                    }
                }
            }
        }
    }

    AppDetail {
        name,
        package_name,
        version,
        min_sdk,
        target_sdk,
        is_32bit,
        is_64bit,
        icon_base64,
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