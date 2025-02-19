{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    xnode-demo = {
      url = "github:Openmesh-Network/xnode-demo";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      xnode-demo,
      ...
    }:
    let
      system = "x86_64-linux";
    in
    {
      nixosConfigurations.container = nixpkgs.lib.nixosSystem {
        inherit system;
        specialArgs = {
          inherit xnode-demo;
        };
        modules = [
          (
            { xnode-demo, ... }:
            {
              imports = [
                xnode-demo.nixosModules.default
              ];

              boot.isContainer = true;

              services.xnode-demo = {
                enable = true;
                xnodes = [ "http://localhost:34391" ];
              };

              networking = {
                firewall.allowedTCPPorts = [ 35963 ];

                useHostResolvConf = nixpkgs.lib.mkForce false;
              };

              services.resolved.enable = true;

              system.stateVersion = "25.05";
            }
          )
        ];
      };
    };
}
