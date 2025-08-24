pub fn save_state(state: bool) {
    let filename = "/var/lib/asuspx/state";
    // save bool as either 1 or 0 in the file
    std::fs::write(filename, if state { "1" } else { "0" })
        .expect("Unable to write file");
}

pub fn load_state() -> bool {
    // load the state from file.  if the file is invalid or missing, use default false
    let filename = "/var/lib/asuspx/state";
    let contents = std::fs::read_to_string(filename)
        .unwrap_or_else(|_| "0".to_string());
    contents.trim() == "1"
}