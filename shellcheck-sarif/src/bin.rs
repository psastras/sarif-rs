#![doc(html_root_url = "https://docs.rs/shellcheck-sarif/0.6.4")]

//! This crate provides a command line tool to convert `shellcheck` diagnostic
//! output into SARIF.
//!
//! The latest [documentation can be found here](https://docs.rs/shellcheck_sarif).
//!
//! shellcheck is a popular linter / static analysis tool for shell scripts. More information
//! can be found on the official repository: [https://github.com/koalaman/shellcheck](https://github.com/koalaman/shellcheck)
//!
//! SARIF or the Static Analysis Results Interchange Format is an industry
//! standard format for the output of static analysis tools. More information
//! can be found on the official website: [https://sarifweb.azurewebsites.net/](https://sarifweb.azurewebsites.net/).
//!
//! ## Installation
//!
//! `shellcheck-sarif` may be installed via `cargo`
//!
//! ```shell
//! cargo install shellcheck-sarif
//! ```
//!
//! or downloaded directly from Github Releases
//!
//!```shell
//! # make sure to adjust the target and version (you may also want to pin to a specific version)
//! curl -sSL https://github.com/psastras/sarif-rs/releases/download/shellcheck-sarif-latest/shellcheck-sarif-x86_64-unknown-linux-gnu -o shellcheck-sarif
//! ```
//!
//! ## Usage
//!
//! For most cases, simply run `shellcheck` with `json` output and pipe the
//! results into `shellcheck-sarif`.
//!
//! ## Example
//!
//!```shell
//! shellcheck -f json shellscript.sh | shellcheck-sarif
//! ```
//!
//! If you are using Github Actions, SARIF is useful for integrating with
//! Github Advanced Security (GHAS), which can show code alerts in the
//! "Security" tab of your repository.
//!
//! After uploading `shellcheck-sarif` output to Github, `shellcheck` diagnostics
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
//!       - run: cargo install shellcheck-sarif sarif-fmt
//!       - run:
//!           shellcheck -f json shellscript.sh |
//!           shellcheck-sarif | tee results.sarif | sarif-fmt
//!       - name: Upload SARIF file
//!         uses: github/codeql-action/upload-sarif@v1
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
  about = "Convert shellcheck warnings into SARIF",
  after_help = "The expected input is generated by running 'shellcheck -f json'.",
  long_about = None,
)]
struct Args {
  /// input file; reads from stdin if none is given
  #[arg(short, long)]
  input: Option<std::path::PathBuf>,
  /// input format; json or json1; defaults to 'json'
  #[arg(short, long, default_value = "json")]
  format: Option<String>,
  
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

  let format = match args.format {
    Some(format) => format,
    None => "json".to_string(),
  };

  let write = match args.output {
    Some(path) => Box::new(File::create(path)?) as Box<dyn Write>,
    None => Box::new(std::io::stdout()) as Box<dyn Write>,
  };
  let writer = BufWriter::new(write);

  serde_sarif::converters::shellcheck::parse_to_writer(reader, writer, format)
}
