{ config, lib, pkgs, ... }:

let
  cfg = config.services.btinfo;
in {
  options.services.btinfo = {
    enable = lib.mkEnableOption "btinfo Bluetooth notification daemon";

    package = lib.mkOption {
      type = lib.types.package;
      default = pkgs.btinfo;
      description = "The btinfo package to use.";
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];

    systemd.user.services.btinfo = {
      enable = true;
      description = "Bluetooth device info and notification daemon";
      wantedBy = [ "default.target" ];
      after = [ "dbus.service" ];

      serviceConfig = {
        ExecStart = "${cfg.package}/bin/dbusevents --mode event";
        Restart = "on-failure";
        RestartSec = "3s";
      };
    };
  };
}
