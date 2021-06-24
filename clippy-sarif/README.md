[![Workflow Status](https://github.com/psastras/sarif-rs/workflows/main/badge.svg)](https://github.com/psastras/sarif-rs/actions?query=workflow%3A%22main%22)

# clippy-sarif

This crate provides a command line tool to convert `cargo clippy` diagnostic
output into SARIF.

The latest [documentation can be found here](https://psastras.github.io/sarif-rs/clippy_sarif/index.html).

clippy is a popular linter / static analysis tool for rust. More information
can be found on the official repository: [https://github.com/rust-lang/rust-clippy](https://github.com/rust-lang/rust-clippy)

SARIF or the Static Analysis Results Interchange Format is an industry
standard format for the output of static analysis tools. More information
can be found on the official website: [https://sarifweb.azurewebsites.net/](https://sarifweb.azurewebsites.net/).

## Installation

`clippy-sarif` may be insalled via `cargo`

```shell
cargo install clippy-sarif
```

## Usage

For most cases, simply run `cargo clippy` with `json` output and pipe the
results into `clippy-sarif`.

## Example

```shell
cargo clippy --message-format=json | clippy-sarif
```

If you are using Github Actions, SARIF is useful for integrating with
Github Advanced Security (GHAS), which can show code alerts in the
"Security" tab of your respository.

After uploading `cargo-clippy` output to Github, `clippy` diagnostics
are available in GHAS.
[You can see a demo in this repository](https://github.com/psastras/sarif-rs/security/code-scanning).

## Example

```yaml
on:
  workflow_run:
  workflows: ["main"]
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
      - run: cargo install clippy-sarif
      - run:
          cargo clippy --all-targets --all-features --message-format=json |
          clippy-sarif > results.sarif
      - name: Upload SARIF file
        uses: github/codeql-action/upload-sarif@v1
        with:
          sarif_file: results.sarif
```


License: MIT