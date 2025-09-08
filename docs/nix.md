# Nix / NixOS

This repo ships a Nix flake with a package and a NixOS module (using nixpkgs-unstable).

- Build the binary:
  - `nix build .#asus-px-keyboard-tool` (result in `./result/bin/asus-px-keyboard-tool`)
- Run the app (needs root for eBPF):
  - `sudo nix run .# -- /etc/asus-px-keyboard-tool.conf`

## NixOS module

You can enable the service and manage the config declaratively.

The module writes the config file `/etc/asus-px-keyboard-tool.conf` (settings taken from `services.asus-px-keyboard-tool.settings` option) and creates `/var/lib/asus-px-kb-tool` automatically.

Set `services.asus-px-keyboard-tool.enable = true` to enable the systemd service.

Flake-based NixOS configuration example:

```
{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  # Use the module directly from this repo or GitHub
  inputs.asus-px-keyboard-tool.url = "github:a-chaudhari/asus-px-keyboard-tool";

  outputs = { self, nixpkgs, asus-px-keyboard-tool, ... }: {
    nixosConfigurations.my-host = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        asus-px-keyboard-tool.nixosModules.default
        ({ pkgs, ... }: {
          services.asus-px-keyboard-tool.enable = true;

          # Optional: override defaults written to /etc/asus-px-keyboard-tool.conf
          # Note: Nix integers are decimal; convert hex (e.g. 0x7e) to decimal (126).
          services.asus-px-keyboard-tool.settings = {
            bpf.remaps = [
              { from = 126; to = 186; } # 0x7e -> 0xba (emoji -> KEY_PROG2)
            ];
            kb_brightness_cycle = {
              enabled = true;
              keycode = "KEY_PROG3";
            };
          };
        })
      ];
    };
  };
}
```

Using a local checkout instead of GitHub: set `inputs.asus-px-keyboard-tool.url = "path:/path/to/asus-px-keyboard-tool"`.

