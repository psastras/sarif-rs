[![Workflow Status](https://github.com/psastras/sarif-rs/workflows/main/badge.svg)](https://github.com/psastras/sarif-rs/actions?query=workflow%3A%22main%22)
[![OpenSSF Scorecard](https://api.scorecard.dev/projects/github.com/psastras/sarif-rs/badge)](https://scorecard.dev/viewer/?uri=github.com/psastras/sarif-rs)
[![OpenSSF Best Practices](https://www.bestpractices.dev/projects/9282/badge)](https://www.bestpractices.dev/projects/9282)

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

Each CLI may be installed via `cargo`, [cargo-binstall](https://github.com/cargo-bins/cargo-binstall) or directly downloaded from the
corresponding Github release.

### Cargo

```shell
cargo install <cli_name> # ex. cargo install sarif-fmt
```

### Cargo-binstall

```shell
cargo binstall <cli_name> # ex. cargo binstall sarif-fmt
```

### Github Releases

The latest version is
[continuously published and tagged](https://github.com/psastras/sarif-rs/releases).

Using `curl`,

```shell
# make sure to adjust the target and version (you may also want to pin to a specific version)
curl -sSL https://github.com/psastras/sarif-rs/releases/download/shellcheck-sarif-v0.8.0/shellcheck-sarif-x86_64-unknown-linux-gnu -o shellcheck-sarif
```

### Fedora Linux

```shell
sudo dnf install <cli_name> # ex. cargo binstall sarif-fmt
```

### Nix

Through the `nix` cli,

```shell
nix --accept-flake-config profile install github:psastras/sarif-rs
```

Or [from FlakeHub](https://flakehub.com/flake/psastras/sarif-rs).

## Documentation

See each subproject for more detailed information:

- `clang-tidy-sarif`: CLI tool to convert `clang-tidy` diagnostics into SARIF.
  See the [Rust documentation](https://docs.rs/clang_tidy_sarif/).
- `clippy-sarif`: CLI tool to convert `clippy` diagnostics into SARIF. See the
  [Rust documentation](https://docs.rs/clippy_sarif/).
- `hadolint-sarif`: CLI tool to convert `hadolint` diagnostics into SARIF. See
  the [Rust documentation](https://docs.rs/hadolint_sarif/).
- `miri-sarif`: CLI tool to convert `miri` diagnostics into SARIF. See the
  [Rust documentation](https://docs.rs/miri_sarif/).
- `shellcheck-sarif`: CLI tool to convert `shellcheck` diagnostics into SARIF.
  See the [Rust documentation](https://docs.rs/shellcheck_sarif/).
- `sarif-fmt`: CLI tool to pretty print SARIF diagnostics. See the
  [Rust documentation](https://docs.rs/sarif_fmt/).
- `serde-sarif`: Typesafe SARIF structures for serializing and deserializing
  SARIF information using [serde](https://serde.rs/). See the
  [Rust documentation](https://docs.rs/serde_sarif/).

## Development

Before you begin, ensure the following programs are available on your machine:

- [`nix`](https://nixos.org/download.html#nix-quick-install)

### Using Cargo

Enter the development shell provisioned by `nix` and build / test the project:

```shell
nix develop
cargo build
cargo test
```

For more information on specific configurations, refer to the
[`cargo` documentation](https://doc.rust-lang.org/cargo).

### Using Nix

Enter the development shell provisioned by `nix` and build / test the project:

```shell
# build all crates
nix build

# optionally, you may build a single crate
nix build ".#sarif-fmt"
```

### Releasing

To release a new version (publish to crates.io), prefix the head commit with `release:` and update the relevant rust crate versions. Once merged into main the pipeline should pick up the change and publish a new version.

License: MIT
