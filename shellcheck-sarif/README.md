[![Workflow Status](https://github.com/psastras/sarif-rs/workflows/main/badge.svg)](https://github.com/psastras/sarif-rs/actions?query=workflow%3A%22main%22)

# shellcheck-sarif

This crate provides a command line tool to convert `shellcheck` diagnostic
output into SARIF.

The latest [documentation can be found here](https://docs.rs/shellcheck_sarif).

shellcheck is a popular linter / static analysis tool for shell scripts. More
information can be found on the official repository:
[https://github.com/koalaman/shellcheck](https://github.com/koalaman/shellcheck)

SARIF or the Static Analysis Results Interchange Format is an industry standard
format for the output of static analysis tools. More information can be found on
the official website:
[https://sarifweb.azurewebsites.net/](https://sarifweb.azurewebsites.net/).

## Installation

`shellcheck-sarif` may be installed via `cargo`

```shell
cargo install shellcheck-sarif
```

via [cargo-binstall](https://github.com/cargo-bins/cargo-binstall)

```shell
cargo binstall shellcheck-sarif
```

or downloaded directly from Github Releases

```shell
# make sure to adjust the target and version (you may also want to pin to a specific version)
curl -sSL https://github.com/psastras/sarif-rs/releases/download/shellcheck-sarif-v0.6.5/shellcheck-sarif-x86_64-unknown-linux-gnu -o shellcheck-sarif
```

### Fedora Linux

```shell
sudo dnf install <cli_name> # ex. cargo binstall shellcheck-sarif
```

### Nix

Through the `nix` cli,

```shell
nix --accept-flake-config profile install github:psastras/sarif-rs#shellcheck-sarif
```

## Usage

For most cases, simply run `shellcheck` with `json` output and pipe the results
into `shellcheck-sarif`.

## Example

```shell
shellcheck -f json shellscript.sh | shellcheck-sarif
```

If you are using Github Actions, SARIF is useful for integrating with Github
Advanced Security (GHAS), which can show code alerts in the "Security" tab of
your repository.

After uploading `shellcheck-sarif` output to Github, `shellcheck` diagnostics
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
      - run: cargo install shellcheck-sarif sarif-fmt
      - run: shellcheck -f json shellscript.sh | shellcheck-sarif | tee
          results.sarif | sarif-fmt
      - name: Upload SARIF file
        uses: github/codeql-action/upload-sarif@v1
        with:
          sarif_file: results.sarif
```

License: MIT
