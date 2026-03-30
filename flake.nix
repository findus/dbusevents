{
  description = "Bluetooth device info and notification daemon";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }: {

    overlays.default = import ./overlay.nix;

    nixosModules.default = import ./module.nix;

    # Convenience: packages for common systems
    packages = nixpkgs.lib.genAttrs [ "x86_64-linux" "aarch64-linux" ] (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ self.overlays.default ];
        };
      in {
        default = pkgs.btinfo;
        btinfo = pkgs.btinfo;
      }
    );

    apps = nixpkgs.lib.genAttrs [ "x86_64-linux" "aarch64-linux" ] (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ self.overlays.default ];
        };
        mkApp = bin: { type = "app"; program = "${pkgs.btinfo}/bin/${bin}"; };
      in {
        default = mkApp "dbusevents";
        dbusevents = mkApp "dbusevents";
        dbusbtinfo = mkApp "dbusbtinfo";
      }
    );

  };
}
