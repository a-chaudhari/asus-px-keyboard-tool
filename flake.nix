{
  description = "asus-px-keyboard-tool";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs =
    { self, nixpkgs, ... }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
    in
    {
      packages.${system} = {
        asus-px-keyboard-tool = pkgs.rustPlatform.buildRustPackage {
          pname = "asus-px-keyboard-tool";
          version = "0.1.0";
          src = pkgs.lib.cleanSource ./.;

          cargoHash = "sha256-y8V+VbV9VzhF82VypEDLRRNdxuJXT0cGm6epW9mqtRg=";

          nativeBuildInputs = with pkgs; [
            pkg-config
            llvmPackages_latest.clang-unwrapped
            bpftools
          ];

          buildInputs = with pkgs; [
            libbpf
            elfutils
            systemd # for libudev
            libevdev
            zlib
          ];

          # Use unwrapped clang for BPF builds to avoid cc-wrapper's flags
          env = {
            CLANG = "${pkgs.llvmPackages_latest.clang-unwrapped}/bin/clang";
            BPF_CLANG = "${pkgs.llvmPackages_latest.clang-unwrapped}/bin/clang";
            LIBBPF_CC = "${pkgs.llvmPackages_latest.clang-unwrapped}/bin/clang";
          };

          meta = with pkgs.lib; {
            description = "Improve ASUS PX keyboard functionality (HID/eBPF)";
            homepage = "https://github.com/a-chaudhari/asus-px-keyboard-tool";
            platforms = [ system ];
          };
        };
        default = self.packages.${system}.asus-px-keyboard-tool;
      };

      apps.${system}.default = {
        type = "app";
        program = "${self.packages.${system}.asus-px-keyboard-tool}/bin/asus-px-keyboard-tool";
      };


      nixosModules = {
        default = self.nixosModules.asus-px-keyboard-tool;
        asus-px-keyboard-tool =
          {
            config,
            lib,
            pkgs,
            ...
          }:
          let
            cfg = config.services.asus-px-keyboard-tool;
            tomlFormat = pkgs.formats.toml { };
            pkgDefault = self.packages.${system}.asus-px-keyboard-tool;
            # Defaults based on the README's minimal config snippet
            defaultSettings = {
              bpf = {
                enabled = true;
                remaps = [
                  # fn-lock (fn + esc) -> key_prog4
                  {
                    from = 78;
                    to = 153;
                  }
                  # The README shows additional commented remaps as examples;
                  # users can add them under services.asus-px-keyboard-tool.settings.bpf.remaps
                ];
              };
              fnlock = {
                enabled = true;
                keycode = "KEY_PROG4";
                boot_default = "last"; # "last", "on", "off"
              };
              kb_brightness_cycle = {
                enabled = false;
                keycode = "KEY_PROG3";
              };
              tablet_kb_backlight_disable = {
                enabled = false;
              };
              # compatibility table intentionally omitted here; defaults baked into the binary
            };
          in
          {
            options.services.asus-px-keyboard-tool = with lib; {
              enable = mkEnableOption "ASUS PX keyboard tool (HID remaps, fn-lock, brightness cycle)";

              package = mkOption {
                type = types.package;
                default = pkgDefault;
                defaultText = lib.literalExpression "self.packages.${system}.asus-px-keyboard-tool";
                description = "Package to use for the asus-px-keyboard-tool service.";
              };

              settings = mkOption {
                type = tomlFormat.type;
                default = defaultSettings;
                description = ''
                  Configuration written to /etc/asus-px-keyboard-tool.conf.
                  See README for details. This default mirrors the README's minimal config.
                '';
              };
            };

            config = lib.mkIf cfg.enable {
              environment.etc."asus-px-keyboard-tool.conf".source =
                tomlFormat.generate "asus-px-keyboard-tool.conf" cfg.settings;

              systemd.services.asus-px-keyboard-tool = {
                description = "asus px kb tool";
                wantedBy = [ "multi-user.target" ];
                unitConfig = {
                  StartLimitIntervalSec = 30;
                  StartLimitBurst = 5;
                };
                serviceConfig = {
                  Type = "simple";
                  ExecStart = "${cfg.package}/bin/asus-px-keyboard-tool /etc/asus-px-keyboard-tool.conf";
                  TimeoutSec = 5;
                  Restart = "on-failure";
                  # Create and manage /var/lib/asus-px-kb-tool
                  StateDirectory = "asus-px-kb-tool";
                };
              };
            };
          };
      };
    };
}
