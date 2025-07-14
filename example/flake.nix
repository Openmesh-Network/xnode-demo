{
  inputs = {
    xnode-manager.url = "github:Openmesh-Network/xnode-manager";
    xnode-demo.url = "github:Openmesh-Network/xnode-demo";
    nixpkgs.follows = "xnode-demo/nixpkgs";
  };

  outputs = inputs: {
    nixosConfigurations.container = inputs.nixpkgs.lib.nixosSystem {
      specialArgs = {
        inherit inputs;
      };
      modules = [
        inputs.xnode-manager.nixosModules.container
        {
          services.xnode-container.xnode-config = {
            host-platform = ./xnode-config/host-platform;
            state-version = ./xnode-config/state-version;
            hostname = ./xnode-config/hostname;
          };
        }
        inputs.xnode-demo.nixosModules.default
        (
          { config, ... }:
          {
            services.xnode-demo = {
              enable = true;
              xnodes = [ "xnode-manager.local" ];
            };

            networking.firewall.allowedTCPPorts = [ config.services.xnode-demo.port ];
          }
        )
      ];
    };
  };
}
