[![Workflow Status](https://github.com/psastras/sarif-rs/workflows/main/badge.svg)](https://github.com/psastras/sarif-rs/actions?query=workflow%3A%22main%22)
[![codecov](https://codecov.io/gh/psastras/sarif-rs/branch/main/graph/badge.svg?token=KSXYAZGS5U)](https://codecov.io/gh/psastras/sarif-rs)

# sarif-rs

A group of Rust projects for interacting with the
[SARIF](https://sarifweb.azurewebsites.net/) format.

## Example

Parse `cargo clippy` output, convert to SARIF (`clippy-sarif`), then pretty
print the SARIF to terminal (`sarif-fmt`).

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

## Install

Each CLI may be installed via `cargo` or directly downloaded from the
corresponding Github release.

### Cargo

```shell
cargo install <cli_name> # ex. cargo install sarif-fmt
```

### Github Releases

The latest version is
[continuously published and tagged](https://github.com/psastras/sarif-rs/releases).

Using `curl`,

```shell
curl -sSL https://github.com/psastras/sarif-rs/releases/download/latest-x86_64-unknown-linux-gnu/sarif-fmt # make sure to adjust the target triplet (latest-<target_triplet>) to the correct target
```

## Documentation

See each subproject for more detailed information:

- `clippy-sarif`: CLI tool to convert `clippy` diagnostics into SARIF. See the
  [Rust documentation](https://docs.rs/clippy_sarif/).
- `hadolint-sarif`: CLI tool to convert `hadolint` diagnostics into SARIF. See
  the [Rust documentation](https://docs.rs/hadolint_sarif/).
- `shellcheck-sarif`: CLI tool to convert `shellcheck` diagnostics into SARIF.
  See the [Rust documentation](https://docs.rs/shellcheck_sarif/).
- `sarif-fmt`: CLI tool to pretty print SARIF diagnostics. See the
  [Rust documentation](https://docs.rs/sarif_fmt/).
- `serde-sarif`: Typesafe SARIF structures for serializing and deserializing
  SARIF information using [serde](https://serde.rs/). See the
  [Rust documentation](https://docs.rs/serde_sarif/).

## Development

Before you begin, ensure the following programs are available on your machine:

- [`cargo`](https://rustup.rs/)
- [`nix`](https://nixos.org/download.html#nix-quick-install)

Assuming `cargo` is installed on your machine, the standard `cargo` commands can
be run to build and test all projects in the workspace:

```shell
cargo build
cargo test
```

For more information on specific configurations, refer to the
[`cargo` documentation](https://doc.rust-lang.org/cargo).

`nix` is used internally (ie. via test fixtures) to manage other dependencies
(so you don't have to manage them yourself.)

License: MIT
