#![doc(html_root_url = "https://docs.rs/sarif-fmt/0.2.4")]

//! # DO NOT USE (EARLY IMPLEMENTATION)
//!
//! This crate provides a command line tool to format SARIF files to pretty
//! printed text.
//!
//! The latest [documentation can be found here](https://psastras.github.io/sarif-rs/sarif_fmt/index.html).
//!
//! SARIF or the Static Analysis Results Interchange Format is an industry
//! standard format for the output of static analysis tools. More information
//! can be found on the official website: [https://sarifweb.azurewebsites.net/](https://sarifweb.azurewebsites.net/).
//!
//! ## Installation
//!
//! `sarif-fmt` may be insalled via `cargo`
//!
//! ```shell
//! cargo install sarif-fmt
//! ```
//!
//! ## Usage
//!
//! For most cases, simply pipe a SARIF file into `sarif-fmt`
//!
//! ## Example
//!
//!```shell
//! cargo clippy --message-format=json | clippy-sarif | sarif-fmt
//! ```
//!

use anyhow::Result;
use clap::{App, Arg};
use codespan_reporting::diagnostic::Diagnostic;
use codespan_reporting::diagnostic::Label;
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::ColorChoice;
use codespan_reporting::term::termcolor::StandardStream;
use serde_sarif::sarif;
use serde_sarif::sarif::ResultLevel;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};
use std::path::Path;
use std::str::FromStr;
use std::usize;

fn process<R: BufRead>(reader: R) -> Result<sarif::Sarif> {
  let s: sarif::Sarif = serde_json::from_reader(reader)?;
  Ok(s)
}

/// Returns [sarif::Sarif] serialized into a JSON stream
///
/// # Arguments
///
/// * `reader` - A `BufRead` of cargo output
/// * `writer` - A `Writer` to write the results to
pub fn parse_to_writer<R: BufRead, W: Write>(
  reader: R,
  writer: W,
) -> Result<()> {
  let sarif = process(reader)?;
  to_writer_pretty(writer, &sarif)?;
  Ok(())
}

fn to_writer_pretty<W>(_: W, sarif: &sarif::Sarif) -> Result<()>
where
  W: io::Write,
{
  let writer = StandardStream::stderr(ColorChoice::Always);

  let mut files = SimpleFiles::new();
  let config = codespan_reporting::term::Config::default();

  sarif.runs.iter().try_for_each(|run| -> Result<()> {
    if let Some(results) = run.results.as_ref() {
      results.iter().try_for_each(|result| -> Result<()> {
        // attempt to fetch text
        let message = result.message.text.as_ref().cloned();

        // attempt to fetch files
        if let Some(locations) = result.locations.as_ref() {
          locations.iter().try_for_each(|location| -> Result<()> {
            if let Some(physical_location) = location.physical_location.as_ref()
            {
              if let Some(artifact_location) =
                physical_location.artifact_location.as_ref()
              {
                if let Some(uri) = artifact_location.uri.as_ref() {
                  let path = Path::new(uri);
                  if path.exists() {
                    let mut file = File::open(uri)?;
                    let mut contents = String::new();
                    file.read_to_string(&mut contents)?;
                    let file_id = files.add(uri, contents);
                    if let Some(region) = physical_location.region.as_ref() {
                      if let (Some(byte_start), Some(byte_length)) =
                        (region.byte_offset, region.byte_length)
                      {
                        let range_start = byte_start as usize;
                        let range_end = (byte_start + byte_length) as usize;
                        let diagnostic = if let Some(level) =
                          result.level.as_ref()
                        {
                          let level = ResultLevel::from_str(
                            level.as_str().unwrap_or("note"),
                          )?;
                          match level {
                            ResultLevel::Warning => {
                              Diagnostic::<usize>::warning()
                            }
                            ResultLevel::Error => Diagnostic::<usize>::error(),
                            ResultLevel::Note => Diagnostic::<usize>::note(),
                            _ => Diagnostic::error(),
                          }
                        } else {
                          Diagnostic::note()
                        };

                        let mut diagnostic = diagnostic
                          .with_message(
                            message.as_ref().unwrap_or(&"".to_string()).clone(),
                          )
                          .with_labels(vec![Label::primary(
                            file_id,
                            range_start..range_end,
                          )]);

                        if let Some(rule_index) = result.rule_index {
                          if let Some(rules) = run.tool.driver.rules.as_ref() {
                            if let Some(rule) = rules.get(rule_index as usize) {
                              if let Some(full_description) =
                                rule.full_description.as_ref()
                              {
                                diagnostic =
                                  diagnostic.with_notes(vec![full_description
                                    .text
                                    .clone()])
                              } else if let Some(short_description) =
                                rule.short_description.as_ref()
                              {
                                diagnostic =
                                  diagnostic.with_notes(vec![short_description
                                    .text
                                    .clone()])
                              }
                            }
                          }
                        }

                        term::emit(
                          &mut writer.lock(),
                          &config,
                          &files,
                          &diagnostic,
                        )?;
                      } else if let (
                        Some(_start_line),
                        Some(_start_column),
                        Some(_end_line),
                        Some(_end_column),
                      ) = (
                        region.start_line,
                        region.start_column,
                        region.end_line,
                        region.end_column,
                      ) {
                        todo!()
                      }
                    }
                  }
                }
              }
            }
            Ok(())
          })?
        }

        Ok(())
      })?;
    };
    Ok(())
  })?;

  Ok(())
}

fn main() -> Result<()> {
  let matches = App::new("clippy-sarif")
        .about("Convert clippy warnings into SARIF")
        .after_help(
            "The expected input is generated by running 'cargo clippy --message-format=json'.",
        )
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::new("input")
                .about("input file; reads from stdin if none is given")
                .takes_value(true),
        )
        .arg(
            Arg::new("output")
                .about("output file; writes to stdout if none is given")
                .short('o')
                .long("output")
                .takes_value(true),
        )
        .get_matches();

  let read = match matches.value_of_os("input").map(Path::new) {
    Some(path) => Box::new(File::open(path)?) as Box<dyn Read>,
    None => Box::new(std::io::stdin()) as Box<dyn Read>,
  };
  let reader = BufReader::new(read);

  let write = match matches.value_of_os("output").map(Path::new) {
    Some(path) => Box::new(File::create(path)?) as Box<dyn Write>,
    None => Box::new(std::io::stdout()) as Box<dyn Write>,
  };
  let writer = BufWriter::new(write);

  parse_to_writer(reader, writer)
}
