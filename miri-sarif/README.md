[![Workflow Status](https://github.com/psastras/sarif-rs/workflows/main/badge.svg)](https://github.com/psastras/sarif-rs/actions?query=workflow%3A%22main%22)

# miri-sarif

This crate provides a command line tool to convert `cargo miri` diagnostic
output into SARIF.

The latest [documentation can be found here](https://docs.rs/miri_sarif).

Miri is an undefined behavior detection tool for rust. More information can
be found on the official repository:
[https://github.com/rust-lang/miri](https://github.com/rust-lang/miri)

SARIF or the Static Analysis Results Interchange Format is an industry standard
format for the output of static analysis tools. More information can be found on
the official website:
[https://sarifweb.azurewebsites.net/](https://sarifweb.azurewebsites.net/).

## Installation

`miri-sarif` may be installed via `cargo`

```shell
cargo install miri-sarif
```

via [cargo-binstall](https://github.com/cargo-bins/cargo-binstall)

```shell
cargo binstall miri-sarif
```

or downloaded directly from Github Releases

```shell
# make sure to adjust the target and version (you may also want to pin to a specific version)
curl -sSL https://github.com/psastras/sarif-rs/releases/download/miri-sarif-v0.8.0/miri-sarif-x86_64-unknown-linux-gnu -o miri-sarif
```

### Fedora Linux

```shell
sudo dnf install <cli_name> # ex. cargo binstall miri-sarif
```

### Nix

Through the `nix` cli,

```shell
nix --accept-flake-config profile install github:psastras/sarif-rs#miri-sarif
```

## Usage

For miri to output machine readable data you need to pass `--error-format=json` in the `MIRIFLAGS` environment variable.

### `cargo miri test` & `cargo miri run`

Because the relevant miri output is printed to stderr you will need to redirect
stderr to stdout and stdout to `/dev/null`.

#### Example

```shell
MIRIFLAGS="--error-format=json" cargo miri test 2>&1 1>/dev/null | miri-sarif
```

### `cargo miri nextest`

Since `nextest` only outputs to stderr, you don't need to redirect stdout to `/dev/null`. \
But you should use `--success-output immediate` to also capture warnings produced by miri. \
Additionally you can use `--no-fail-fast` for miri to run all tests and not stop on the first failure.

#### Example

```shell
MIRIFLAGS="--error-format=json" cargo miri nextest --no-fail-fast --success-output immediate 2>&1 | miri-sarif
```

## Github Actions

If you are using Github Actions, SARIF is useful for integrating with Github
Advanced Security (GHAS), which can show code alerts in the "Security" tab of
your repository.

After uploading `miri-sarif` output to Github, `miri` diagnostics are
available in GHAS.

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
          toolchain: nightly
          components: miri
          override: true
      - uses: Swatinem/rust-cache@v1
      - run: cargo install miri-sarif sarif-fmt cargo-nextest
      - run: MIRIFLAGS="--error-format=json" cargo miri nextest run --no-fail-fast --success-output immediate 2>&1 |
          miri-sarif | tee results.sarif | sarif-fmt
      - name: Upload SARIF file
        uses: github/codeql-action/upload-sarif@v3
        with:
          sarif_file: results.sarif
```

In some cases, the path to the file contained in the SARIF report [may be different than what is expected](https://github.com/psastras/sarif-rs/issues/370). This can happen for example if running `miri-sarif` from a different folder than the crate folder. In this case consider using a tool like `jq` to amend to path:

### Example

```bash
cat results.sarif \
    | jq --arg pwd "some_folder/my_crate" '.runs[].results[].locations[].physicalLocation.artifactLocation.uri |= $pwd + "/" + .' \
    > results.sarif.tmp
```

Note that this maybe be fixed in a future release.

License: MIT
