[![Workflow Status](https://github.com/psastras/sarif-rs/workflows/main/badge.svg)](https://github.com/psastras/sarif-rs/actions?query=workflow%3A%22main%22)

# sarif-fmt

This crate provides a command line tool to pretty print SARIF files to easy
human readable output.

The latest [documentation can be found here](https://docs.rs/sarif_fmt).

SARIF or the Static Analysis Results Interchange Format is an industry standard
format for the output of static analysis tools. More information can be found on
the official website:
[https://sarifweb.azurewebsites.net/](https://sarifweb.azurewebsites.net/).

## Installation

`sarif-fmt` may be insalled via `cargo`

```shell
cargo install sarif-fmt
```

## Usage

For most cases, simply pipe a SARIF file into `sarif-fmt`
(`cat ./foo.sarif | sarif-fmt`)

## Example

```shell
$ cargo clippy --message-format=json | clippy-sarif | sarif-fmt
$ warning: using `Option.and_then(|x| Some(y))`, which is more succinctly expressed as `map(|x| y)`
    ┌─ sarif-fmt/src/bin.rs:423:13
    │
423 │ ╭             the_rule
424 │ │               .full_description
425 │ │               .as_ref()
426 │ │               .and_then(|mfms| Some(mfms.text.clone()))
    │ ╰───────────────────────────────────────────────────────^
    │
    = `#[warn(clippy::bind_instead_of_map)]` on by default
      for further information visit https://rust-lang.github.io/rust-clippy/master#bind_instead_of_map
```

License: MIT
