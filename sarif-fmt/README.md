[![Workflow Status](https://github.com/psastras/sarif-rs/workflows/main/badge.svg)](https://github.com/psastras/sarif-rs/actions?query=workflow%3A%22main%22)

# sarif-fmt

# DO NOT USE (EARLY IMPLEMENTATION)

This crate provides a command line tool to format SARIF files to pretty
printed text.

The latest [documentation can be found here](https://psastras.github.io/sarif-rs/sarif_fmt/index.html).

SARIF or the Static Analysis Results Interchange Format is an industry
standard format for the output of static analysis tools. More information
can be found on the official website: [https://sarifweb.azurewebsites.net/](https://sarifweb.azurewebsites.net/).

## Installation

`sarif-fmt` may be insalled via `cargo`

```shell
cargo install sarif-fmt
```

## Usage

For most cases, simply pipe a SARIF file into `sarif-fmt`

## Example

```shell
cargo clippy --message-format=json | clippy-sarif | sarif-fmt
```


License: MIT