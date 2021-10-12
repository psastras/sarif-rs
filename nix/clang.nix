let
  sources = import ./sources.nix;
  pkgs = import sources.nixpkgs {};

  inputs = with pkgs; [
    clang-tools
  ];

  shell = pkgs.mkShell {
    buildInputs = inputs;
  };
in {
  shell = shell;
}
