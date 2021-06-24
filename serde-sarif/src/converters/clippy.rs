use std::io::{BufRead, Write};

use crate::sarif;
use anyhow::Result;
use cargo_metadata::{self, diagnostic::DiagnosticLevel, Message};

// TODO: refactor, add features, etc.
fn process<R: BufRead>(reader: R) -> Result<sarif::Sarif> {
  let mut results = vec![];
  for result in Message::parse_stream(reader) {
    let message = result?;
    if let Message::CompilerMessage(msg) = message {
      let diagnostic = &msg.message;
      for span in &diagnostic.spans {
        let message = sarif::MessageBuilder::default()
          .id(match &diagnostic.code {
            Some(diagnostic_code) => diagnostic_code.code.clone(),
            _ => "".into(),
          })
          .text(&diagnostic.message)
          .build()?;
        let artifact_location = sarif::ArtifactLocationBuilder::default()
          .uri(&span.file_name)
          .build()?;
        let region = sarif::RegionBuilder::default()
          // todo: why isn't this correct?
          // .byte_offset(span.byte_start)
          // .byte_length(span.byte_end - span.byte_start)
          .start_line(span.line_start as i64)
          .start_column(span.column_start as i64)
          .end_line(span.line_end as i64)
          .end_column(span.column_end as i64)
          .build()?;
        let physical_location = sarif::PhysicalLocationBuilder::default()
          .artifact_location(artifact_location)
          .region(region)
          .build()?;
        let location = sarif::LocationBuilder::default()
          .physical_location(physical_location)
          .build()?;
        results.push(
          sarif::ResultBuilder::default()
            .message(message)
            .locations(vec![location])
            .level(match diagnostic.level {
              DiagnosticLevel::Help | DiagnosticLevel::Note => "note",
              DiagnosticLevel::Warning => "warning",
              DiagnosticLevel::Error => "error",
              _ => "none",
            })
            .build()?,
        )
      }
    }
  }

  let tool_component = sarif::ToolComponentBuilder::default()
    .name("clippy")
    .build()?;

  let tool = sarif::ToolBuilder::default()
    .driver(tool_component)
    .build()?;

  let run = sarif::RunBuilder::default()
    .tool(tool)
    .results(results)
    .build()?;

  let sarif = sarif::SarifBuilder::default()
    .version("2.1.0")
    .runs(vec![run])
    .build()?;

  Ok(sarif)
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
  serde_json::to_writer_pretty(writer, &sarif)?;
  Ok(())
}

/// Returns [sarif::Sarif] serialized into a JSON string
///
/// # Arguments
///
/// * `reader` - A `BufRead` of cargo clippy output
pub fn parse_to_string<R: BufRead>(reader: R) -> Result<String> {
  let sarif = process(reader)?;
  let json = serde_json::to_string_pretty(&sarif)?;
  Ok(json)
}

#[cfg(test)]
mod tests {
  use super::*;

  fn test() {}
}
