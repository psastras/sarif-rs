[![Workflow Status](https://github.com/psastras/sarif-rs/workflows/main/badge.svg)](https://github.com/psastras/sarif-rs/actions?query=workflow%3A%22main%22)

This crate provides a command line tool to convert `clang-tidy` diagnostic
output into SARIF.

The latest [documentation can be found here](https://docs.rs/clang_tidy_sarif).

clang-tidy is a popular linter / static analysis tool for C++. More information
can be found on the official page:
[https://clang.llvm.org/extra/clang-tidy/](https://ng.llvm.org/extra/clang-tidy/)

SARIF or the Static Analysis Results Interchange Format is an industry standard
format for the output of static analysis tools. More information can be found on
the official website:
[https://sarifweb.azurewebsites.net/](https://ifweb.azurewebsites.net/).

## Installation

`clang-tidy-sarif` may be insalled via `cargo`

```shell
cargo install clang-tidy-sarif
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
your respository.

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
      - run: cargo install clang-tidy-sarif
      - run:
          clang-tidy -checks=cert-* -warnings-as-errors=* main.cpp -- |
          clang-tidy-sarif > results.sarif
      - name: Upload SARIF file
        uses: github/codeql-action/upload-sarif@v1
        with:
          sarif_file: results.sarif
```

License: MIT
