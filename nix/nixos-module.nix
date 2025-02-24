{
  config,
  pkgs,
  lib,
  ...
}:
let
  cfg = config.services.xnode-demo;
  xnode-demo = pkgs.callPackage ./package.nix { };
in
{
  options = {
    services.xnode-demo = {
      enable = lib.mkEnableOption "Enable the rust app";

      hostname = lib.mkOption {
        type = lib.types.str;
        default = "0.0.0.0";
        example = "127.0.0.1";
        description = ''
          The hostname under which the app should be accessible.
        '';
      };

      port = lib.mkOption {
        type = lib.types.port;
        default = 35963;
        example = 35963;
        description = ''
          The port under which the app should be accessible.
        '';
      };

      verbosity = lib.mkOption {
        type = lib.types.str;
        default = "warn";
        example = "info";
        description = ''
          The logging verbosity that the app should use.
        '';
      };

      dataDir = lib.mkOption {
        type = lib.types.path;
        default = "/var/lib/xnode-demo";
        example = "/var/lib/xnode-demo";
        description = ''
          The main directory to store data.
        '';
      };

      reservationsDir = lib.mkOption {
        type = lib.types.path;
        default = "${cfg.dataDir}/reservation";
        example = "/var/lib/xnode-demo/reservation";
        description = ''
          The directory to store Xnode reservations.
        '';
      };

      reservationDuration = lib.mkOption {
        type = lib.types.ints.u32;
        default = 3600;
        example = 3600;
        description = ''
          Amount of seconds a reservation lasts.
        '';
      };

      xnodes = lib.mkOption {
        type = lib.types.listOf lib.types.str;
        default = [ ];
        example = [
          "http://192.168.0.0:34391"
          "https://google.com"
        ];
        description = ''
          The list of Xnode manager instances to give demo access to. Trailing slashes are not allowed.
        '';
      };
    };
  };

  config = lib.mkIf cfg.enable {
    systemd.services.xnode-demo = {
      wantedBy = [ "multi-user.target" ];
      description = "Rust App.";
      after = [ "network.target" ];
      environment = {
        HOSTNAME = cfg.hostname;
        PORT = toString cfg.port;
        RUST_LOG = cfg.verbosity;
        DATADIR = cfg.dataDir;
        RESERVATIONSDIR = cfg.reservationsDir;
        RESERVATIONDURATION = toString cfg.reservationDuration;
        XNODES = toString cfg.xnodes;
      };
      serviceConfig = {
        ExecStart = "${lib.getExe xnode-demo}";
        User = "root";
        Group = "root";
        CacheDirectory = "rust-app";
      };
    };
  };
}
