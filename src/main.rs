mod apkt_config;
mod bpf_loader;
mod hid;
mod kb_illumination;
mod state;

use evdev::{Device, KeyCode};
use crate::apkt_config::get_config;
use crate::bpf_loader::start_bpf;
use crate::hid::{get_hardware_info, toggle_fn_lock};
use crate::state::{load_state, save_state};
use evdev_rs::enums::{EV_KEY};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // read args to get config path
     let args: Vec<String> = std::env::args().collect();
    let mut config_path = "asus-px-keyboard-tool.conf";
    if args.len() >= 2 {
        // allow user to specify config path as first arg
        config_path = &args[1];
    }
    println!("Using config path: {}", config_path);
    if args.len() == 3 {
        if args[2] == "restore" {
            // restore mode after sleep wakeup
            restore(config_path);
        } else {
            println!("Invalid argument: {}", args[2]);
            println!("Usage: {} [config_path] [restore]", args[0]);
        }
        return Err("Invalid argument".into());
    }
    if args.len() > 3 {
        println!("Too many arguments");
        println!("Usage: {} [config_path] [restore]", args[0]);
        return Err("Too many arguments".into());
    }

    let config = get_config(config_path);

    // convert string to enum
    let mut target_keycodes: Vec<KeyCode> = Vec::new();
    let mut illum_keycode: Option<KeyCode> = None;
    let mut fnlock_keycode: Option<KeyCode> = None;

    if config.kb_brightness_cycle.enabled {
        let ev_key = EV_KEY::from(config.kb_brightness_cycle.keycode.parse()
            .expect("Invalid kb_brightness keycode in config"));
        let key_code = KeyCode::new(ev_key as u16);
        illum_keycode = Some(key_code);
        target_keycodes.push(key_code);
    }

    if config.fnlock.enabled {
        let ev_key = EV_KEY::from(config.fnlock.keycode.parse()
            .expect("Invalid fnlock keycode in config"));
        let key_code = KeyCode::new(ev_key as u16);
        fnlock_keycode = Some(key_code);
        target_keycodes.push(key_code);
    }

    let mut dev_info = get_hardware_info(target_keycodes);
    if config.compatibility.hid_id_override.is_some() {
        println!(
            "Overriding HID ID from {} to {}",
            dev_info.hid_id,
            config.compatibility.hid_id_override.unwrap()
        );
        dev_info.hid_id = config.compatibility.hid_id_override.unwrap();
    }
    if config.compatibility.hid_path_override.is_some() {
        println!(
            "Overriding HID path from {} to {}",
            dev_info.hidraw_device_path,
            config.compatibility.hid_path_override.as_ref().unwrap()
        );
        dev_info.hidraw_device_path = config.compatibility.hid_path_override.unwrap();
    }
    if config.compatibility.event_path_override.is_some() {
        println!(
            "Overriding event path to {}",
            config.compatibility.event_path_override.as_ref().unwrap()
        );
        dev_info.possible_event_paths = vec![config.compatibility.event_path_override.unwrap()];

    }
    println!("HID ID: {}", dev_info.hid_id);
    println!("Possible event devices: {:?}", dev_info.possible_event_paths);
    println!("HIDRAW device: {}", dev_info.hidraw_device_path);
    if config.bpf.enabled {
        println!("Starting BPF with remaps: {:?}", config.bpf.remaps);
        start_bpf(dev_info.hid_id as i32, config.bpf.remaps);
    } else {
        println!("BPF disabled in config");
    }

    let mut state = false;

    if config.fnlock.enabled {
        // apply initial fnlock state
        if config.fnlock.boot_default == "on" {
            state = true;
        } else if config.fnlock.boot_default == "off" {
            state = false;
        } else if config.fnlock.boot_default == "last" {
            state = load_state();
        } else {
            panic!(
                "Invalid fnlock.boot_default value in config: {}",
                config.fnlock.boot_default
            );
        }
        toggle_fn_lock(&dev_info.hidraw_device_path, state);
        save_state(state);
    }

    for path in dev_info.possible_event_paths {
        let str_pointer = dev_info.hidraw_device_path.clone();
        tokio::spawn(async move {
            println!("Opening event device: {}", path);
            let device = Device::open(&path)
                .expect("Failed to open input device");

            let mut stream = device.into_event_stream()
                .expect("Failed to create event stream");

            loop {
                let event = stream.next_event().await;
                if let Ok(ev) = event {
                    if ev.event_type() == evdev::EventType::KEY {
                        // check for kb_illum_toggle keycode
                        if config.kb_brightness_cycle.enabled
                            && ev.code() == illum_keycode.unwrap().code()
                            && ev.value() == 1
                        {
                            println!("kb brightness event");
                            kb_illumination::cycle();
                        }

                        // check for fnlock
                        if config.fnlock.enabled
                            && ev.code() == fnlock_keycode.unwrap().code()
                            && ev.value() == 1
                        {
                            println!("Fn key event");
                            state = !state;
                            toggle_fn_lock(&str_pointer, state);
                            save_state(state);
                        }
                    }
                }
            }
        });
    }

    // Keep the main task alive
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    }
}

fn restore(config_path: &str) {
    let config = get_config(config_path);
    if !config.fnlock.enabled {
        println!("FnLock not enabled in config, nothing to restore");
        return;
    }
    let state = load_state();
    let dev_info = hid::get_hardware_info(vec![]);
    toggle_fn_lock(&dev_info.hidraw_device_path, state);
    println!(
        "Restored FnLock state to: {}",
        if state { "on" } else { "off" }
    );
}
