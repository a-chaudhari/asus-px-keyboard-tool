use config::{File, FileFormat};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Remap {
    pub from: u32,
    pub to: u32,
}

#[derive(Debug, Deserialize)]
pub struct CompatibilityConfig {
    pub keyd: bool,
}

#[derive(Debug, Deserialize)]
pub struct FnLockConfig {
    pub enabled: bool,
    pub keycode: String,
    pub boot_default: String,
}

#[derive(Debug, Deserialize)]
pub struct ConfigWrapper {
    pub compatibility: CompatibilityConfig,
    pub fnlock: FnLockConfig,
    pub bpf: BpfConfig,
    pub kb_brightness_cycle: KbBrightnessConfig,
}

#[derive(Debug, Deserialize)]
pub struct BpfConfig {
    pub enabled: bool,
    pub remaps: Vec<Remap>,
}

#[derive(Debug, Deserialize)]
pub struct KbBrightnessConfig {
    pub enabled: bool,
    pub keycode: String,
}

pub fn get_config(path: &str) -> ConfigWrapper {
    let settings = config::Config::builder()
        // Load defaults from embedded string
        .add_source(File::from_str(DEFAULT_CONFIG, FileFormat::Toml))
        .add_source(config::File::with_name(path).format(config::FileFormat::Toml))
        .build()
        .unwrap();
    settings.try_deserialize::<ConfigWrapper>().unwrap()
}

static DEFAULT_CONFIG: &str = r#"
[bpf]
enabled = false
remaps = []

[compatibility]
keyd = false # only enable if you use keyd

[fnlock]
enabled = false
keycode = "KEY_PROG3"
boot_default = "last" # "last", "on", "off"

[kb_brightness_cycle]
enabled = false
keycode = "KEY_PROG4"
"#;