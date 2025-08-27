# Asus PX Keyboard Tool
A tool to fix missing functionality in Asus PX keyboards on Linux. Parts of this may work with other Asus laptops as well.

## Features
- Can remap ignored hotkeys to ones the asus keyboard driver supports
  - like the emoji and proart keys!
- Can listen for fn-lock key presses and toggle fn-lock state
- Adds support for the single-button keyboard backlight cycle key
- Optional compatibility mode for keyd users

## Installation
- run the install.sh script `sudo ./install.sh`
- modify the config file at `/etc/asus-px-keyboard-tool.conf`
  - ⚠️ see the config section below. very important!
- enable the systemd service with `systemctl enable --now asus-px-keyboard-tool.service`

## Uninstallation
The uninstall script will clean up all files. `sudo ./uninstall.sh`

## Configuration
The default config shipped is optimized for a PX13 or PX16 laptop.  Other asus models can still use this tool, but should try with the minimal config below as a starting point.

Submit an issue if you have problems with the default config or need help customizing it.  Admittedly I'm still working config on documentation.

```
# TOML configuration file for asus-px-keyboard-tool
# minimal config

[bpf]
enabled = true
remaps = [
    { from = 0x4e, to = 0x5c }, # fn-lock (fn + esc) -> key_prog3
#    { from = 0x7e, to = 0xba }, # emoji picker key -> key_prog2
#    { from = 0x8b, to = 0x38 }, # proart hub key -> key_prog1
#    { from = 0xc7, to = 0x99 }, # kb backlight key -> key_prog4
]

[compatibility]
keyd = false # only enable if you use keyd

[fnlock]
enabled = true
keycode = "KEY_PROG3"
boot_default = "last" # "last", "on", "off"

[kb_brightness_cycle]
enabled = false
keycode = "KEY_PROG4"
```

## Creating your own BPF remaps

### TL;DR
1. Find which scancodes are being sent by the ignored keys. The tool will log all detected scancodes.
    * `journalctl -f -u asus-px-keyboard-tool` 
    * if you don't see any `BPF:` log messages, try running the tool manually `sudo asus-px-keyboard-tool /etc/asus-px-keyboard-tool.conf`
2. Look at the hid-asus driver source code to find supported scancodes and their corresponding keycodes.
   * https://github.com/torvalds/linux/blob/1b237f190eb3d36f52dffe07a40b5eb210280e00/drivers/hid/hid-asus.c#L964-L992
3. Pick a supported scancode that you don't care about and note its keycode.
4. Add remap entries in the config file to map ignored scancodes to the supported ones.

### High level overview
Your keyboard hardware sends "scancodes" when you press a button.  The linux kernel (and the driver attached) takes these 
scancodes and converts them to keycodes which is what the rest of the system uses.

For Example: the physical Backlight Up key sends a scancode of `0x20` which the kernel maps to keycode `KEY_BRIGHTNESSUP`.  
Some utility, likely your desktop environment, listens for this keycode and increases the brightness and shows a nice on screen display. 

The asus driver (hid-asus) doesn't support every scancode out there in every asus keyboard. So some keys just don't work.  
For example, the emoji key on the PX13 sends a scancode of `0x7e` which the driver ignores.  So pressing the emoji key does nothing.

The bpf program allows modifying keyboard scancodes.  This is done at the kernel level before the hid-asus driver even receives the events.
This gets around limitations of the hid-asus driver by remapping ignored scancodes to ones that work

In the example above, we remap the emoji key's scancode `0x7e` to `0xba` which the driver recognizes as the "Fn+C ASUS Splendid" key which gets the keycode `KEY_PROG2`.
This is ok for the PX13 because there is no Splendid key. If your laptop has a Splendid key, you should pick a different one.
You can then use this keycode in your desktop environment or window manager to launch your emoji picker.

By looking at the source code of the hid-asus driver, you can find out which scancodes are supported and what keycodes they map to.  
Then you can pick the ones you don't care about and remap your ignored keys to those. 

## Building
### Main Tool
Most of the codebase is written in rust.  So it requires a Rust toolchain.  Then run `cargo build`.  The binary will be at `target/debug/asus-px-keyboard-tool`.
### BPF Program
The bpf program is written in C and compiled with clang. Cargo (rust build tool) will compile it automatically when you build the main tool.
```

Non-exhaustive list:
- libbpf-dev
- clang
- linux-headers