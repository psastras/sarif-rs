let
  sources = import ./sources.nix;
  pkgs = import sources.nixpkgs {};

  inputs = with pkgs; [
    hadolint
  ];

  shell = pkgs.mkShell {
    buildInputs = inputs;
  };
in {
  shell = shell;
}
