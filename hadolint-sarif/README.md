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

`hadolint-sarif` may be insalled via `cargo`

```shell
cargo install hadolint-sarif
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
your respository.

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
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          profile: minimal
          toolchain: stable
          override: true
      - uses: Swatinem/rust-cache@v1
      - run: cargo install hadolint-sarif sarif-fmt
      - run:
          hadolint -f json Dockerfile | hadolint-sarif | tee results.sarif |
          sarif-fmt
      - name: Upload SARIF file
        uses: github/codeql-action/upload-sarif@v1
        with:
          sarif_file: results.sarif
```

License: MIT
