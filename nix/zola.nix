let
  sources = import ./sources.nix;
  pkgs = import sources.nixpkgs {};

  inputs = with pkgs; [
    zola
  ];

  shell = pkgs.mkShell {
    buildInputs = inputs;
  };
in {
  shell = shell;
}
