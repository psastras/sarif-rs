{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/25.05";
    flake-parts.url = "github:hercules-ci/flake-parts/main";
    systems.url = "github:nix-systems/default";
    pre-commit-hooks.url = "github:cachix/pre-commit-hooks.nix";
    rust-flake.url = "github:juspay/rust-flake/f69408a404f09afe0d85be88eddff07a054c2397";
    devour-flake.url = "github:srid/devour-flake";
    devour-flake.flake = false;
  };

  outputs = inputs:

    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = import inputs.systems;

      imports = [
        inputs.rust-flake.flakeModules.default
        inputs.rust-flake.flakeModules.nixpkgs
        inputs.pre-commit-hooks.flakeModule
        ./nix/pre-commit.nix
        ./nix/rust.nix
      ];

      perSystem = { pkgs, self', config, ... }: {
        formatter = pkgs.nixpkgs-fmt;
        devShells.default = pkgs.mkShell {
          inputsFrom = [
            self'.devShells.rust
            config.pre-commit.devShell
          ];
          # Add your devShell tools here clang-tidy ./sarif-fmt/tests/data/cpp.cpp 
          packages = with pkgs; [
            git-cliff
            zola
          ];

          shellHook =
            ''
              # For nixci
              export DEVOUR_FLAKE=${inputs.devour-flake}
            '';
        };
      };
    };
}
