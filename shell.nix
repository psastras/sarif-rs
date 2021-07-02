let nixpkgs = import <nixpkgs> {
  config = { allowUnfree = true; };
};
in
with nixpkgs;

pkgs.mkShell {
  buildInputs = [
    bashInteractive
    pkgs.codeql 
  ];
}