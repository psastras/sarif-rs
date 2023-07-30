use crate::sarif::{self};
use anyhow::Result;
use derive_builder::Builder;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::convert::TryInto;
use std::io::{BufRead, Write};

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, Builder)]
#[builder(setter(into, strip_option))]
struct ClangTidyResult {
  pub file: Option<String>,
  pub line: Option<i64>,
  pub column: Option<i64>,
  pub level: String,
  pub message: String,
  pub rules: String,
}

impl TryFrom<&ClangTidyResult> for sarif::ArtifactLocation {
  type Error = sarif::ArtifactLocationBuilderError;

  fn try_from(result: &ClangTidyResult) -> Result<Self, Self::Error> {
    sarif::ArtifactLocationBuilder::default()
      .uri(result.file.as_ref().ok_or(
        sarif::ArtifactLocationBuilderError::UninitializedField("file"),
      )?)
      .build()
  }
}

impl TryFrom<&ClangTidyResult> for sarif::Region {
  type Error = sarif::RegionBuilderError;

  fn try_from(result: &ClangTidyResult) -> Result<Self, Self::Error> {
    sarif::RegionBuilder::default()
      .start_line(
        result
          .line
          .ok_or(sarif::RegionBuilderError::UninitializedField("line"))?,
      )
      .start_column(
        result
          .column
          .ok_or(sarif::RegionBuilderError::UninitializedField("column"))?,
      )
      .build()
  }
}

impl TryFrom<&ClangTidyResult> for sarif::Location {
  type Error = sarif::BuilderError;

  fn try_from(result: &ClangTidyResult) -> Result<Self, Self::Error> {
    let artifact_location: sarif::ArtifactLocation = result.try_into()?;
    let region: sarif::Region = result.try_into()?;
    let mut builder = sarif::LocationBuilder::default();
    builder.physical_location(
      sarif::PhysicalLocationBuilder::default()
        .artifact_location(artifact_location)
        .region(region)
        .build()?,
    );
    // Notes are converted to 'location' items with the message stored along with the location.
    // For other types of items (error, warning, info), the message will be stored inside the
    // 'result', so we skip it here.
    if result.level == "note" {
      builder.message::<sarif::Message>((&result.message).try_into()?);
    }
    Ok(builder.build()?)
  }
}

fn parse_clang_tidy_line(
  line: Result<String, std::io::Error>,
) -> Option<ClangTidyResult> {
  let re = Regex::new(
    r#"^(?P<file>([a-zA-Z]:|)[\w/\.\- \\]+):(?P<line>\d+):(?P<column>\d+):\s+(?P<level>error|warning|info|note):\s+(?P<message>.+)(?:\s+(?P<rules>\[[\w\-,\.]+\]))?$"#,
  ).unwrap();
  let line = line.unwrap();
  let caps = re.captures(&line);
  if let Some(caps) = caps {
    if let Some(message) = caps.name("message") {
      return Some(ClangTidyResult {
        file: caps.name("file").map(|f| f.as_str().into()),
        line: caps
          .name("line")
          .and_then(|f| f.as_str().parse::<i64>().ok()),
        column: caps
          .name("column")
          .and_then(|f| f.as_str().parse::<i64>().ok()),
        level: caps
          .name("level")
          .map_or_else(|| "info".into(), |f| f.as_str().into()),
        message: message.as_str().into(),
        rules: caps
          .name("rules")
          .map_or_else(|| "".into(), |f| f.as_str().into()),
      });
    }
  }
  None
}

fn process<R: BufRead>(reader: R) -> Result<sarif::Sarif> {
  let mut results = vec![];
  // Create an iterator over all the ClangTidyResult items
  let mut clang_tidy_result_iter =
    reader.lines().filter_map(parse_clang_tidy_line).peekable();

  while let Some(result) = clang_tidy_result_iter.next() {
    let location: sarif::Location = (&result).try_into()?;
    let mut related_locations = vec![];
    let message = format!("{} {}", result.message, result.rules);

    // Since clang-tidy emits "note" items which have to be folded into
    // the previous error/warning/info items, look ahead at the next items
    // and collect all the "notes".
    while let Some(next_result) = clang_tidy_result_iter.peek() {
      match next_result.level.as_str() {
        "note" => {
          let note_location: sarif::Location = (next_result).try_into()?;
          related_locations.push(note_location);
          // Since we got the next result via .peek(), advance the iterator
          clang_tidy_result_iter.next();
        }
        _ => {
          // Not a note, back to the outer loop
          break;
        }
      }
    }
    let mut builder = sarif::ResultBuilder::default();
    builder
      .message::<sarif::Message>((&message).try_into()?)
      .locations(vec![location])
      .level(result.level);
    if !related_locations.is_empty() {
      builder.related_locations(related_locations);
    }
    results.push(builder.build()?);
  }

  let tool_component: sarif::ToolComponent =
    sarif::ToolComponentBuilder::default()
      .name("clang-tidy")
      .build()?;
  let run = sarif::RunBuilder::default()
    .tool::<sarif::Tool>(tool_component.try_into()?)
    .results(results)
    .build()?;

  let sarif = sarif::SarifBuilder::default()
    .version(sarif::Version::V2_1_0.to_string())
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
/// * `reader` - A `BufRead` of clang-tidy output
pub fn parse_to_string<R: BufRead>(reader: R) -> Result<String> {
  let sarif = process(reader)?;
  let json = serde_json::to_string_pretty(&sarif)?;
  Ok(json)
}
