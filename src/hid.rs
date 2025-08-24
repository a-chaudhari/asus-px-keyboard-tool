use std::ffi::CString;
use hidapi::HidApi;

pub struct HidDeviceInfo {
    hid_id: u32,
    input_device_path: String,
    hidraw_device_path: String,
}

pub fn toggle_fn_lock(hid_path: String, new_state: bool) {
    let c_string = CString::new(hid_path).expect("CString::new failed");
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
        Ok(_) => println!("Feature report sent successfully!"),
        Err(e) => eprintln!("Error sending feature report: {}", e),
    }
}

pub fn get_hardware_info() -> HidDeviceInfo{
    let syspath = "/sys/bus/hid/devices";
    let target = "0B05:19B6";
    // read in directory to find the target
    let entries = std::fs::read_dir(syspath)
        .expect("Failed to read /sys/bus/hid/devices directory");
    let mut retval = HidDeviceInfo {
        hid_id: 0,
        input_device_path: String::new(),
        hidraw_device_path: String::new(),
    };

    for entry in entries {
        let entry = entry.expect("Failed to read entry");
        let file_name = entry.file_name();
        let file_name_str = file_name.to_str()
            .expect("Failed to convert file name to string");
        if file_name_str.contains(target) {
            let full_path = format!("{}/{}", syspath, file_name_str);
            // now check the report_descriptor file
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
                // get the last character of the filename and parse to u32
                let hid_id_str = &file_name_str[file_name_str.len() - 1..];
                let hid_id = hid_id_str.parse::<u32>()
                    .expect("Failed to parse HID ID");
                retval.hid_id = hid_id;

                // now find the event device path in input subdirectory
                let input_dir_path = format!("{}/input", full_path);
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
                                retval.input_device_path = event_full_path;
                                break;
                            }
                        }
                    }
                }
                // now find the hidraw device path in hidraw subdirectory
                let hidraw_dir_path = format!("{}/hidraw", full_path);
                let hidraw_entries = std::fs::read_dir(hidraw_dir_path)
                    .expect("Failed to read hidraw directory");
                for hidraw_entry in hidraw_entries {
                    let hidraw_entry = hidraw_entry.expect("Failed to read hidraw entry");
                    let hidraw_file_name = hidraw_entry.file_name();
                    let hidraw_file_name_str = hidraw_file_name.to_str()
                        .expect("Failed to convert hidraw file name to string");
                    if hidraw_file_name_str.starts_with("hidraw") {
                        let hidraw_full_path = format!("/dev/{}", hidraw_file_name_str);
                        retval.hidraw_device_path = hidraw_full_path;
                        break;
                    }
                }

                return retval;
            }
        }
    }
    panic!("No matching HID device found");
}