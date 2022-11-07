[![Workflow Status](https://github.com/psastras/sarif-rs/workflows/main/badge.svg)](https://github.com/psastras/sarif-rs/actions?query=workflow%3A%22main%22)

# cppcheck-sarif

This crate provides a command line tool to convert `cppcheck` diagnostic
output into SARIF.

The latest [documentation can be found here](https://docs.rs/cppcheck_sarif).

cppcheck is a popular linter / static analysis tool for C++. More information
can be found on the official page:
[https://cppcheck.sourceforge.io/](https://cppcheck.sourceforge.io/)

SARIF or the Static Analysis Results Interchange Format is an industry standard
format for the output of static analysis tools. More information can be found on
the official website:
[https://sarifweb.azurewebsites.net/](https://ifweb.azurewebsites.net/).

## Installation

`cppcheck` may be installed via `cargo`

```shell
cargo install cppcheck-sarif
```

via [cargo-binstall](https://github.com/cargo-bins/cargo-binstall)

```shell
cargo binstall cppcheck-sarif
```

or downloaded directly from Github Releases

```shell
# make sure to adjust the target and version (you may also want to pin to a specific version)
curl -sSL https://github.com/psastras/sarif-rs/releases/download/cppcheck-sarif-latest/cppcheck-sarif-x86_64-unknown-linux-gnu -o cppcheck-sarif
```

## Usage

For most cases, simply run `cppcheck` and pipe the results into
`cppcheck-sarif`.

## Example

```shell
 cppcheck -checks=cert-* -warnings-as-errors=* main.cpp -- | cppcheck-sarif
```

If you are using Github Actions, SARIF is useful for integrating with Github
Advanced Security (GHAS), which can show code alerts in the "Security" tab of
your repository.

After uploading `cppcheck-sarif` output to Github, `cppcheck` diagnostics
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
      - run: cargo install cppcheck-sarif sarif-fmt
      - run: cppcheck main.cpp 2>&1 | cppcheck-sarif | tee results.sarif | sarif-fmt
      - name: Upload SARIF file
        uses: github/codeql-action/upload-sarif@v1
        with:
          sarif_file: results.sarif
```

License: MIT
