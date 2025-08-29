use std::option::Option;
use std::ffi::CString;
use std::path::Path;
use hidapi::HidApi;
use udev::Device;

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
            input_device_path: get_keyd_event_path(),
            hidraw_device_path: get_hidraw_path(),
        }
    }

    let info = HidDeviceInfo {
        hid_id: parse_hid_id(asus_bus_path.clone()),
        input_device_path: get_normal_event_path(asus_bus_path.clone()),
        hidraw_device_path: get_hidraw_path(),
    };

    info // return
}

fn get_normal_event_path(bus_path: String) -> String {
    let hid_dev = udev::Device::from_syspath(Path::new(&bus_path))
        .expect("Failed to create udev enumerator");

    let mut enumerator = udev::Enumerator::new().expect("Failed to create udev enumerator");

    enumerator.match_subsystem("input").unwrap();
    enumerator.match_property("ID_VENDOR_ID", "0b05").unwrap();
    enumerator.match_property("ID_MODEL_ID", "19b6").unwrap();
    enumerator.match_parent(&hid_dev).unwrap();

    for device in enumerator.scan_devices().expect("Failed to scan devices") {
        let found = device.properties().find(|p| p.name() == "DEVNAME").map(|p| {
            return p.value().to_owned();
        });
        if !found.is_none() {
            return found.unwrap().into_string().unwrap();
        }
    }

    panic!("No matching event device found");
}


fn get_keyd_event_path() -> String {
    let mut enumerator = udev::Enumerator::new().expect("Failed to create udev enumerator");

    enumerator.match_subsystem("input").unwrap();
    enumerator.match_attribute("name", "keyd virtual keyboard").unwrap();

    let mut maybe_parent: Option<Device> = None;
    for device in enumerator.scan_devices().expect("Failed to scan devices") {
        let found = device.properties().find(|p| p.name() == "DEVPATH").map(|p| {
            return p.value().to_owned();
        });
        if !found.is_none() {
            maybe_parent = Some(device);
            break;
        }
    }

    if !Option::is_some(&maybe_parent) {
        panic!("No matching keyd parent device found");
    }
    let parent = maybe_parent.unwrap();

    let mut enumerator = udev::Enumerator::new().expect("Failed to create udev enumerator");
    enumerator.match_subsystem("input").unwrap();
    enumerator.match_parent(&parent).unwrap();
    for device in enumerator.scan_devices().expect("Failed to scan devices") {
        let found = device.properties().find(|p| p.name() == "DEVNAME").map(|p| {
            println!("{}", p.value().to_str().unwrap());
            return p.value().to_owned();
        });
        if !found.is_none() {
            return found.unwrap().into_string().unwrap();
        }
    }

    panic!("No matching keyd event device found");
}

fn get_hidraw_path() -> String{
    let mut hid = HidApi::new().expect("HidApi::new failed");
    hid.reset_devices().expect("HidApi::new failed");
    hid.add_devices(0x0b05, 0x19b6).expect("Failed to add devices");
    for device in hid.device_list() {
        if device.usage() == 0x76 && device.usage_page() == 0xff31 {
            println!("Found target device!");
            return device.path().to_str().unwrap().to_string();
        }
    }
    panic!("No matching HID device found");
}