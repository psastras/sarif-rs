use std::io::{BufRead, Write};

use crate::sarif;
use anyhow::Result;
use cargo_metadata::{self, diagnostic::Diagnostic};
use serde::Deserialize;

struct DiagnosticIter<R> {
  input: R,
}

impl<R: BufRead> Iterator for DiagnosticIter<R> {
  type Item = std::io::Result<Option<Diagnostic>>;

  fn next(&mut self) -> Option<Self::Item> {
    let mut line = String::new();
    self
      .input
      .read_line(&mut line)
      .map(|n| {
        if n == 0 {
          None
        } else {
          if line.ends_with('\n') {
            line.truncate(line.len() - 1);
          }
          let mut deserializer = serde_json::Deserializer::from_str(&line);
          deserializer.disable_recursion_limit();
          Some(Diagnostic::deserialize(&mut deserializer).ok())
        }
      })
      .transpose()
  }
}

fn process<R: BufRead>(reader: R) -> Result<sarif::Sarif> {
  let stream = DiagnosticIter { input: reader };
  let iter = stream.filter_map(|r| r.ok()).flatten();

  super::cargo::process(iter, "miri", "https://github.com/rust-lang/miri")
}

/// Returns [sarif::Sarif] serialized into a JSON stream
///
/// # Arguments
///
/// * `reader` - A `BufRead` of cargo miri output
/// * `writer` - A `Writer` to write the results to
pub fn parse_to_writer<R: BufRead, W: Write>(
  reader: R,
  writer: W,
) -> Result<()> {
  let sarif = process(reader)?;
  serde_json::to_writer_pretty(writer, &sarif)?;
  Ok(())
}

/// Returns [sarif::Sarif] serialized into a JSON string
///
/// # Arguments
///
/// * `reader` - A `BufRead` of cargo miri output
pub fn parse_to_string<R: BufRead>(reader: R) -> Result<String> {
  let sarif = process(reader)?;
  let json = serde_json::to_string_pretty(&sarif)?;
  Ok(json)
}
