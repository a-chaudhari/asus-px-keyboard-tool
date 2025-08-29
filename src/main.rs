mod apkt_config;
mod bpf_loader;
mod hid;
mod kb_illumination;
mod state;

use crate::apkt_config::get_config;
use crate::bpf_loader::start_bpf;
use crate::hid::{get_hardware_info, toggle_fn_lock};
use crate::kb_illumination::cycle;
use crate::state::{load_state, save_state};
use evdev_rs::Device;
use evdev_rs::ReadFlag;
use evdev_rs::enums::{EventCode, EventType};

fn main() {
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
        return;
    }
    if args.len() > 3 {
        println!("Too many arguments");
        println!("Usage: {} [config_path] [restore]", args[0]);
        return;
    }

    let config = get_config(config_path);
    let mut dev_info = get_hardware_info(config.compatibility.keyd);
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
            "Overriding event path from {} to {}",
            dev_info.input_device_path,
            config.compatibility.event_path_override.as_ref().unwrap()
        );
        dev_info.input_device_path = config.compatibility.event_path_override.unwrap();
    }
    println!("HID ID: {}", dev_info.hid_id);
    println!("Using device: {}", dev_info.input_device_path);
    println!("HIDRAW device: {}", dev_info.hidraw_device_path);
    if config.bpf.enabled {
        println!("Starting BPF with remaps: {:?}", config.bpf.remaps);
        start_bpf(dev_info.hid_id as i32, config.bpf.remaps);
    } else {
        println!("BPF disabled in config");
    }
    // open file as blocking to save cpu cycles
    let file = std::fs::File::open(&dev_info.input_device_path).expect(&format!(
        "Failed to open input device: {}",
        &dev_info.input_device_path
    ));
    let input_device = Device::new_from_file(file).unwrap();
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

    // convert string to enum
    let mut illum_keycode: Option<EventCode> = None;
    let mut fnlock_keycode: Option<EventCode> = None;

    if config.kb_brightness_cycle.enabled {
        illum_keycode = Some(
            EventCode::from_str(&EventType::EV_KEY, &config.kb_brightness_cycle.keycode)
                .expect("Invalid kb_brightness keycode in config"),
        );
    }

    if config.fnlock.enabled {
        fnlock_keycode = Some(
            EventCode::from_str(&EventType::EV_KEY, &config.fnlock.keycode)
                .expect("Invalid fnlock keycode in config"),
        );
    }

    loop {
        let ev = input_device.next_event(ReadFlag::BLOCKING).map(|val| val.1);
        match ev {
            Ok(ev) => {
                // check for kb_illum_toggle keycode
                if config.kb_brightness_cycle.enabled
                    && ev.event_code == illum_keycode.unwrap()
                    && ev.value == 1
                {
                    println!("kb brightness event");
                    cycle();
                }

                // check for fnlock
                if config.fnlock.enabled
                    && ev.event_code == fnlock_keycode.unwrap()
                    && ev.value == 1
                {
                    println!("Fn key event");
                    state = !state;
                    toggle_fn_lock(&dev_info.hidraw_device_path, state);
                    save_state(state);
                }
            }
            Err(_) => (),
        }
    }
}

fn restore(config_path: &str) {
    let config = get_config(config_path);
    if !config.fnlock.enabled {
        println!("FnLock not enabled in config, nothing to restore");
        return;
    }
    let state = load_state();
    let dev_info = hid::get_hardware_info(config.compatibility.keyd);
    toggle_fn_lock(&dev_info.hidraw_device_path, state);
    println!(
        "Restored FnLock state to: {}",
        if state { "on" } else { "off" }
    );
}
