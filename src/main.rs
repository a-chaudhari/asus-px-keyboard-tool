mod hid;
mod state;
mod apkt_config;
mod kb_illumination;

use evdev_rs::Device;
use evdev_rs::enums::{EventCode, EventType};
use evdev_rs::ReadFlag;
use crate::hid::toggle_fn_lock;
use crate::kb_illumination::cycle;
use crate::apkt_config::{get_config, Remap};
use crate::state::{load_state, save_state};

unsafe extern "C" {
    fn run_bpf(hid_id: u32, remap_array: *const u32, remap_count: u32) -> i32;
}

fn main() {
    // read args to get config path
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        panic!("Usage: {} [config_path]", args[0]);
    }
    let config = get_config("asus-px-keyboard-tool");
    let dev_info = hid::get_hardware_info(config.compatibility.keyd);
    if config.bpf.enabled {
        println!("Starting BPF with remaps: {:?}", config.bpf.remaps);
        start_bpf(dev_info.hid_id, config.bpf.remaps);
    } else {
        println!("BPF disabled in config");
    }
    let input_device = Device::new_from_path(dev_info.input_device_path).unwrap();
    let mut state;

    // apply initial fnlock state
    if config.fnlock.boot_default == "on" {
        state = true;
    } else if config.fnlock.boot_default == "off" {
        state = false;
    } else if config.fnlock.boot_default == "last" {
        state = load_state();
    } else {
        panic!("Invalid fnlock.boot_default value in config: {}", config.fnlock.boot_default);
    }
    toggle_fn_lock(&dev_info.hidraw_device_path, state);
    save_state(state);

    // convert string to enum
    let mut illum_keycode : Option<EventCode> = None;
    let mut fnlock_keycode : Option<EventCode> = None;

    if config.kb_brightness_cycle.enabled {
        illum_keycode = Some(EventCode::from_str(&EventType::EV_KEY, &config.kb_brightness_cycle.keycode).unwrap());
    }

    if config.fnlock.enabled {
        fnlock_keycode = Some(EventCode::from_str(&EventType::EV_KEY, &config.fnlock.keycode).unwrap());
    }

    loop {
        let ev = input_device.next_event(ReadFlag::NORMAL).map(|val| val.1);
        match ev {
            Ok(ev) => {
                // check for kb_illum_toggle keycode
                if config.kb_brightness_cycle.enabled && ev.event_code == illum_keycode.unwrap() && ev.value == 1{
                    println!("kb brightness event");
                    cycle();
                }

                // check for fnlock
                if config.fnlock.enabled && ev.event_code == fnlock_keycode.unwrap() && ev.value == 1{
                    println!("Fn key event");
                    state = !state;
                    toggle_fn_lock(&dev_info.hidraw_device_path, state);
                    save_state(state);
                }
            },
            Err(_) => (),
        }
    }
}

fn start_bpf(hid_id: u32, remaps: Vec<Remap>)
{
    let flat_remaps: Vec<u32> = remaps.iter().flat_map(|r| vec![r.from as u32, r.to as u32]).collect();
    unsafe {
        let ret = run_bpf(hid_id, flat_remaps.as_ptr(), remaps.len() as u32);
        println!("run_bpf returned: {}", ret);
    }
}
