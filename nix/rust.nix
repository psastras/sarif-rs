{ inputs, ... }:
{
  debug = true;
  perSystem = { config, self', pkgs, lib, system, ... }:
    let
      inherit (pkgs.stdenv) isDarwin;
      inherit (pkgs.darwin) apple_sdk;
      globalCrateConfig = {
        crane = {
          args = {
            buildInputs = lib.optionals isDarwin
              ([
                pkgs.fixDarwinDylibNames
              ]) ++ [
              pkgs.libiconv
            ];
            cargoTestExtraArgs = "-- --nocapture";
            DEVOUR_FLAKE = inputs.devour-flake;
          } // lib.optionalAttrs pkgs.stdenv.isLinux {
            CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
            CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";
          };
          extraBuildArgs = {
            postInstall = ''
              ${if isDarwin then "fixDarwinDylibNames" else ""}
            '';
          };
        };
      };
    in
    {
      rust-project = {
        src = lib.cleanSourceWith {
          name = "sarif-rs";
          src = inputs.self; # The original, unfiltered source
          filter = path: type:
            # Needed for documentation checks
            (lib.hasSuffix "\.md" path) ||
            # Needed for .json schema
            (lib.hasSuffix "\.json" path) ||
            # Needed for tests
            (lib.hasInfix "/data/" path) ||
            # Default filter from crane (allow .rs files)
            (config.rust-project.crane-lib.filterCargoSources path type)
          ;
        };
      };

      packages =
        let
          inherit (config.rust-project) crates;
        in
        rec {
          all = pkgs.symlinkJoin {
            name = "all";
            paths = with crates; [
              sarif-fmt.crane.outputs.drv.crate
              clippy-sarif.crane.outputs.drv.crate
              hadolint-sarif.crane.outputs.drv.crate
              miri-sarif.crane.outputs.drv.crate
              shellcheck-sarif.crane.outputs.drv.crate
              clang-tidy-sarif.crane.outputs.drv.crate
              deny-sarif.crane.outputs.drv.crate
            ];
          };
          default = all;
        };
    };
}
