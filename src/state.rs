static FILE_ROOT: &str = "/var/lib/asus-px-kb-tool";

pub fn save_state(state: bool) {
    let filename = format!("{}/state", FILE_ROOT);
    // save bool as either 1 or 0 in the file
    // create the directory if it doesn't exist
    std::fs::create_dir_all(FILE_ROOT)
        .expect("Unable to create directory");
    std::fs::write(filename, if state { "1" } else { "0" })
        .expect("Unable to write file");
}

pub fn load_state() -> bool {
    // load the state from file.  if the file is invalid or missing, use default false
    let filename = format!("{}/state", FILE_ROOT);
    let contents = std::fs::read_to_string(filename)
        .unwrap_or_else(|_| "0".to_string());
    contents.trim() == "1"
}