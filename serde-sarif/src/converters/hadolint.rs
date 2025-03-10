use std::{
  collections::HashMap,
  io::{BufRead, Write},
  str::FromStr,
};

use strum_macros::Display;
use strum_macros::EnumString;
use typed_builder::TypedBuilder;

use crate::sarif::{self, ResultLevel};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, TypedBuilder)]
#[builder(field_defaults(setter(into)))]
struct HadolintResult {
  file: String,
  line: i64,
  column: i64,
  level: String,
  code: String,
  message: String,
}

#[doc = "A value specifying the severity level of the result."]
#[derive(Display, Debug, Serialize, Deserialize, EnumString)]
#[serde(untagged)]
enum HadolintLevel {
  #[strum(serialize = "info")]
  Info,
  #[strum(serialize = "warning")]
  Warning,
  #[strum(serialize = "error")]
  Error,
  #[strum(serialize = "style")]
  Style,
}

impl From<HadolintLevel> for sarif::ResultLevel {
  fn from(level: HadolintLevel) -> Self {
    match level {
      HadolintLevel::Info => ResultLevel::Note,
      HadolintLevel::Warning => ResultLevel::Warning,
      HadolintLevel::Error => ResultLevel::Error,
      HadolintLevel::Style => ResultLevel::Note,
    }
  }
}

impl From<&HadolintResult> for sarif::ArtifactLocation {
  fn from(result: &HadolintResult) -> Self {
    sarif::ArtifactLocation::builder().uri(&result.file).build()
  }
}

impl From<&HadolintResult> for sarif::Location {
  fn from(result: &HadolintResult) -> Self {
    let artifact_location = sarif::ArtifactLocation::from(result);
    let region = sarif::Region::from(result);
    sarif::Location::builder()
      .physical_location(
        sarif::PhysicalLocation::builder()
          .artifact_location(artifact_location)
          .region(region)
          .build(),
      )
      .build()
  }
}

impl From<&HadolintResult> for sarif::Region {
  fn from(result: &HadolintResult) -> Self {
    sarif::Region::builder()
      .start_line(result.line)
      .start_column(result.column)
      .build()
  }
}

fn process<R: BufRead>(mut reader: R) -> Result<sarif::Sarif> {
  let mut data = String::new();
  reader.read_to_string(&mut data)?;
  let mut results = vec![];
  let mut map = HashMap::new();
  let mut rules = vec![];

  let hadolint_results: Vec<HadolintResult> = serde_json::from_str(&data)?;
  hadolint_results
    .iter()
    .try_for_each(|result| -> Result<()> {
      if !map.contains_key(&result.code) {
        map.insert(result.code.clone(), map.len() as i64);
        rules.push(
          sarif::ReportingDescriptor::builder()
            .id(result.code.clone())
            .name(result.code.clone())
            .short_description(&result.code)
            .full_description(&format!(
              "For more information: https://github.com/hadolint/hadolint/wiki/{}",
              result.code
            ))
            .build(),
        );
      }
      if let Some(value) = map.get(&result.code) {
        let level: sarif::ResultLevel =
          HadolintLevel::from_str(&result.level)?.into();
        results.push(
          sarif::Result::builder()
            .rule_id(result.code.clone())
            .rule_index(*value)
            .message(&result.message)
            .locations(vec![result.into()])
            .level(level)
            .build(),
        );
      }
      Ok(())
    })?;
  let tool_component = sarif::ToolComponent::builder()
    .name("hadolint")
    .rules(rules)
    .build();
  let run = sarif::Run::builder()
    .tool(tool_component)
    .results(results)
    .build();

  Ok(
    sarif::Sarif::builder()
      .version(sarif::Version::V2_1_0.to_string())
      .runs(vec![run])
      .build(),
  )
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
/// * `reader` - A `BufRead` of hadolint output
pub fn parse_to_string<R: BufRead>(reader: R) -> Result<String> {
  let sarif = process(reader)?;
  let json = serde_json::to_string_pretty(&sarif)?;
  Ok(json)
}
