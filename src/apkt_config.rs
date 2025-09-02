use config::{File, FileFormat};
use evdev::KeyCode;
use evdev_rs::enums::EV_KEY;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Remap {
    pub from: u32,
    pub to: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CompatibilityConfig {
    pub hid_path_override: Option<String>,
    pub hid_id_override: Option<u32>,
    pub event_path_override: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FnLockConfig {
    pub enabled: bool,
    pub keycode: String,
    pub boot_default: String,
    #[serde(skip)]
    pub keycode_enum: Option<KeyCode>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ConfigWrapper {
    pub compatibility: CompatibilityConfig,
    pub fnlock: FnLockConfig,
    pub bpf: BpfConfig,
    pub tablet_kb_backlight_disable: TabletKbBacklightDisableConfig,
    pub kb_brightness_cycle: KbBrightnessConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BpfConfig {
    pub enabled: bool,
    pub remaps: Vec<Remap>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct KbBrightnessConfig {
    pub enabled: bool,
    pub keycode: String,
    #[serde(skip)]
    pub keycode_enum: Option<KeyCode>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TabletKbBacklightDisableConfig {
    pub enabled: bool,
}

pub fn get_config(path: &str) -> ConfigWrapper {
    let settings = config::Config::builder()
        // Load defaults from embedded string
        .add_source(File::from_str(DEFAULT_CONFIG, FileFormat::Toml))
        .add_source(config::File::with_name(path).format(FileFormat::Toml))
        .build()
        .unwrap();
    let mut config = settings.try_deserialize::<ConfigWrapper>().unwrap();

    if config.kb_brightness_cycle.enabled {
        let ev_key = EV_KEY::from(config.kb_brightness_cycle.keycode.parse()
            .expect("Invalid kb_brightness keycode in config"));
        let key_code = KeyCode::new(ev_key as u16);
        config.kb_brightness_cycle.keycode_enum = Some(key_code);
    }

    if config.fnlock.enabled {
        let ev_key = EV_KEY::from(config.fnlock.keycode.parse()
            .expect("Invalid fnlock keycode in config"));
        let key_code = KeyCode::new(ev_key as u16);
        config.fnlock.keycode_enum = Some(key_code);
    }
    
    config // return
}

static DEFAULT_CONFIG: &str = r#"
[bpf]
enabled = false
remaps = []

[compatibility]

[fnlock]
enabled = false
keycode = "KEY_PROG3"
boot_default = "last" # "last", "on", "off"

[kb_brightness_cycle]
enabled = false
keycode = "KEY_PROG4"

[tablet_kb_backlight_disable]
enabled = false
"#;