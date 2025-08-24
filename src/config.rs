pub struct Remap {
    from: u8,
    to: u8,
}

pub enum FnLockDefault {
    On,
    Off,
    Preserve,
}

pub struct Config {
    bpf_enabled: bool,
    bpf_remaps: Vec<Remap>,

    fn_lock_enabled: bool,
    fn_lock_keycode: u8,
    fn_lock_default: FnLockDefault,

    kb_enabled: bool,
    kb_keycode: u8,
}

pub fn load_config() -> Config {
    // For now, return a default config
    Config {
        bpf_enabled: true,
        bpf_remaps: vec![
            Remap { from: 0x4e, to: 0x5c }, // fn-lock (fn + esc) -> key_prog3
            Remap { from: 0x7e, to: 0xba }, // emoji picker key -> key_prog2
            Remap { from: 0x8b, to: 0x38 }, // proart hub key -> key_prog1
        ],
        fn_lock_enabled: true,
        fn_lock_keycode: 0x4e,
        fn_lock_default: FnLockDefault::Preserve,
        kb_enabled: true,
        kb_keycode: 0x3a, // Example keycode for keyboard backlight toggle
    }
}