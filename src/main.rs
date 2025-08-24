mod hid;
mod state;
mod key_listener;
mod config;

unsafe extern "C" {
    fn run_bpf(hid_id: u32, remap_array: *const u32, remap_count: u32) -> i32;
}

fn main() {
    println!("Hello, world!");
    let dev_info = hid::get_hardware_info();
    // do_pbf();
    // hid::toggle_fn_lock(hidraw_path, true);
    println!();
}

fn do_pbf()
{
    let remaps: [u32; 6] = [
        0x4e, 0x5c, // fn-lock (fn + esc) -> key_prog3
        0x7e, 0xba, // emoji picker key -> key_prog2
        0x8b, 0x38]; // proart hub key -> key_prog1
    unsafe {
        let ret = run_bpf(2, remaps.as_ptr(), remaps.len() as u32 / 2);
        println!("run_bpf returned: {}", ret);
    }
}
