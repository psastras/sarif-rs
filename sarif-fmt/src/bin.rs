#![doc(html_root_url = "https://docs.rs/sarif-fmt/0.2.5")]
#![recursion_limit = "256"]
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
use codespan_reporting::files::Files;
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::ColorChoice;
use codespan_reporting::term::termcolor::StandardStream;
use if_chain::if_chain;
use serde_sarif::sarif;
use serde_sarif::sarif::ResultLevel;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
use std::usize;

fn process<R: BufRead>(reader: R) -> Result<sarif::Sarif> {
  let s: sarif::Sarif = serde_json::from_reader(reader)?;
  Ok(s)
}

fn try_find_file(
  physical_location: &sarif::PhysicalLocation,
  run: &sarif::Run,
) -> Result<PathBuf> {
  let artifact_location = physical_location
    .artifact_location
    .as_ref()
    .map_or_else(|| Err(anyhow::anyhow!("No artifact location.")), Ok)?;
  let uri = artifact_location
    .uri
    .as_ref()
    .map_or_else(|| Err(anyhow::anyhow!("No uri.")), Ok)?;
  let path = Path::new(uri);

  // if the path is absolute and exists, return it, else
  if path.is_absolute() {
    if path.exists() {
      return Ok(path.to_path_buf());
    }
    return Err(anyhow::anyhow!("Path does not exist"));
  }

  // if it's a relative path, it's more complicated and dictated by the SARIF spec

  // 1. check if uri base id exists -- it SHOULD exist, but may not
  if let Some(uri_base_id) = artifact_location.uri_base_id.as_ref() {
    // check if this is defined in originalUriBaseIds
    if let Some(original_uri_base_ids) = run.original_uri_base_ids.as_ref() {
      let mut path = PathBuf::new();
      if let Some(uri_base) = original_uri_base_ids.get(uri_base_id) {
        // todo: this doesn't handle recursive uri_base_id...
        if let Some(uri) = uri_base.uri.as_ref() {
          path.push(uri);
        }
      }
      path.push(uri);
      if path.exists() {
        return Ok(path);
      }
    }

    // just check if the relative path exists by chance
    if path.exists() {
      return Ok(path.to_path_buf());
    }
  }
  if path.exists() {
    Ok(path.to_path_buf())
  } else {
    Err(anyhow::anyhow!("Path not found: {:#?}", path))
  }
}

fn get_physical_location_contents(
  physical_location: &sarif::PhysicalLocation,
  run: &sarif::Run,
) -> Result<String> {
  let path = try_find_file(physical_location, run)?;
  let mut file = File::open(path)?;
  let mut contents = String::new();
  file.read_to_string(&mut contents)?;
  Ok(contents)
}

fn try_get_byte_offset(
  file_id: usize,
  files: &SimpleFiles<&String, String>,
  row: i64,
  column: i64,
) -> Result<usize> {
  files
    .line_range(file_id, row as usize - 1)?
    .find(|byte| {
      if let Ok(location) = files.location(file_id, *byte) {
        location.column_number == column as usize
      } else {
        false
      }
    })
    .ok_or_else(|| anyhow::anyhow!("Byte offset not found"))
}

fn get_byte_range(
  file_id: usize,
  files: &SimpleFiles<&String, String>,
  region: &sarif::Region,
) -> (Option<usize>, Option<usize>) {
  // todo: support character regions
  let byte_offset = if let Some(byte_offset) = region.byte_offset {
    Some(byte_offset as usize)
  } else if let (Some(start_line), Some(start_column)) =
    (region.start_line, region.start_column)
  {
    if let Ok(byte_offset) =
      try_get_byte_offset(file_id, files, start_line, start_column)
    {
      Some(byte_offset)
    } else {
      None
    }
  } else {
    None
  };

  let byte_end = if let Some(byte_offset) = byte_offset {
    if let Some(byte_length) = region.byte_length {
      Some(byte_offset + byte_length as usize)
    } else if let (Some(end_line), Some(end_column)) =
      (region.end_line, region.end_column)
    {
      if let Ok(byte_offset) =
        try_get_byte_offset(file_id, files, end_line, end_column)
      {
        Some(byte_offset)
      } else {
        Some(byte_offset)
      }
    } else {
      Some(byte_offset)
    }
  } else {
    None
  };

  (byte_offset, byte_end)
}

fn to_writer_pretty(sarif: &sarif::Sarif) -> Result<()> {
  let writer = StandardStream::stderr(ColorChoice::Always);
  let mut files = SimpleFiles::new();
  let config = codespan_reporting::term::Config::default();

  sarif.runs.iter().try_for_each(|run| -> Result<()> {
    if let Some(results) = run.results.as_ref() {
      results.iter().try_for_each(|result| -> Result<()> {
        if_chain! {
          if let Some(message) = result.message.text.as_ref().cloned();
          if let Some(locations) = result.locations.as_ref();
          then {
            locations.iter().try_for_each(|location| -> Result<()> {
              if_chain! {
                if let Some(physical_location) = location.physical_location.as_ref();
                if let Some(artifact_location) = physical_location.artifact_location.as_ref();
                if let Some(uri) = artifact_location.uri.as_ref();
                if let Some(region) = physical_location.region.as_ref();
                if let Ok(contents) = get_physical_location_contents(physical_location, run);
                let file_id = files.add(uri, contents);
                if let (Some(range_start), Some(range_end))  = get_byte_range(file_id, &files, region);
                then {
                  let mut diagnostic = diagnostic(result)?
                    .with_message(message.clone())
                    .with_labels(vec![Label::primary(
                      file_id,
                      range_start..range_end,
                    )]);
                  if_chain! {
                    if let Some(rule_index) = result.rule_index;
                    if let Some(rules) = run.tool.driver.rules.as_ref();
                    if let Some(rule) = rules.get(rule_index as usize);
                    then {
                      if let Some(full_description) = rule.full_description.as_ref() {
                        diagnostic = diagnostic.with_notes(vec![full_description.text.clone()])
                      } else if let Some(short_description) = rule.short_description.as_ref() {
                        diagnostic = diagnostic.with_notes(vec![short_description.text.clone()])
                      }
                      term::emit(&mut writer.lock(), &config, &files, &diagnostic,)?;
                    }
                  }
                }
              }
              Ok(())
            })?;
          } else {
            println!("else just printing a message");
          }
        }

        Ok(())
      })?;
    };
    Ok(())
  })?;

  Ok(())
}

fn diagnostic(
  result: &sarif::Result,
) -> Result<Diagnostic<usize>, anyhow::Error> {
  let diagnostic = if let Some(level) = result.level.as_ref() {
    let level = ResultLevel::from_str(level.as_str().unwrap_or("note"))?;
    match level {
      ResultLevel::Warning => Diagnostic::<usize>::warning(),
      ResultLevel::Error => Diagnostic::<usize>::error(),
      ResultLevel::Note => Diagnostic::<usize>::note(),
      _ => Diagnostic::error(),
    }
  } else {
    Diagnostic::note()
  };
  Ok(diagnostic)
}

fn main() -> Result<()> {
  let matches = App::new("sarif-fmt")
        .about("Pretty print SARIF results")
        .after_help(
            "The expected input is generated by running 'cargo clippy --message-format=json'.",
        )
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::new("input")
                .about("input file; reads from stdin if none is given")
                .takes_value(true),
        )
        .get_matches();

  let read = match matches.value_of_os("input").map(Path::new) {
    Some(path) => Box::new(File::open(path)?) as Box<dyn Read>,
    None => Box::new(std::io::stdin()) as Box<dyn Read>,
  };
  let reader = BufReader::new(read);
  let sarif = process(reader)?;
  to_writer_pretty(&sarif)
}
