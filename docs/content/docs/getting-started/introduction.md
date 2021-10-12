+++
title = "Introduction"
description = "sarif-rs is group of Rust projects (CLI and libraries) for interacting with the SARIF format."
date = 2021-05-01T08:00:00+00:00
updated = 2021-05-01T08:00:00+00:00
draft = false
weight = 10
sort_by = "weight"
template = "docs/page.html"

[extra]
lead = '<b>sarif-rs</b> is group of Rust projects (CLI and libraries) for interacting with the <a href="https://sarifweb.azurewebsites.net/">SARIF</a> format.'
toc = true
top = false
+++

## Examples

Parse `cargo clippy` output, convert to SARIF (`clippy-sarif`) and save the
file, then pretty print the SARIF to terminal (`sarif-fmt`).

```sh
$ cargo clippy --message-format=json | clippy-sarif | tee results.sarif | sarif-fmt
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

![alt text](images/ghas.png "Title")

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

## Provided Tools

Below is a list of libraries and tools which are part of the `sarif-rs` project:

- `clang-tidy-sarif`: CLI tool to convert `clang-tidy` diagnostics into SARIF.
- `clippy-sarif`: CLI tool to convert `clippy` diagnostics into SARIF.
- `hadolint-sarif`: CLI tool to convert `hadolint` diagnostics into SARIF.
- `shellcheck-sarif`: CLI tool to convert `shellcheck` diagnostics into SARIF.
- `sarif-fmt`: CLI tool to pretty print SARIF diagnostics.
- `serde-sarif`: Typesafe SARIF structures for serializing and deserializing
  SARIF information using [serde](https://serde.rs/).
