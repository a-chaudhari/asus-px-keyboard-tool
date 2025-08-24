use std::ffi::CString;
use hidapi::HidApi;

pub fn toggle_fn_lock(hid_path: String, new_state: bool) {
    let c_string = CString::new(hid_path).expect("CString::new failed");
    let c_str = c_string.as_c_str();

    // Open the HID device at the specified path
    let device = hidapi::HidApi::new()
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

pub fn get_hidraw_path(hid_api: HidApi) -> String {
    // Iterate over all connected devices
    for device in hid_api.device_list() {
        if device.vendor_id() != 0xb05 ||
            device.product_id() != 0x19b6 ||
            device.usage_page() != 0xff31 ||
            device.usage() != 0x76 {
            continue; // Skip devices that are not the target
        }
        return device.path().to_str()
            .expect("Failed to convert HID path to string")
            .to_string();
    }
    // If no device is found, raise an error
    panic!("No device found with the specified criteria.");
}