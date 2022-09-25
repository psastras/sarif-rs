#![doc(html_root_url = "https://docs.rs/clang-tidy-sarif/0.3.4")]

//! This crate provides a command line tool to convert `clang-tidy` diagnostic
//! output into SARIF.
//!
//! The latest [documentation can be found here](https://docs.rs/clang_tidy_sarif).
//!
//! clang-tidy is a popular linter / static analysis tool for C++. More information
//! can be found on the official page: [https://clang.llvm.org/extra/clang-tidy/](https://clang.llvm.org/extra/clang-tidy/)
//!
//! SARIF or the Static Analysis Results Interchange Format is an industry
//! standard format for the output of static analysis tools. More information
//! can be found on the official website: [https://sarifweb.azurewebsites.net/](https://sarifweb.azurewebsites.net/).
//!
//! ## Installation
//!
//! `clang-tidy-sarif` may be installed via `cargo`
//!
//! ```shell
//! cargo install clang-tidy-sarif
//! ```
//!
//! or downloaded directly from Github Releases
//!
//!```shell
//! # make sure to adjust the target and version (you may also want to pin to a specific version)
//! curl -sSL https://github.com/psastras/sarif-rs/releases/download/clang-tidy-sarif-latest/clang-tidy-sarif-x86_64-unknown-linux-gnu -o clang-tidy-sarif
//! ```
//!
//! ## Usage
//!
//! For most cases, simply run `clang-tidy` and pipe the
//! results into `clang-tidy-sarif`.
//!
//! ## Example
//!
//!```shell
//! clang-tidy -checks=cert-* -warnings-as-errors=* main.cpp -- | clang-tidy-sarif
//! ```
//!
//! If you are using Github Actions, SARIF is useful for integrating with
//! Github Advanced Security (GHAS), which can show code alerts in the
//! "Security" tab of your repository.
//!
//! After uploading `clang-tidy-sarif` output to Github, `clang-tidy` diagnostics
//! are available in GHAS.
//!
//! ## Example
//!
//! ```yaml
//! on:
//!   workflow_run:
//!     workflows: ["main"]
//!     branches: [main]
//!     types: [completed]
//!
//! name: sarif
//!
//! jobs:
//!   upload-sarif:
//!     runs-on: ubuntu-latest
//!     if: ${{ github.ref == 'refs/heads/main' }}
//!     steps:
//!       - uses: actions/checkout@v2
//!       - uses: actions-rs/toolchain@v1
//!         with:
//!           profile: minimal
//!           toolchain: stable
//!           override: true
//!       - uses: Swatinem/rust-cache@v1
//!       - run: cargo install clang-tidy-sarif sarif-fmt
//!       - run:
//!           clang-tidy -checks=cert-* -warnings-as-errors=* main.cpp -- | tee
//!           results.sarif | sarif-fmt
//!       - name: Upload SARIF file
//!         uses: github/codeql-action/upload-sarif@v1
//!         with:
//!           sarif_file: results.sarif
//! ```
//!

use anyhow::Result;
use clap::{Arg, Command};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};

fn main() -> Result<()> {
  let matches = Command::new("clang-tidy-sarif")
    .about("Convert clang-tidy output into SARIF")
    .after_help("The expected input is generated by running 'clang-tidy'.")
    .version(env!("CARGO_PKG_VERSION"))
    .arg(
      Arg::new("input")
        .help("input file; reads from stdin if none is given")
        .takes_value(true)
        .value_parser(clap::value_parser!(std::path::PathBuf)),
    )
    .arg(
      Arg::new("output")
        .help("output file; writes to stdout if none is given")
        .short('o')
        .long("output")
        .takes_value(true)
        .value_parser(clap::value_parser!(std::path::PathBuf)),
    )
    .get_matches();

  let read = match matches.get_one::<std::path::PathBuf>("input") {
    Some(path) => Box::new(File::open(path)?) as Box<dyn Read>,
    None => Box::new(std::io::stdin()) as Box<dyn Read>,
  };
  let reader = BufReader::new(read);

  let write = match matches.get_one::<std::path::PathBuf>("output") {
    Some(path) => Box::new(File::create(path)?) as Box<dyn Write>,
    None => Box::new(std::io::stdout()) as Box<dyn Write>,
  };
  let writer = BufWriter::new(write);

  serde_sarif::converters::clang_tidy::parse_to_writer(reader, writer)
}
