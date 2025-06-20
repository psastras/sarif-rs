#![doc(html_root_url = "https://docs.rs/clippy-sarif/0.8.0")]

//! This crate provides a command line tool to convert `cargo clippy` diagnostic
//! output into SARIF.
//!
//! The latest [documentation can be found here](https://docs.rs/clippy_sarif).
//!
//! clippy is a popular linter / static analysis tool for rust. More information
//! can be found on the official repository: [https://github.com/rust-lang/rust-clippy](https://github.com/rust-lang/rust-clippy)
//!
//! SARIF or the Static Analysis Results Interchange Format is an industry
//! standard format for the output of static analysis tools. More information
//! can be found on the official website: [https://sarifweb.azurewebsites.net/](https://sarifweb.azurewebsites.net/).
//!
//! ## Installation
//!
//! `clippy-sarif` may be installed via `cargo`
//!
//! ```shell
//! cargo install clippy-sarif
//! ```
//!
//! or downloaded directly from Github Releases
//!
//!```shell
//! # make sure to adjust the target and version (you may also want to pin to a specific version)
//! curl -sSL https://github.com/psastras/sarif-rs/releases/download/clippy-sarif-latest/clippy-sarif-x86_64-unknown-linux-gnu -o clippy-sarif
//! ```
//!
//! ## Usage
//!
//! For most cases, simply run `cargo clippy` with `json` output and pipe the
//! results into `clippy-sarif`.
//!
//! ## Example
//!
//!```shell
//! cargo clippy --message-format=json | clippy-sarif
//! ```
//!
//! If you are using Github Actions, SARIF is useful for integrating with
//! Github Advanced Security (GHAS), which can show code alerts in the
//! "Security" tab of your repository.
//!
//! After uploading `clippy-sarif` output to Github, `clippy` diagnostics
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
//!       - uses: actions/checkout@v4
//!       - uses: dtolnay/rust-toolchain@stable
//!         with:
//!           toolchain: stable
//!           components: clippy,rustfmt
//!       - uses: Swatinem/rust-cache@v2
//!       - run: cargo install clippy-sarif sarif-fmt
//!       - run:
//!           cargo clippy --all-targets --all-features --message-format=json |
//!           clippy-sarif | tee results.sarif | sarif-fmt
//!       - name: Upload SARIF file
//!         uses: github/codeql-action/upload-sarif@v4
//!         with:
//!           sarif_file: results.sarif
//! ```
//!

use anyhow::Result;
use clap::Parser;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};

#[derive(Parser, Debug)]
#[command(
  version,
  about = "Convert clippy output into SARIF",
  after_help = "The expected input is generated by running 'cargo clippy --message-format=json'.",
  long_about = None,
)]
struct Args {
  /// input file; reads from stdin if none is given
  #[arg(short, long)]
  input: Option<std::path::PathBuf>,
  /// output file; writes to stdout if none is given
  #[arg(short, long)]
  output: Option<std::path::PathBuf>,
}

fn main() -> Result<()> {
  let args = Args::parse();

  let read = match args.input {
    Some(path) => Box::new(File::open(path)?) as Box<dyn Read>,
    None => Box::new(std::io::stdin()) as Box<dyn Read>,
  };
  let reader = BufReader::new(read);

  let write = match args.output {
    Some(path) => Box::new(File::create(path)?) as Box<dyn Write>,
    None => Box::new(std::io::stdout()) as Box<dyn Write>,
  };
  let writer = BufWriter::new(write);

  serde_sarif::converters::clippy::parse_to_writer(reader, writer)
}
