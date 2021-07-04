let
  sources = import ./sources.nix;
  pkgs = import sources.nixpkgs {};

  inputs = with pkgs; [
    hadolint
    shellcheck
  ];

  shell = pkgs.mkShell {
    buildInputs = inputs;
  };
in {
  shell = shell;
}
