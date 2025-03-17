use std::{fs::File, io::{BufRead, BufReader, Read}, path::Path, process::{Command, Stdio}};

use base64::{engine::general_purpose, Engine};
use tauri::{AppHandle, Emitter};
use which::{which, which_in};
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

#[derive(Debug, serde::Serialize)]
pub struct Directory {
    pub name: String,
    pub r#type: u16, // 0 = file, 1 = folder, 2 = folder symlink, 3 = file symlink
    pub link_to: String,
}

pub fn get_aapt2() -> Option<String> {
    which_in("aapt2", Some("binaries"), std::env::current_dir().unwrap()).ok().map(|path| path.to_string_lossy().to_string())
}

pub fn get_adb() -> Option<String> {
    which_in("adb", Some("binaries"), std::env::current_dir().unwrap()).ok().map(|path| path.to_string_lossy().to_string())
}

pub fn get_scrcpy() -> Option<String> {
    match which("scrcpy") {
        Ok(path) => Some(path.to_string_lossy().to_string()),
        Err(_) => which_in("scrcpy", Some("binaries"), std::env::current_dir().unwrap()).ok().map(|path| path.to_string_lossy().to_string())
    }
}

pub fn get_app_detail_from_xapk(app_path: String) -> Option<AppDetail> {
    let file = File::open(&app_path).ok()?;
    let mut archive = ZipArchive::new(file).ok()?;
    
    let manifest_data = archive.by_name("manifest.json").ok()?.bytes().collect::<Result<Vec<_>, _>>().ok()?;
    let manifest: serde_json::Value = serde_json::from_slice(&manifest_data).ok()?;
    
    let mut app_detail = AppDetail {
        name: manifest["name"].as_str()?.to_string(),
        package_name: manifest["package_name"].as_str()?.to_string(),
        version: manifest["version_name"].as_str()?.to_string(),
        min_sdk: manifest["min_sdk_version"].as_str()?.to_string(),
        target_sdk: manifest["target_sdk_version"].as_str()?.to_string(),
        is_32bit: manifest["split_configs"].as_array()?.iter().any(|c| c.as_str().unwrap() == "config.armeabi_v7a"),
        is_64bit: manifest["split_configs"].as_array()?.iter().any(|c| c.as_str().unwrap() == "config.arm64_v8a"),
        ..Default::default()
    };

    if let Some(icon_path) = manifest["icon"].as_str() {
        if let Ok(mut icon_file) = archive.by_name(icon_path) {
            let mut icon_data = Vec::new();
            if icon_file.read_to_end(&mut icon_data).is_ok() {
                app_detail.icon_base64 = Some(general_purpose::STANDARD.encode(&icon_data));
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

    let apktool_data: serde_yaml::Value = serde_yaml::from_str(&std::fs::read_to_string(apktool_yml).ok()?).ok()?;
    let mut app_detail = AppDetail {
        version: apktool_data["versionInfo"]["versionName"].as_f64()?.to_string(),
        min_sdk: apktool_data["sdkInfo"]["minSdkVersion"].as_i64()?.to_string(),
        target_sdk: apktool_data["sdkInfo"]["targetSdkVersion"].as_i64()?.to_string(),
        ..Default::default()
    };
    
    let manifest_content = std::fs::read_to_string(&manifest).ok()?;
    let doc = roxmltree::Document::parse(&manifest_content).ok()?;

    app_detail.package_name = doc.descendants().find(|n| n.has_tag_name("manifest"))?.attribute("package")?.to_string();
    app_detail.name = doc.descendants().find(|n| n.has_tag_name("application"))?.attributes().find(|attr| attr.name().ends_with("label"))?.value().to_string();

    if app_detail.name.starts_with("@string/") {
        let string_name = app_detail.name.trim_start_matches("@string/");
        let strings_path = Path::new(&app_path).join("res/values/strings.xml");
        if strings_path.exists() {
            let strings_content = std::fs::read_to_string(&strings_path).ok()?;
            let strings_doc = roxmltree::Document::parse(&strings_content).ok()?;
            app_detail.name = strings_doc.descendants().find(|n| n.has_tag_name("string") && n.attribute("name") == Some(string_name))?.text().unwrap().to_string();
        }
    }
    
    if let Some(icon_reference) = doc.descendants().find(|n| n.has_tag_name("application"))?.attributes().find(|attr| attr.name().ends_with("icon")){
        let icon_name = icon_reference.value().trim_start_matches("@mipmap/").trim_start_matches("@drawable/");
        
        if let Ok(entries) = std::fs::read_dir(&res_path) {
            for entry in entries.flatten() {
                let folder_name = entry.file_name().to_string_lossy().to_string();
                if folder_name.starts_with("mipmap") || folder_name.starts_with("drawable") {
                    let icon_path = entry.path().join(format!("{}.png", icon_name));
                    if icon_path.exists() {
                        if let Ok(icon_data) = std::fs::read(&icon_path) {
                            app_detail.icon_base64 = Some(general_purpose::STANDARD.encode(&icon_data));
                            break;
                        }
                    }
                }
            }
        }
    }
    
    let lib_path = Path::new(&app_path).join("lib");
    if let Ok(entries) = std::fs::read_dir(&lib_path) {
        for entry in entries.flatten() {
            if let Some(dir_name) = entry.file_name().to_str() {
                app_detail.is_32bit |= dir_name.starts_with("armeabi-v7a");
                app_detail.is_64bit |= dir_name.starts_with("arm64-v8a");
            }
        }
    }
    
    Some(app_detail)
}

pub fn run_java_tool(
    handle: AppHandle,
    tool_name: &str,
    args: &[&str],
    success_msg: String,
    error_msg: String,
) {
    let tool_path = format!("binaries/{}.jar", tool_name);
    let tool_path = Path::new(&tool_path);
    
    if !tool_path.exists() {
        handle.emit("log-error", format!("{}.jar not found", tool_name)).unwrap();
        return;
    }

    let mut cmd = Command::new("java")
        .args(&["-jar", tool_path.to_str().unwrap()])
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect(&format!("Failed to start {}", tool_name));

    let stdout = cmd.stdout.take().expect("Failed to get stdout");
    let stderr = cmd.stderr.take().expect("Failed to get stderr");

    let handle_clone = handle.clone();
    std::thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines().flatten() {
            handle_clone.emit("log", line.trim()).unwrap();
        }
    });

    let handle_clone = handle.clone();
    std::thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines().flatten() {
            handle_clone.emit("log", line.trim()).unwrap();
        }
    });

    std::thread::spawn(move || {
        let status = cmd.wait().expect("Failed to wait on child process");
        if status.success() {
            handle.emit("log", success_msg).unwrap();
        } else {
            handle.emit("log", error_msg).unwrap();
        }
    });
}

pub fn parse_ls_output(output: &str) -> Vec<Directory> {
    let mut directories = Vec::new();

    for line in output.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 2 {
            continue; // Skip total
        }
        let mut name = parts.last().unwrap().to_string();
        let link_to = parts.last().unwrap().to_string();
        let mut file_type = 0;

        if parts[0].starts_with("d") {
            file_type = 1;
        } else if parts[0].starts_with("l") {
            if parts[parts.len() - 2] == "->" {
                file_type = 2;
                name = parts[parts.len() - 3].to_string();
            } else {
                file_type = 3;
            }
        }
        
        directories.push(Directory { r#type: file_type, name, link_to });
    }

    directories
}