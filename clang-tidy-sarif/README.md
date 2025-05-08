[![Workflow Status](https://github.com/psastras/sarif-rs/workflows/main/badge.svg)](https://github.com/psastras/sarif-rs/actions?query=workflow%3A%22main%22)

# clang-tidy-sarif

This crate provides a command line tool to convert `clang-tidy` diagnostic
output into SARIF.

The latest [documentation can be found here](https://docs.rs/clang_tidy_sarif).

clang-tidy is a popular linter / static analysis tool for C++. More information
can be found on the official page:
[https://clang.llvm.org/extra/clang-tidy/](https://clang.llvm.org/extra/clang-tidy/)

SARIF or the Static Analysis Results Interchange Format is an industry standard
format for the output of static analysis tools. More information can be found on
the official website:
[https://sarifweb.azurewebsites.net/](https://sarifweb.azurewebsites.net/).

## Installation

`clang-tidy-sarif` may be installed via `cargo`

```shell
cargo install clang-tidy-sarif
```

via [cargo-binstall](https://github.com/cargo-bins/cargo-binstall)

```shell
cargo binstall clang-tidy-sarif
```

or downloaded directly from Github Releases

```shell
# make sure to adjust the target and version (you may also want to pin to a specific version)
curl -sSL https://github.com/psastras/sarif-rs/releases/download/clang-tidy-sarif-v0.8.0/clang-tidy-sarif-x86_64-unknown-linux-gnu -o clang-tidy-sarif
```

### Fedora Linux

```shell
sudo dnf install <cli_name> # ex. cargo binstall clang-tidy-sarif
```

### Nix

Through the `nix` cli,

```shell
nix --accept-flake-config profile install github:psastras/sarif-rs#clang-tidy-sarif
```

## Usage

For most cases, simply run `clang-tidy` and pipe the results into
`clang-tidy-sarif`.

## Example

```shell
 clang-tidy -checks=cert-* -warnings-as-errors=* main.cpp -- | clang-tidy-sarif
```

If you are using Github Actions, SARIF is useful for integrating with Github
Advanced Security (GHAS), which can show code alerts in the "Security" tab of
your repository.

After uploading `clang-tidy-sarif` output to Github, `clang-tidy` diagnostics
are available in GHAS.

## Example

```yaml
on:
  workflow_run:
    workflows: ["main"]
    branches: [main]
    types: [completed]

name: sarif

jobs:
  upload-sarif:
    runs-on: ubuntu-latest
    if: ${{ github.ref == 'refs/heads/main' }}
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          profile: minimal
          toolchain: stable
          override: true
      - uses: Swatinem/rust-cache@v1
      - run: cargo install clang-tidy-sarif sarif-fmt
      - run: clang-tidy -checks=cert-* -warnings-as-errors=* main.cpp -- | clang-tidy-sarif | tee
          results.sarif | sarif-fmt
      - name: Upload SARIF file
        uses: github/codeql-action/upload-sarif@v1
        with:
          sarif_file: results.sarif
```

License: MIT
