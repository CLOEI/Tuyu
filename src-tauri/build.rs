use std::{env, fs, path::{Path, PathBuf}};

fn copy_files(src: &Path, dest: &Path) {
    if !dest.exists() {
        fs::create_dir_all(dest).unwrap();
    }

    for entry in fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_file() {
            fs::copy(&path, dest.join(entry.file_name())).unwrap();
        }
    }
}

fn copy_dir_recursive(src: &Path, dest: &Path) {
    if !dest.exists() {
        fs::create_dir_all(dest).unwrap();
    }

    for entry in fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let dest_path = dest.join(entry.file_name());

        #[cfg(target_arch = "arm")]
        if path.is_dir() && path.ends_with("arm") {
            copy_files(&path, &dest_path);
        } else if path.is_file() {
            fs::copy(&path, &dest_path).unwrap();
        }

        #[cfg(target_arch = "x86_64")]
        if path.is_dir() && path.ends_with("x86_64") {
            copy_files(&path, &dest_path);
        } else if path.is_file() {
            fs::copy(&path, &dest_path).unwrap();
        }
    }
}


fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let dest = PathBuf::from("binaries");
    if dest.exists() {
        fs::remove_dir_all(&dest).unwrap();
    }
    fs::create_dir_all(&dest).unwrap();

    let source_dir = PathBuf::from("../binaries");
    let os_specific_dir = source_dir.join(&target_os);
    copy_files(&source_dir, &dest);

    if os_specific_dir.exists() {
        copy_dir_recursive(&os_specific_dir, &dest);
    }

    tauri_build::build()
}
