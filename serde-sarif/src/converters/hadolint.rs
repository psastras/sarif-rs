use std::{
  collections::HashMap,
  convert::TryFrom,
  io::{BufRead, Write},
  str::FromStr,
};

use strum_macros::Display;
use strum_macros::EnumString;

use crate::sarif::{self, BuilderError, ResultLevel};
use anyhow::Result;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use std::convert::TryInto;

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, Builder)]
#[builder(setter(into, strip_option))]
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

impl TryFrom<&HadolintResult> for sarif::ArtifactLocation {
  type Error = sarif::ArtifactLocationBuilderError;

  fn try_from(result: &HadolintResult) -> Result<Self, Self::Error> {
    sarif::ArtifactLocationBuilder::default()
      .uri(&result.file)
      .build()
  }
}

impl TryFrom<&HadolintResult> for sarif::Location {
  type Error = BuilderError;

  fn try_from(result: &HadolintResult) -> Result<Self, Self::Error> {
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

impl TryFrom<&HadolintResult> for sarif::Region {
  type Error = sarif::RegionBuilderError;

  fn try_from(result: &HadolintResult) -> Result<Self, Self::Error> {
    sarif::RegionBuilder::default()
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
          sarif::ReportingDescriptorBuilder::default()
            .id(result.code.clone())
            .name(result.code.clone())
            .short_description::<sarif::MultiformatMessageString>(
              (&result.code as &String).try_into()?,
            )
            .full_description::<sarif::MultiformatMessageString>(
              (&format!(
                "For more information: https://github.com/hadolint/hadolint/wiki/{}",
                result.code
              ))
                .try_into()?,
            )
            .build()?,
        );
      }
      if let Some(value) = map.get(&result.code) {
        let level: sarif::ResultLevel =
          HadolintLevel::from_str(&result.level)?.into();
        results.push(
          sarif::ResultBuilder::default()
            .rule_id(result.code.clone())
            .rule_index(*value)
            .message::<sarif::Message>((&result.message).try_into()?)
            .locations(vec![result.try_into()?])
            .level(level.to_string())
            .build()?,
        );
      }
      Ok(())
    })?;
  let tool_component: sarif::ToolComponent =
    sarif::ToolComponentBuilder::default()
      .name("hadolint")
      .rules(rules)
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
/// * `reader` - A `BufRead` of hadolint output
pub fn parse_to_string<R: BufRead>(reader: R) -> Result<String> {
  let sarif = process(reader)?;
  let json = serde_json::to_string_pretty(&sarif)?;
  Ok(json)
}
