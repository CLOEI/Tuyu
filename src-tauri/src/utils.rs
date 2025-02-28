use which::which_in;

pub fn get_aapt2() -> Option<String> {
    which_in("aapt2", Some("binaries"), std::env::current_dir().unwrap()).ok().map(|path| path.to_string_lossy().to_string())
}