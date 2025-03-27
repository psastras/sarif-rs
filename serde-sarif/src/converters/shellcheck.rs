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
  #[builder(setter(strip_option), default)]
  fix: Option<ShellcheckFix>,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, TypedBuilder)]
#[builder(field_defaults(setter(into)))]
struct JSON1Format {
  #[builder(setter(transform = |i: impl IntoIterator<Item = impl Into<ShellcheckResult>>| i.into_iter().map(Into::into).collect()))]
  comments: Vec<ShellcheckResult>,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, TypedBuilder)]
#[builder(field_defaults(setter(into)))]
struct ShellcheckFix {
  #[builder(setter(transform = |i: impl IntoIterator<Item = impl Into<ShellcheckReplacement>>| i.into_iter().map(Into::into).collect()))]
  replacements: Vec<ShellcheckReplacement>,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, TypedBuilder)]
#[builder(field_defaults(setter(into)))]
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

impl From<&ShellcheckResult> for sarif::ArtifactLocation {
  fn from(result: &ShellcheckResult) -> Self {
    sarif::ArtifactLocation::builder().uri(&result.file).build()
  }
}

impl From<&ShellcheckResult> for sarif::Location {
  fn from(result: &ShellcheckResult) -> Self {
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

impl From<&ShellcheckReplacement> for sarif::Region {
  fn from(replacement: &ShellcheckReplacement) -> Self {
    sarif::Region::builder()
      .start_line(replacement.line)
      .start_column(replacement.column)
      .end_line(replacement.end_line)
      .end_column(replacement.end_column)
      .build()
  }
}

impl From<&ShellcheckResult> for sarif::Region {
  fn from(result: &ShellcheckResult) -> Self {
    sarif::Region::builder()
      .start_line(result.line)
      .start_column(result.column)
      .end_line(result.end_line)
      .end_column(result.end_column)
      .build()
  }
}

fn process<R: BufRead>(mut reader: R, format: String) -> Result<sarif::Sarif> {
  let mut data = String::new();
  reader.read_to_string(&mut data)?;
  let mut results = vec![];
  let mut map = HashMap::new();
  let mut rules = vec![];

  let shellcheck_results: Vec<ShellcheckResult> = if format != "json1" {
    serde_json::from_str(&data)?
  } else {
    let json1_format: JSON1Format = serde_json::from_str(&data)?;
    json1_format.comments
  };

  shellcheck_results
    .iter()
    .try_for_each(|result| -> Result<()> {
      #[allow(clippy::map_entry)]
      if !map.contains_key(&result.code.to_string()) {
        map.insert(result.code.to_string(), map.len() as i64);
        rules.push(
          sarif::ReportingDescriptor::builder()
            .id(result.code.to_string())
            .name(result.code.to_string())
            .short_description(&format!("SC{}", result.code))
            .help_uri(format!(
              "https://www.shellcheck.net/wiki/SC{}",
              result.code
            ))
            .full_description(&format!(
              "For more information: https://www.shellcheck.net/wiki/SC{}",
              result.code
            ))
            .build(),
        );
      }
      if let Some(value) = map.get(&result.code.to_string()) {
        let level: sarif::ResultLevel =
          ShellcheckLevel::from_str(&result.level)?.into();
        let fixes = if let Some(fix) = result.fix.as_ref() {
          fix
            .replacements
            .iter()
            .map(|fix| {
              sarif::Fix::builder().description(&fix.replacement).build()
            })
            // .filter_map(|v| v.ok())
            .collect()
        } else {
          vec![]
        };
        let related_locations = if let Some(fix) = result.fix.as_ref() {
          fix
            .replacements
            .iter()
            .map(|replacement| {
              sarif::Location::builder()
                .physical_location(
                  sarif::PhysicalLocation::builder()
                    .artifact_location(result)
                    .region(replacement)
                    .build(),
                )
                .build()
            })
            .collect()
        } else {
          vec![]
        };
        results.push(
          sarif::Result::builder()
            .rule_id(result.code.to_string())
            .rule_index(*value)
            .message(&result.message)
            .locations(vec![result.into()])
            .related_locations(related_locations)
            .fixes(fixes)
            .level(level)
            .build(),
        );
      }
      Ok(())
    })?;
  let tool_component: sarif::ToolComponent = sarif::ToolComponent::builder()
    .name("shellcheck")
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
/// * `format` - The format of the input
pub fn parse_to_writer<R: BufRead, W: Write>(
  reader: R,
  writer: W,
  format: String,
) -> Result<()> {
  let sarif = process(reader, format)?;
  serde_json::to_writer_pretty(writer, &sarif)?;
  Ok(())
}

/// Returns [sarif::Sarif] serialized into a JSON string
///
/// # Arguments
///
/// * `reader` - A `BufRead` of shellcheck output
/// * `format` - The format of the input
pub fn parse_to_string<R: BufRead>(
  reader: R,
  format: String,
) -> Result<String> {
  let sarif = process(reader, format)?;
  let json = serde_json::to_string_pretty(&sarif)?;
  Ok(json)
}
