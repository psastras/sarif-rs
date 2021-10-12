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
          .ok_or(sarif::RegionBuilderError::UninitializedField("line"))?
          as i64,
      )
      .start_column(
        result
          .column
          .ok_or(sarif::RegionBuilderError::UninitializedField("column"))?
          as i64,
      )
      .build()
  }
}

impl TryFrom<&ClangTidyResult> for sarif::Location {
  type Error = sarif::BuilderError;

  fn try_from(result: &ClangTidyResult) -> Result<Self, Self::Error> {
    let artifact_location: sarif::ArtifactLocation = result.try_into()?;
    let region: sarif::Region = result.try_into()?;
    Ok(
      sarif::LocationBuilder::default()
        .physical_location(
          sarif::PhysicalLocationBuilder::default()
            .artifact_location(artifact_location)
            .region(region)
            .build()?,
        )
        .build()?,
    )
  }
}

fn process<R: BufRead>(reader: R) -> Result<sarif::Sarif> {
  let mut results = vec![];
  let re = Regex::new(
    r#"^(?P<file>[\w/\.\- ]+):(?P<line>\d+):(?P<column>\d+):\s+(?P<level>error|warning|info):\s+(?P<message>.+)\s+(?P<rules>\[[\w\-,\.]+\])$"#,
  )?;
  reader
    .lines()
    .into_iter()
    .try_for_each(|line| -> Result<()> {
      let line = line.unwrap();
      let caps = re.captures(&line);
      if let Some(caps) = caps {
        if let Some(message) = caps.name("message") {
          let result = ClangTidyResult {
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
          };
          let location: sarif::Location = (&result).try_into()?;
          let message = format!("{} {}", result.message, result.rules);
          results.push(
            sarif::ResultBuilder::default()
              .message::<sarif::Message>((&message).try_into()?)
              .locations(vec![location])
              .level(result.level)
              .build()?,
          );
        }
      }
      Ok(())
    })?;

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
