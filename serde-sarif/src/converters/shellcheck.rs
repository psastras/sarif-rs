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
struct ShellcheckResult {
  file: String,
  line: i64,
  #[serde(rename = "endLine")]
  end_line: i64,
  column: i64,
  #[serde(rename = "endColumn")]
  end_column: i64,
  level: String,
  code: i64,
  message: String,
  fix: Option<ShellcheckFix>,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, Builder)]
#[builder(setter(into, strip_option))]
struct ShellcheckFix {
  replacements: Vec<ShellcheckReplacement>,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, Builder)]
#[builder(setter(into, strip_option))]
struct ShellcheckReplacement {
  line: i64,
  #[serde(rename = "endLine")]
  end_line: i64,
  precedence: i64,
  #[serde(rename = "insertionPoint")]
  insertion_point: String,
  column: i64,
  replacement: String,
  #[serde(rename = "endColumn")]
  end_column: i64,
}

#[doc = "A value specifying the severity level of the result."]
#[derive(
  Display, Debug, Serialize, Deserialize, EnumString, Copy, Clone, PartialEq,
)]
#[serde(untagged)]
enum ShellcheckLevel {
  #[strum(serialize = "info")]
  Info,
  #[strum(serialize = "warning")]
  Warning,
  #[strum(serialize = "error")]
  Error,
  #[strum(serialize = "style")]
  Style,
}

impl From<ShellcheckLevel> for sarif::ResultLevel {
  fn from(level: ShellcheckLevel) -> Self {
    match level {
      ShellcheckLevel::Info => ResultLevel::Note,
      ShellcheckLevel::Warning => ResultLevel::Warning,
      ShellcheckLevel::Error => ResultLevel::Error,
      ShellcheckLevel::Style => ResultLevel::Note,
    }
  }
}

impl TryFrom<&ShellcheckResult> for sarif::ArtifactLocation {
  type Error = sarif::ArtifactLocationBuilderError;

  fn try_from(result: &ShellcheckResult) -> Result<Self, Self::Error> {
    sarif::ArtifactLocationBuilder::default()
      .uri(&result.file)
      .build()
  }
}

impl TryFrom<&ShellcheckResult> for sarif::Location {
  type Error = BuilderError;

  fn try_from(result: &ShellcheckResult) -> Result<Self, Self::Error> {
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

impl TryFrom<&ShellcheckReplacement> for sarif::Region {
  type Error = sarif::RegionBuilderError;

  fn try_from(
    replacement: &ShellcheckReplacement,
  ) -> Result<Self, Self::Error> {
    sarif::RegionBuilder::default()
      .start_line(replacement.line as i64)
      .start_column(replacement.column as i64)
      .end_line(replacement.end_line as i64)
      .end_column(replacement.end_column as i64)
      .build()
  }
}

impl TryFrom<&ShellcheckResult> for sarif::Region {
  type Error = sarif::RegionBuilderError;

  fn try_from(result: &ShellcheckResult) -> Result<Self, Self::Error> {
    sarif::RegionBuilder::default()
      .start_line(result.line as i64)
      .start_column(result.column as i64)
      .end_line(result.end_line as i64)
      .end_column(result.end_column as i64)
      .build()
  }
}

fn process<R: BufRead>(mut reader: R) -> Result<sarif::Sarif> {
  let mut data = String::new();
  reader.read_to_string(&mut data)?;
  let mut results = vec![];
  let mut map = HashMap::new();
  let mut rules = vec![];

  let shellcheck_results: Vec<ShellcheckResult> = serde_json::from_str(&data)?;
  shellcheck_results
    .iter()
    .try_for_each(|result| -> Result<()> {
      #[allow(clippy::map_entry)]
      if !map.contains_key(&result.code.to_string()) {
        map.insert(result.code.to_string(), map.len() as i64);
        rules.push(
          sarif::ReportingDescriptorBuilder::default()
            .id(result.code.to_string())
            .name(result.code.to_string())
            .short_description::<sarif::MultiformatMessageString>(
              (&format!("SC{}", result.code)).try_into()?,
            )
            .full_description::<sarif::MultiformatMessageString>(
              (&format!(
                "For more information: https://www.shellcheck.net/wiki/SC{}",
                result.code
              ))
                .try_into()?,
            )
            .build()?,
        );
      }
      if let Some(value) = map.get(&result.code.to_string()) {
        let level: sarif::ResultLevel =
          ShellcheckLevel::from_str(&result.level)?.into();
        let fixes = if let Some(fix) = result.fix.as_ref() {
          fix
            .replacements
            .iter()
            .map(|fix| -> Result<sarif::Fix> {
              Ok(
                sarif::FixBuilder::default()
                  .description::<sarif::Message>((&fix.replacement).try_into()?)
                  .build()?,
              )
            })
            .filter_map(|v| v.ok())
            .collect()
        } else {
          vec![]
        };
        let related_locations = if let Some(fix) = result.fix.as_ref() {
          fix
            .replacements
            .iter()
            .map(|replacement| -> Result<sarif::Location> {
              let region: sarif::Region = replacement.try_into()?;
              let artifact_location: sarif::ArtifactLocation =
                result.try_into()?;
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
            })
            .filter_map(|v| v.ok())
            .collect()
        } else {
          vec![]
        };
        results.push(
          sarif::ResultBuilder::default()
            .rule_id(result.code.to_string())
            .rule_index(*value)
            .message::<sarif::Message>((&result.message).try_into()?)
            .locations(vec![result.try_into()?])
            .related_locations(related_locations)
            .fixes(fixes)
            .level(level.to_string())
            .build()?,
        );
      }
      Ok(())
    })?;
  let tool_component: sarif::ToolComponent =
    sarif::ToolComponentBuilder::default()
      .name("shellcheck")
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
/// * `reader` - A `BufRead` of shellcheck output
pub fn parse_to_string<R: BufRead>(reader: R) -> Result<String> {
  let sarif = process(reader)?;
  let json = serde_json::to_string_pretty(&sarif)?;
  Ok(json)
}
