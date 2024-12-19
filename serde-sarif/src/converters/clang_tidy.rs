use crate::sarif::{self};
use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::convert::TryInto;
use std::io::{BufRead, Write};
use std::str::FromStr;
use typed_builder::TypedBuilder;

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, TypedBuilder)]
#[builder(field_defaults(setter(into)))]
struct ClangTidyResult {
  #[builder(setter(strip_option), default)]
  pub file: Option<String>,
  #[builder(setter(strip_option), default)]
  pub line: Option<i64>,
  #[builder(setter(strip_option), default)]
  pub column: Option<i64>,
  pub level: String,
  pub message: String,
  pub rules: String,
}

impl TryFrom<&ClangTidyResult> for sarif::ArtifactLocation {
  type Error = sarif::BuilderError;

  fn try_from(result: &ClangTidyResult) -> Result<Self, Self::Error> {
    result
      .file
      .as_ref()
      .ok_or(sarif::BuilderError::UninitializedField("file"))
      .map(|uri| sarif::ArtifactLocation::builder().uri(uri).build())
  }
}

impl TryFrom<&ClangTidyResult> for sarif::Region {
  type Error = sarif::BuilderError;

  fn try_from(result: &ClangTidyResult) -> Result<Self, Self::Error> {
    let start_line = result
      .line
      .ok_or(sarif::BuilderError::UninitializedField("line"))?;
    let start_column = result
      .column
      .ok_or(sarif::BuilderError::UninitializedField("column"))?;
    Ok(
      sarif::Region::builder()
        .start_line(start_line)
        .start_column(start_column)
        .build(),
    )
  }
}

impl TryFrom<&ClangTidyResult> for sarif::Location {
  type Error = sarif::BuilderError;

  fn try_from(result: &ClangTidyResult) -> Result<Self, Self::Error> {
    let artifact_location = sarif::ArtifactLocation::try_from(result)?;
    let region = sarif::Region::try_from(result)?;
    let location = sarif::Location::builder().physical_location(
      sarif::PhysicalLocation::builder()
        .artifact_location(artifact_location)
        .region(region)
        .build(),
    );

    // Notes are converted to 'location' items with the message stored along with the location.
    // For other types of items (error, warning, info), the message will be stored inside the
    // 'result', so we skip it here.
    Ok(if result.level == "note" {
      location.message(&result.message).build()
    } else {
      location.build()
    })
  }
}

fn parse_clang_tidy_line(
  line: Result<String, std::io::Error>,
) -> Option<ClangTidyResult> {
  let re = Regex::new(
    r"^(?P<file>([a-zA-Z]:|)[\w/\.\- \\]+):(?P<line>\d+):(?P<column>\d+):\s+(?P<level>error|warning|info|note):\s+(?P<message>.+)(?:\s+(?P<rules>\[[\w\-,\.]+\]))?$"
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

    let builder = sarif::Result::builder()
      .message(&message)
      .locations(vec![location])
      .level(sarif::ResultLevel::from_str(&result.level)?);
    let result = if !related_locations.is_empty() {
      builder.related_locations(related_locations).build()
    } else {
      builder.build()
    };

    results.push(result);
  }

  let tool_component: sarif::ToolComponent =
    sarif::ToolComponent::builder().name("clang-tidy").build();
  let run = sarif::Run::builder()
    .tool(tool_component)
    .results(results)
    .build();

  let sarif = sarif::Sarif::builder()
    .version(sarif::Version::V2_1_0.to_string())
    .runs(vec![run])
    .build();

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
