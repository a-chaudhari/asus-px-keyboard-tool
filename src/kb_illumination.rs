static KB_BRIGHTNESS_PATH: &str = "/sys/class/leds/asus::kbd_backlight";
static mut saved_value: Option<u32> = None;

pub fn cycle() {
    println!("kb brightness cycle!");
    let max_brightness = get_max_brightness();
    let current_brightness = get_current_brightness();
    let new_brightness = if current_brightness < max_brightness {
        current_brightness + 1
    } else {
        0
    };
    set_brightness(new_brightness);
}

pub fn toggle(state: bool) {
    unsafe {
        if !state {
            // Save current brightness
            if saved_value.is_none() {
                saved_value = Some(get_current_brightness());
            }
            // Turn off keyboard backlight
            set_brightness(0);
        } else {
            // Restore saved brightness
            if let Some(value) = saved_value {
                set_brightness(value);
                saved_value = None;
            }
        }
    }
}

fn get_max_brightness() -> u32 {
    let max_brightness_path = format!("{}/max_brightness", KB_BRIGHTNESS_PATH);
    let contents = std::fs::read_to_string(max_brightness_path)
        .expect("Unable to read max_brightness file");
    contents.trim().parse::<u32>()
        .expect("Unable to parse max_brightness value")
}

fn get_current_brightness() -> u32 {
    let brightness_path = format!("{}/brightness", KB_BRIGHTNESS_PATH);
    let contents = std::fs::read_to_string(brightness_path)
        .expect("Unable to read brightness file");
    contents.trim().parse::<u32>()
        .expect("Unable to parse brightness value")
}

fn set_brightness(value: u32) {
    let brightness_path = format!("{}/brightness", KB_BRIGHTNESS_PATH);
    std::fs::write(brightness_path, value.to_string())
        .expect("Unable to write brightness file");
}