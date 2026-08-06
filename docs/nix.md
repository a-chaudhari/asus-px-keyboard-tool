# Nix / NixOS

This repo ships a Nix flake with a package and a NixOS module (uses nixpkgs-unstable, `x86_64-linux` only).

- Build the binary:
  - `nix build .#asus-px-keyboard-tool` (result in `./result/bin/asus-px-keyboard-tool`)
- Run the app (needs root for eBPF):
  - `sudo nix run .# -- /etc/asus-px-keyboard-tool.conf`

## NixOS module

The module:
- writes `/etc/asus-px-keyboard-tool.conf` from `services.asus-px-keyboard-tool.settings`
- creates `/var/lib/asus-px-kb-tool` automatically
- installs and starts a systemd service

### Step 1 — add the flake input

In your `flake.nix`, add the input and make it follow your nixpkgs to avoid downloading a second copy:

```nix
inputs = {
  nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

  asus-px-keyboard-tool = {
    url = "github:a-chaudhari/asus-px-keyboard-tool";
    inputs.nixpkgs.follows = "nixpkgs";
  };
};
```

### Step 2 — add the module and enable the service

Pass the module to your `nixosSystem` and enable the service in any module:

```nix
outputs = { self, nixpkgs, asus-px-keyboard-tool, ... }: {
  nixosConfigurations.my-host = nixpkgs.lib.nixosSystem {
    system = "x86_64-linux";
    modules = [
      asus-px-keyboard-tool.nixosModules.default
      ./configuration.nix   # your existing config
    ];
  };
};
```

Then, in `configuration.nix` (or any imported module):

```nix
services.asus-px-keyboard-tool.enable = true;
```

That's it. Rebuild with `sudo nixos-rebuild switch`.

### Step 3 (optional) — customise the config

The default settings match the minimal config from the README and target PX13/PX16 laptops.
Override any subset under `services.asus-px-keyboard-tool.settings`:

```nix
# Note: Nix integers are decimal — convert hex values (e.g. 0x7e → 126, 0xba → 186).
services.asus-px-keyboard-tool.settings = {
  bpf.remaps = [
    { from = 78;  to = 153; } # 0x4e -> 0x99  fn-lock (fn+esc) -> KEY_PROG4
    { from = 126; to = 186; } # 0x7e -> 0xba  emoji key        -> KEY_PROG2
  ];
  fnlock = {
    enabled = true;
    keycode = "KEY_PROG4";
    boot_default = "last"; # "last", "on", "off"
  };
  kb_brightness_cycle = {
    enabled = true;
    keycode = "KEY_PROG3";
  };
  tablet_kb_backlight_disable.enabled = false;
};
```

See the [Configuration section in the README](../Readme.md#configuration) for all available options.

### Using a local checkout

Set the flake URL to a local path instead of GitHub:

```nix
asus-px-keyboard-tool.url = "path:/path/to/asus-px-keyboard-tool";
```

