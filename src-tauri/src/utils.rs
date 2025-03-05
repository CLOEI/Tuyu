use std::{fs::File, io::{BufRead, BufReader, Read}, path::Path, process::{Command, Stdio}};

use base64::{engine::general_purpose, Engine};
use quick_xml::{events::Event, name::QName, Reader};
use tauri::{AppHandle, Emitter};
use which::which_in;
use zip::ZipArchive;

#[derive(Debug, serde::Serialize, Default)]
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

pub fn get_aapt2() -> Option<String> {
    which_in("aapt2", Some("binaries"), std::env::current_dir().unwrap()).ok().map(|path| path.to_string_lossy().to_string())
}

pub fn get_app_detail_from_xapk(app_path: String) -> Option<AppDetail> {
    let path = Path::new(&app_path);
    let mut app_detail = AppDetail::default();

    if let Ok(file) = File::open(&path) {
        if let Ok(mut archive) = ZipArchive::new(file) {
            let mut icon_path = String::new();
            if let Ok(mut manifest_file) = archive.by_name("manifest.json") {
                let mut manifest_data = String::new();
                if manifest_file.read_to_string(&mut manifest_data).is_ok() {
                    let manifest: serde_json::Value = serde_json::from_str(&manifest_data).expect("failed to parse manifest.json");
                    app_detail.name = manifest["name"].as_str().expect("failed to get app name").to_string();
                    app_detail.package_name = manifest["package_name"].as_str().expect("failed to get package name").to_string();
                    app_detail.version = manifest["version_name"].as_str().expect("failed to get version code").to_string();
                    app_detail.min_sdk = manifest["min_sdk_version"].as_str().expect("failed to get min sdk version").to_string();
                    app_detail.target_sdk = manifest["target_sdk_version"].as_str().expect("failed to get target sdk version").to_string();
                    icon_path = manifest["icon"].as_str().expect("failed to get icon path").to_string();
                    app_detail.is_32bit = manifest["split_configs"].as_array().expect("failed to get split configs").iter().any(|config| config.as_str().expect("failed to get config") == "config.armeabi_v7a");
                    app_detail.is_64bit = manifest["split_configs"].as_array().expect("failed to get split configs").iter().any(|config| config.as_str().expect("failed to get config") == "config.arm64_v8a");
                }
            }
            if let Ok(mut icon_file) = archive.by_name(&icon_path) {
                let mut icon_data = Vec::new();
                if icon_file.read_to_end(&mut icon_data).is_ok() {
                    app_detail.icon_base64 = Some(general_purpose::STANDARD.encode(&icon_data));
                }
            }
        }
    }

    Some(app_detail)
}

pub fn get_app_detail_from_apk(app_path: String) -> Option<AppDetail> {
    let path = Path::new(&app_path);

    let aapt2_path = get_aapt2().expect("aapt2 not found");
    let output = std::process::Command::new(aapt2_path)
        .args(&["dump", "badging", &path.to_str().unwrap()])
        .output()
        .expect("failed to execute aapt2");

    let output = String::from_utf8(output.stdout).expect("failed to convert output to string");
    let mut app_detail = AppDetail::default();
    let mut icon_path = String::new();

    for line in output.lines() {
        if line.starts_with("package:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            app_detail.package_name = parts.get(1).unwrap().split("=").nth(1).unwrap().replace("'", "");
            app_detail.version = parts.get(3).unwrap().split("=").nth(1).unwrap().replace("'", "");
        } else if line.starts_with("sdkVersion:") {
            app_detail.min_sdk = line.split(':').nth(1).unwrap().trim().replace("'", "");
        } else if line.starts_with("targetSdkVersion:") {
            app_detail.target_sdk = line.split(':').nth(1).unwrap().trim().replace("'", "");
        } else if line.starts_with("application-label:") {
            app_detail.name = line.split(':').nth(1).unwrap().trim().replace("'", "");
        } else if line.starts_with("application:") {
            if let Some(start) = line.find("icon='") {
                let end = line[start + 6..].find('\'').unwrap_or(line.len());
                icon_path = line[start + 6..start + 6 + end].to_string();
            }
        } else if line.starts_with("native-code:") {
            let native_code = line.split(':').nth(1).unwrap().trim().replace("'", "");
            let archs: Vec<&str> = native_code.split_whitespace().collect();
            app_detail.is_32bit = archs.contains(&"armeabi-v7a");
            app_detail.is_64bit = archs.contains(&"arm64-v8a");
        }
    }

    if !icon_path.is_empty() {
        if let Ok(file) = File::open(&path) {
            if let Ok(mut archive) = ZipArchive::new(file) {
                if let Ok(mut icon_file) = archive.by_name(&icon_path) {
                    let mut icon_data = Vec::new();
                    if icon_file.read_to_end(&mut icon_data).is_ok() {
                        app_detail.icon_base64 = Some(general_purpose::STANDARD.encode(&icon_data));
                    }
                }
            }
        }
    }

    Some(app_detail)
}

pub fn get_app_detail_from_dir(app_path: String) -> Option<AppDetail> {
    let manifest = Path::new(&app_path).join("AndroidManifest.xml");
    let apktool_yml = Path::new(&app_path).join("apktool.yml");
    let res_path = Path::new(&app_path).join("res");
    
    if !manifest.exists() || !apktool_yml.exists() || !res_path.exists() {
        return None;
    }
    let mut reader = Reader::from_file(manifest).expect("failed to read manifest");
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut app_detail = AppDetail::default();
    let deserialized_apktool_yml: serde_yaml::Value = serde_yaml::from_str(&std::fs::read_to_string(apktool_yml).expect("failed to read apktool.yml")).expect("failed to deserialize apktool.yml");
    let version_code = deserialized_apktool_yml["versionInfo"]["versionCode"].as_i64().expect("failed to get version code");
    let min_sdk_version = deserialized_apktool_yml["sdkInfo"]["minSdkVersion"].as_i64().expect("failed to get min sdk version");
    let target_sdk_version = deserialized_apktool_yml["sdkInfo"]["targetSdkVersion"].as_i64().expect("failed to get target sdk version");

    app_detail.version = version_code.to_string();
    app_detail.min_sdk = min_sdk_version.to_string();
    app_detail.target_sdk = target_sdk_version.to_string();

    let mut label_reference: Option<String> = None;
    let mut icon_reference: Option<String> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => panic!("failed to read event: {}", e),
            Ok(quick_xml::events::Event::Eof) => break,
            Ok(Event::Start(e)) => {
               match e.name().as_ref() {
                b"manifest" => {
                    for attr in e.attributes() {
                        let attr = attr.expect("failed to get attribute");
                        if attr.key == QName(b"package") {
                            app_detail.package_name = String::from_utf8(attr.value.to_vec()).expect("failed to convert package name to string");
                        }
                    }
                },
                b"application" => {
                    for attr in e.attributes() {
                        let attr = attr.expect("failed to get attribute");
                        if attr.key == QName(b"android:label") {
                            let label_value = String::from_utf8(attr.value.to_vec()).expect("failed to convert app name to string");
                            if label_value.starts_with("@string/") {
                                label_reference = Some(label_value.trim_start_matches("@string/").to_string());
                            } else {
                                app_detail.name = label_value;
                            }
                        }
                        if attr.key == QName(b"android:icon") {
                            icon_reference = Some(String::from_utf8(attr.value.to_vec()).expect("failed to convert icon reference to string"));
                        }
                    }
                }
                _ => (),
               }
            }
            _ => (),
        }
    }

    if let Some(ref key) = label_reference {
        let strings_xml_path = res_path.join("values/strings.xml");
        if let Ok(file) = std::fs::read_to_string(&strings_xml_path) {
            let mut reader = Reader::from_str(&file);
            reader.config_mut().trim_text(true);
            let mut buf = Vec::new();
            while let Ok(event) = reader.read_event_into(&mut buf) {
                match event {
                    Event::Start(ref e) if e.name().as_ref() == b"string" => {
                        if let Some(attr) = e.attributes().find(|a| a.as_ref().ok().map(|a| a.key.as_ref() == b"name").unwrap_or(false)) {
                            if let Ok(attr) = attr {
                                if let Ok(name) = attr.unescape_value() {
                                    if name == *key {
                                        if let Ok(Event::Text(text)) = reader.read_event_into(&mut buf) {
                                            app_detail.name = text.unescape().unwrap_or_default().to_string();
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Event::Eof => break,
                    _ => (),
                }
                buf.clear();
            }
        }
    }

    if let Some(ref key) = icon_reference {
        let icon_dirs = ["mipmap", "drawable"];
        
        for folder in icon_dirs.iter() {
            if let Ok(entries) = std::fs::read_dir(&res_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        let dir_name = path.file_name().unwrap().to_string_lossy();
                        if dir_name.starts_with(folder) {
                            let icon_path = path.join(format!("{}.png", key.trim_start_matches("@mipmap/").trim_start_matches("@drawable/")));
                            if icon_path.exists() {
                                if let Ok(mut file) = File::open(&icon_path) {
                                    let mut data = Vec::new();
                                    if file.read_to_end(&mut data).is_ok() {
                                        app_detail.icon_base64 = Some(general_purpose::STANDARD.encode(&data));
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }    

    let lib_path = Path::new(&app_path).join("lib");
    if lib_path.exists() {
        if let Ok(entries) = std::fs::read_dir(&lib_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let dir_name = path.file_name().unwrap().to_string_lossy();
                    if dir_name.starts_with("armeabi-v7a") {
                        app_detail.is_32bit = true;
                    } else if dir_name.starts_with("arm64-v8a") {
                        app_detail.is_64bit = true;
                    }
                }
            }
        }
    }

    Some(app_detail)
}

pub fn run_apktool_command(handle: AppHandle, args: &[&str], success_msg: String, error_msg: String) {
    let apktool_path = Path::new("binaries/apktool.jar");
    if !apktool_path.exists() {
        handle.emit("log-error", "apktool.jar not found").unwrap();
        return;
    }

    let mut cmd = Command::new("java")
        .args(&["-jar", apktool_path.to_str().unwrap()])
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start Apktool");

    let stdout = cmd.stdout.take().expect("Failed to get stdout");
    let stderr = cmd.stderr.take().expect("Failed to get stderr");

    let handle_clone = handle.clone();
    std::thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines().flatten() {
            handle_clone.emit("log-info", line.trim()).unwrap();
        }
    });

    let handle_clone = handle.clone();
    std::thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines().flatten() {
            handle_clone.emit("log-error", line.trim()).unwrap();
        }
    });

    std::thread::spawn(move || {
        let status = cmd.wait().expect("Failed to wait on child process");
        if status.success() {
            handle.emit("log-info", success_msg).unwrap();
        } else {
            handle.emit("log-error", error_msg).unwrap();
        }
    });
}