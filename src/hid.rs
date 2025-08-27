use std::ffi::CString;
use hidapi::HidApi;

pub struct HidDeviceInfo {
    pub hid_id: u32,
    pub input_device_path: String,
    pub hidraw_device_path: String,
}

pub fn toggle_fn_lock(hid_path: &String, new_state: bool) {
    let c_string = CString::new(hid_path.clone()).expect("CString::new failed");
    let c_str = c_string.as_c_str();

    // Open the HID device at the specified path
    let device = HidApi::new()
        .expect("HidApi::new failed");
    let handle = device.open_path(c_str)
        .expect("Failed to open HID device");

    // Create a feature report to send
    let mut feature_report: [u8; 63] = [
        0x5a, 0xd0, 0x4e, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];

    feature_report[3] = if new_state { 0 } else { 1 };

    // Send the feature report
    match handle.send_feature_report(&feature_report) {
        Ok(_) => println!("Fn-Lock command sent successfully!"),
        Err(e) => eprintln!("Error sending feature report: {}", e),
    }
}

fn get_hidraw_path(bus_path: String) -> String {
    let hidraw_dir_path = format!("{}/hidraw", bus_path);
    let hidraw_entries = std::fs::read_dir(hidraw_dir_path)
        .expect("Failed to read hidraw directory");
    for hidraw_entry in hidraw_entries {
        let hidraw_entry = hidraw_entry.expect("Failed to read hidraw entry");
        let hidraw_file_name = hidraw_entry.file_name();
        let hidraw_file_name_str = hidraw_file_name.to_str()
            .expect("Failed to convert hidraw file name to string");
        if hidraw_file_name_str.starts_with("hidraw") {
            let hidraw_full_path = format!("/dev/{}", hidraw_file_name_str);
            return hidraw_full_path;
        }
    }
    panic!("No matching hidraw device found");
}

fn get_input_device_path(bus_path: String) -> String {
    let input_dir_path = format!("{}/input", bus_path);
    let input_entries = std::fs::read_dir(input_dir_path)
        .expect("Failed to read input directory");
    for input_entry in input_entries {
        let input_entry = input_entry.expect("Failed to read input entry");
        let input_file_name = input_entry.file_name();
        let input_file_name_str = input_file_name.to_str()
            .expect("Failed to convert input file name to string");
        if input_file_name_str.starts_with("input") {
            // now get the event path in that folder
            let event_dir_path = format!("{}", input_entry.path().to_str().unwrap());
            let event_entries = std::fs::read_dir(event_dir_path)
                .expect("Failed to read event directory");
            for event_entry in event_entries {
                let event_entry = event_entry.expect("Failed to read event entry");
                let event_file_name = event_entry.file_name();
                let event_file_name_str = event_file_name.to_str()
                    .expect("Failed to convert event file name to string");
                if event_file_name_str.starts_with("event") {
                    let event_full_path = format!("/dev/input/{}", event_file_name_str);
                    return event_full_path;
                }
            }
        }
    }
    panic!("No matching input device found");
}

fn get_keyd_input_path(vid: &str, pid: &str) -> String {
    // loop through /sys/devices/virtual/input and check the pid and vid
    // then use that to find the event path
    let syspath = "/sys/devices/virtual/input";
    let entries = std::fs::read_dir(syspath)
        .expect("Failed to read /sys/devices/virtual/input directory");
    for entry in entries {
        let entry = entry.expect("Failed to read entry");
        let file_name = entry.file_name();
        let file_name_str = file_name.to_str()
            .expect("Failed to convert file name to string");
        let full_path = format!("{}/{}", syspath, file_name_str);
        let uevent_path = format!("{}/uevent", full_path);
        let uevent_content = std::fs::read_to_string(uevent_path)
            .expect("Failed to read uevent file");
        let mut found_vid = false;
        let mut found_pid = false;
        for line in uevent_content.lines() {
            if line.starts_with("PRODUCT=") {
                let parts: Vec<&str> = line["PRODUCT=".len()..].split('/').collect();
                if parts.len() >= 2 {
                    if parts[1].eq_ignore_ascii_case(vid) {
                        found_vid = true;
                    }
                    if parts[2].eq_ignore_ascii_case(pid) {
                        found_pid = true;
                    }

                    if found_vid && found_pid {
                        break;
                    }
                }
            }
        }
        if found_vid && found_pid {
            // now find the event path in that folder
            let event_entries = std::fs::read_dir(full_path)
                .expect("Failed to read event directory");
            for event_entry in event_entries {
                let event_entry = event_entry.expect("Failed to read event entry");
                let event_file_name = event_entry.file_name();
                let event_file_name_str = event_file_name.to_str()
                    .expect("Failed to convert event file name to string");
                if event_file_name_str.starts_with("event") {
                    let event_full_path = format!("/dev/input/{}", event_file_name_str);
                    return event_full_path;
                }
            }
        }
    }
    panic!("No matching virtual input device found");
}

fn get_bus_path(vid_pid: &str, descriptor_check: bool) -> String {
    let syspath = "/sys/bus/hid/devices";
    // read in directory to find the target
    let entries = std::fs::read_dir(syspath)
        .expect("Failed to read /sys/bus/hid/devices directory");

    for entry in entries {
        let entry = entry.expect("Failed to read entry");
        let file_name = entry.file_name();
        let file_name_str = file_name.to_str()
            .expect("Failed to convert file name to string");
        if file_name_str.contains(vid_pid) {
            let full_path = format!("{}/{}", syspath, file_name_str);
            // now check the report_descriptor file
            if !descriptor_check {
                return full_path;
            }
            let report_descriptor_path = format!("{}/report_descriptor", full_path);
            let report_descriptor = std::fs::read(report_descriptor_path)
                .expect("Failed to read report_descriptor file");
            // let expected_descriptor: [u32; 9] = [0x06, 0x31, 0xff, 0x09, 0x76, 0xa1, 0x01, 0x85, 0x5a];
            let exp_bytes = [0x06, 0x31, 0xff, 0x09, 0x76, 0xa1, 0x01, 0x85, 0x5a].to_vec();
            // check if report_descriptor contains expected_descriptor bytes at any position
            let found = report_descriptor
                .windows(exp_bytes.len())
                .any(|window| window == exp_bytes.as_slice());
            if found {
                return full_path;
            }
        }
    }
    panic!("No matching HID device found");
}

fn parse_hid_id(bus_path: String) -> u32 {
    let parts: Vec<&str> = bus_path.split(':').collect();
    let last_part = parts.last().expect("Failed to get last part of bus path");
    let hid_id_str = &last_part[last_part.len() - 1..];
    let hid_id = hid_id_str.parse::<u32>()
        .expect("Failed to parse HID ID");
    hid_id
}

pub fn get_hardware_info(keyd_mode: bool) -> HidDeviceInfo{
    let asus_ids = "0B05:19B6";
    let asus_bus_path = get_bus_path(asus_ids, true);

    if keyd_mode {
        return HidDeviceInfo {
            hid_id: parse_hid_id(asus_bus_path.clone()),
            input_device_path: get_keyd_input_path("fac", "ade"),
            hidraw_device_path: get_hidraw_path(asus_bus_path.clone()),
        }
    }

    HidDeviceInfo {
        hid_id: parse_hid_id(asus_bus_path.clone()),
        input_device_path: get_input_device_path(asus_bus_path.clone()),
        hidraw_device_path: get_hidraw_path(asus_bus_path.clone()),
    }
}