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
        .add_source(config::File::with_name(path))
        .build()
        .unwrap();
    settings.try_deserialize::<ConfigWrapper>().unwrap()
}
