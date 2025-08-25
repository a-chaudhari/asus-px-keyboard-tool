mod hid;
mod state;
mod myconfig;

use evdev_rs::Device;
use evdev_rs::enums::{EventCode, EventType};
use evdev_rs::ReadFlag;
use crate::hid::toggle_fn_lock;
use crate::myconfig::{get_config, Remap};
use crate::state::{load_state, save_state};

unsafe extern "C" {
    fn run_bpf(hid_id: u32, remap_array: *const u32, remap_count: u32) -> i32;
}

fn main() {
    let config = get_config();
    let dev_info = hid::get_hardware_info(config.compatibility.keyd);
    // hid::toggle_fn_lock(hidraw_path, true);
    start_bpf(dev_info.hid_id, config.bpf.remaps);
    let d = Device::new_from_path(dev_info.input_device_path).unwrap();
    let mut state = load_state();

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
        let ev = d.next_event(ReadFlag::NORMAL).map(|val| val.1);
        match ev {
            Ok(ev) => {
                // check for kb_illum_toggle keycode
                if config.kb_brightness_cycle.enabled && ev.event_code == illum_keycode.unwrap() && ev.value == 1{
                    println!("kb brightness event: {:?}", ev.value);
                }

                // check for fnlock
                if config.fnlock.enabled && ev.event_code == fnlock_keycode.unwrap() && ev.value == 1{
                    println!("Fn key event: {:?}", ev.value);
                    state = !state;
                    toggle_fn_lock(&dev_info.hidraw_device_path, state);
                    save_state(state);
                }
            },
            Err(e) => (),
        }
    }
}

fn start_bpf(hid_id: u32, remaps: Vec<Remap>)
{
    let flat_remaps: Vec<u32> = remaps.iter().flat_map(|r| vec![r.from as u32, r.to as u32]).collect();
    // let remaps: [u32; 6] = [
    //     0x4e, 0x5c, // fn-lock (fn + esc) -> key_prog3
    //     0x7e, 0xba, // emoji picker key -> key_prog2
    //     0x8b, 0x38]; // proart hub key -> key_prog1
    unsafe {
        let ret = run_bpf(hid_id, flat_remaps.as_ptr(), remaps.len() as u32);
        println!("run_bpf returned: {}", ret);
    }
}
