# deny-sarif

This crate provides a command line tool to convert `cargo-deny` diagnostic
output into SARIF.

The latest [documentation can be found here](https://docs.rs/deny_sarif).

cargo-deny is a popular linter / static analysis tool for Rust dependencies. More information
can be found on the official repository: [https://github.com/EmbarkStudios/cargo-deny](https://github.com/EmbarkStudios/cargo-deny)

SARIF or the Static Analysis Results Interchange Format is an industry
standard format for the output of static analysis tools. More information
can be found on the official website: [https://sarifweb.azurewebsites.net/](https://sarifweb.azurewebsites.net/).

## Installation

`deny-sarif` may be installed via `cargo`

```shell
cargo install deny-sarif
```

or downloaded directly from Github Releases

```shell
# make sure to adjust the target and version (you may also want to pin to a specific version)
curl -sSL https://github.com/psastras/sarif-rs/releases/download/deny-sarif-latest/deny-sarif-x86_64-unknown-linux-gnu -o deny-sarif
```

## Usage

For most cases, simply run `cargo deny` with `json` output and pipe the
results into `deny-sarif`.

### Example

```shell
cargo deny --format json check | deny-sarif
```

## Github Actions

If you are using Github Actions, SARIF is useful for integrating with
Github Advanced Security (GHAS), which can show code alerts in the
"Security" tab of your repository.

After uploading `deny-sarif` output to Github, `cargo-deny` diagnostics
are available in GHAS.

### Example

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
      - run: cargo install deny-sarif sarif-fmt
      - run:
          cargo deny --format json check |
          deny-sarif | tee results.sarif | sarif-fmt
      - name: Upload SARIF file
        uses: github/codeql-action/upload-sarif@v1
        with:
          sarif_file: results.sarif
```

License: MIT