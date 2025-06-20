[![Workflow Status](https://github.com/psastras/sarif-rs/workflows/main/badge.svg)](https://github.com/psastras/sarif-rs/actions?query=workflow%3A%22main%22)

# hadolint-sarif

This crate provides a command line tool to convert `hadolint` diagnostic output
into SARIF.

The latest [documentation can be found here](https://docs.rs/hadolint_sarif).

hadolint is a popular linter / static analysis tool for Dockerfiles. More
information can be found on the official repository:
[https://github.com/hadolint/hadolint](https://github.com/hadolint/hadolint)

SARIF or the Static Analysis Results Interchange Format is an industry standard
format for the output of static analysis tools. More information can be found on
the official website:
[https://sarifweb.azurewebsites.net/](https://sarifweb.azurewebsites.net/).

## Installation

`hadolint-sarif` may be installed via `cargo`

```shell
cargo install hadolint-sarif
```

via [cargo-binstall](https://github.com/cargo-bins/cargo-binstall)

```shell
cargo binstall hadolint-sarif
```

or downloaded directly from Github Releases

```shell
# make sure to adjust the target and version (you may also want to pin to a specific version)
curl -sSL https://github.com/psastras/sarif-rs/releases/download/hadolint-sarif-v0.8.0/hadolint-sarif-x86_64-unknown-linux-gnu -o hadolint-sarif
```

### Fedora Linux

```shell
sudo dnf install <cli_name> # ex. cargo binstall hadolint-sarif
```

### Nix

Through the `nix` cli,

```shell
nix --accept-flake-config profile install github:psastras/sarif-rs#hadolint-sarif
```

## Usage

For most cases, simply run `hadolint` with `json` output and pipe the results
into `hadolint-sarif`.

## Example

```shell
hadolint -f json Dockerfile | hadolint-sarif
```

If you are using Github Actions, SARIF is useful for integrating with Github
Advanced Security (GHAS), which can show code alerts in the "Security" tab of
your repository.

After uploading `hadolint-sarif` output to Github, `hadolint` diagnostics are
available in GHAS.

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
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo install hadolint-sarif sarif-fmt
      - run: hadolint -f json Dockerfile | hadolint-sarif | tee results.sarif |
          sarif-fmt
      - name: Upload SARIF file
        uses: github/codeql-action/upload-sarif@v4
        with:
          sarif_file: results.sarif
```

License: MIT
