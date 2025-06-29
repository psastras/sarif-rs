[![Workflow Status](https://github.com/psastras/sarif-rs/workflows/main/badge.svg)](https://github.com/psastras/sarif-rs/actions?query=workflow%3A%22main%22)

# clippy-sarif

This crate provides a command line tool to convert `cargo clippy` diagnostic
output into SARIF.

The latest [documentation can be found here](https://docs.rs/clippy_sarif).

clippy is a popular linter / static analysis tool for rust. More information can
be found on the official repository:
[https://github.com/rust-lang/rust-clippy](https://github.com/rust-lang/rust-clippy)

SARIF or the Static Analysis Results Interchange Format is an industry standard
format for the output of static analysis tools. More information can be found on
the official website:
[https://sarifweb.azurewebsites.net/](https://sarifweb.azurewebsites.net/).

## Installation

`clippy-sarif` may be installed via `cargo`

```shell
cargo install clippy-sarif
```

via [cargo-binstall](https://github.com/cargo-bins/cargo-binstall)

```shell
cargo binstall clippy-sarif
```

or downloaded directly from Github Releases

```shell
# make sure to adjust the target and version (you may also want to pin to a specific version)
curl -sSL https://github.com/psastras/sarif-rs/releases/download/clippy-sarif-v0.8.0/clippy-sarif-x86_64-unknown-linux-gnu -o clippy-sarif
```

### Fedora Linux

```shell
sudo dnf install <cli_name> # ex. cargo binstall clippy-sarif
```

### Nix

Through the `nix` cli,

```shell
nix --accept-flake-config profile install github:psastras/sarif-rs#clippy-sarif
```

## Usage

For most cases, simply run `cargo clippy` with `json` output and pipe the
results into `clippy-sarif`.

## Example

```shell
cargo clippy --message-format=json | clippy-sarif
```

If you are using Github Actions, SARIF is useful for integrating with Github
Advanced Security (GHAS), which can show code alerts in the "Security" tab of
your repository.

After uploading `clippy-sarif` output to Github, `clippy` diagnostics are
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
          components: clippy,rustfmt
      - uses: Swatinem/rust-cache@v2
      - run: cargo install clippy-sarif sarif-fmt
      - run: cargo clippy --all-targets --all-features --message-format=json |
          clippy-sarif | tee results.sarif | sarif-fmt
      - name: Upload SARIF file
        uses: github/codeql-action/upload-sarif@v4
        with:
          sarif_file: results.sarif
```

In some cases, the path to the file contained in the SARIF report [may be different than what is expected](https://github.com/psastras/sarif-rs/issues/370). This can happen for example if running `clippy-sarif` from a different folder than the crate folder. In this case consider using a tool like `jq` to amend to path:

## Example

```bash
cat results.sarif \
    | jq --arg pwd "some_folder/my_crate" '.runs[].results[].locations[].physicalLocation.artifactLocation.uri |= $pwd + "/" + .' \
    > results.sarif.tmp
```

Note that this maybe be fixed in a future release.

License: MIT
