mod apkt_config;
mod bpf_loader;
mod hid;
mod kb_illumination;
mod state;

use std::collections::HashSet;
use std::sync::{Arc};
use tokio::sync::{Mutex, RwLock};
use evdev::{Device, EventType, KeyCode, SwitchCode};
use crate::apkt_config::{get_config, ConfigWrapper};
use crate::bpf_loader::start_bpf;
use crate::hid::{get_hardware_info, toggle_fn_lock, HidDeviceInfo};
use crate::state::{load_state, save_state};

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

    let config = &Arc::new(get_config(config_path));

    // convert string to enum
    let mut target_keycodes: Vec<KeyCode> = Vec::new();
    if config.fnlock.enabled && config.fnlock.keycode_enum.is_some(){
        target_keycodes.push(config.fnlock.keycode_enum.unwrap());
    }
    if config.kb_brightness_cycle.enabled && config.kb_brightness_cycle.keycode_enum.is_some(){
        target_keycodes.push(config.kb_brightness_cycle.keycode_enum.unwrap());
    }

    let mut dev_info = get_hardware_info(&target_keycodes);
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
        dev_info.hidraw_device_path = config.compatibility.hid_path_override.as_ref().unwrap().to_string();
    }
    if config.compatibility.event_path_override.is_some() {
        println!(
            "Overriding event path to {}",
            config.compatibility.event_path_override.as_ref().unwrap()
        );
        dev_info.possible_event_paths = vec![config.compatibility.event_path_override.as_ref().unwrap().to_string()];

    }
    println!("HID ID: {}", dev_info.hid_id);
    println!("Possible event devices: {:?}", dev_info.possible_event_paths);
    println!("HIDRAW device: {}", dev_info.hidraw_device_path);
    if config.bpf.enabled {
        println!("Starting BPF with remaps: {:?}", config.bpf.remaps);
        start_bpf(dev_info.hid_id as i32, config.bpf.remaps.as_ref());
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

    let active_paths: HashSet<String> = HashSet::new();
    let active_paths_mutex = &Arc::new(RwLock::new(active_paths));
    let dev_info_arc = &Arc::new(dev_info.clone());

    let state_mutex = &Arc::new(Mutex::new(state));

    for path in dev_info.possible_event_paths {
        let mut data = active_paths_mutex.write().await;
        data.insert(path.clone());
        start_device_thread(path.clone(), Arc::clone(config), Arc::clone(state_mutex), Arc::clone(dev_info_arc), Arc::clone(active_paths_mutex));
    }

    // Keep the main task alive
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        let mut data = active_paths_mutex.write().await;
        // println!("Active event device paths: {:?}", *data);

        // check for new paths
        let new_dev_info = get_hardware_info(&target_keycodes);
        for path in new_dev_info.possible_event_paths {
            if !data.contains(&path) {
                println!("New event device path detected: {}", path);
                data.insert(path.clone());
                start_device_thread(path.clone(), Arc::clone(config), Arc::clone(state_mutex),
                                    Arc::clone(dev_info_arc), Arc::clone(active_paths_mutex));
            }
        }
    }
}

fn start_device_thread(device_path: String, config: Arc<ConfigWrapper>, state: Arc<Mutex<bool>>,
                       hid_device_info: Arc<HidDeviceInfo>, active_paths_mutex: Arc<RwLock<HashSet<String>>>) {
    tokio::spawn(async move {
        println!("Opening event device: {}", device_path);
        let device = Device::open(&device_path)
            .expect("Failed to open input device");

        let mut stream = device.into_event_stream()
            .expect("Failed to create event stream");

        loop {
            let event = stream.next_event().await;
            if event.is_err() {
                println!("Error reading event, exiting loop: {:?}", event);
                break;
            }
            if let Ok(ev) = event {
                if ev.event_type() == EventType::KEY {
                    // check for kb_illum_toggle keycode
                    if config.kb_brightness_cycle.enabled
                        && ev.code() == config.kb_brightness_cycle.keycode_enum.unwrap().code()
                        && ev.value() == 1
                    {
                        println!("kb brightness event");
                        kb_illumination::cycle();
                    }

                    // check for fnlock
                    if config.fnlock.enabled
                        && ev.code() == config.fnlock.keycode_enum.unwrap().code()
                        && ev.value() == 1
                    {
                        let mut state = state.lock().await;
                        println!("Fn key event");
                        *state = !*state;
                        toggle_fn_lock(&hid_device_info.hidraw_device_path, state.clone());
                        save_state(state.clone());
                    }
                } else if ev.event_type() == EventType::SWITCH {
                    if ev.code() == SwitchCode::SW_TABLET_MODE.0 {
                        if config.tablet_kb_backlight_disable.enabled {
                            if ev.value() == 1 {
                                println!("Tablet mode enabled, disabling keyboard backlight");
                                kb_illumination::disable_toggle(true);
                            } else {
                                println!("Tablet mode disabled, restoring keyboard backlight");
                                kb_illumination::disable_toggle(false);
                            }
                        }
                    }
                }
            }
        }
        println!("Event device {} disconnected, exiting task", device_path);
        let mut data = active_paths_mutex.write().await;
        data.remove(&device_path);
    });
}

fn restore(config_path: &str) {
    let config = get_config(config_path);
    if !config.fnlock.enabled {
        println!("FnLock not enabled in config, nothing to restore");
        return;
    }
    let state = load_state();
    let dev_info = hid::get_hardware_info(&vec![]);
    toggle_fn_lock(&dev_info.hidraw_device_path, state);
    println!(
        "Restored FnLock state to: {}",
        if state { "on" } else { "off" }
    );
}
