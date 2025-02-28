use std::{env, fs, path::PathBuf};

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let dest = PathBuf::from("binaries");
    if dest.exists() {
        fs::remove_dir_all(&dest).unwrap();
    }
    fs::create_dir_all(&dest).unwrap();

    for entry in fs::read_dir("../binaries").unwrap() {
        let entry = entry.unwrap();
        let is_dir = entry.file_type().unwrap().is_dir();
        if is_dir && entry.file_name().into_string().unwrap() == target_os {
            for entry in fs::read_dir(entry.path()).unwrap() {
                let entry = entry.unwrap();
                let path = entry.path();
                let file_name = entry.file_name();
                fs::copy(path, dest.join(file_name)).unwrap();
            }
        }
        if is_dir {
            continue;
        } else {
            let path = entry.path();
            let file_name = entry.file_name();
            fs::copy(path, dest.join(file_name)).unwrap();
        }
    }

    tauri_build::build()
}
