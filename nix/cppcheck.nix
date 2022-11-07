let
  sources = import ./sources.nix;
  pkgs = import sources.nixpkgs {};

  inputs = with pkgs; [
    cppcheck
  ];

  shell = pkgs.mkShell {
    buildInputs = inputs;
  };
in {
  shell = shell;
}
