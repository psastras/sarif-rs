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
struct DenyDiagnostic {
  #[serde(rename = "type")]
  diagnostic_type: String,
  message: String,
  #[builder(setter(strip_option), default)]
  severity: Option<String>,
  #[builder(setter(strip_option), default)]
  package: Option<DenyPackage>,
  #[builder(setter(strip_option), default)]
  advisory: Option<DenyAdvisory>,
  #[builder(setter(strip_option), default)]
  license: Option<DenyLicense>,
  #[builder(setter(strip_option), default)]
  code: Option<String>,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, TypedBuilder)]
#[builder(field_defaults(setter(into)))]
struct DenyPackage {
  name: String,
  version: String,
  #[builder(setter(strip_option), default)]
  source: Option<String>,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, TypedBuilder)]
#[builder(field_defaults(setter(into)))]
struct DenyAdvisory {
  id: String,
  title: String,
  #[builder(setter(strip_option), default)]
  description: Option<String>,
  #[builder(setter(strip_option), default)]
  url: Option<String>,
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, TypedBuilder)]
#[builder(field_defaults(setter(into)))]
struct DenyLicense {
  #[builder(setter(strip_option), default)]
  expression: Option<String>,
  #[builder(setter(strip_option), default)]
  files: Option<Vec<String>>,
}

#[doc = "A value specifying the severity level of the result."]
#[derive(Display, Debug, Serialize, Deserialize, EnumString)]
#[serde(untagged)]
enum DenySeverity {
  #[strum(serialize = "error")]
  Error,
  #[strum(serialize = "warning")]
  Warning,
  #[strum(serialize = "note")]
  Note,
}

impl From<DenySeverity> for sarif::ResultLevel {
  fn from(level: DenySeverity) -> Self {
    match level {
      DenySeverity::Error => ResultLevel::Error,
      DenySeverity::Warning => ResultLevel::Warning,
      DenySeverity::Note => ResultLevel::Note,
    }
  }
}

impl From<&DenyDiagnostic> for sarif::ArtifactLocation {
  fn from(_diagnostic: &DenyDiagnostic) -> Self {
    // cargo-deny typically applies to Cargo.toml
    sarif::ArtifactLocation::builder().uri("Cargo.toml").build()
  }
}

impl From<&DenyDiagnostic> for sarif::Location {
  fn from(diagnostic: &DenyDiagnostic) -> Self {
    let artifact_location = sarif::ArtifactLocation::from(diagnostic);
    let region = sarif::Region::builder()
      .start_line(1)
      .start_column(1)
      .build();
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

fn process<R: BufRead>(mut reader: R) -> Result<sarif::Sarif> {
  let mut data = String::new();
  reader.read_to_string(&mut data)?;
  let mut results = vec![];
  let mut map = HashMap::new();
  let mut rules = vec![];

  // Parse each line as a separate JSON object (JSONL format typical for cargo-deny)
  for line in data.lines() {
    if line.trim().is_empty() {
      continue;
    }
    
    let diagnostic: DenyDiagnostic = match serde_json::from_str(line) {
      Ok(d) => d,
      Err(_) => continue, // Skip lines that aren't valid JSON
    };
    
    let rule_id = diagnostic.code.clone()
      .or_else(|| diagnostic.advisory.as_ref().map(|a| a.id.clone()))
      .unwrap_or_else(|| diagnostic.diagnostic_type.clone());

    if !map.contains_key(&rule_id) {
      map.insert(rule_id.clone(), map.len() as i64);
      
      let rule_description = if let Some(ref advisory) = diagnostic.advisory {
        advisory.title.to_string()
      } else {
        diagnostic.message.clone()
      };
      
      let help_uri = diagnostic.advisory.as_ref()
        .and_then(|a| a.url.clone())
        .or_else(|| {
          if diagnostic.diagnostic_type == "advisory" {
            Some("https://rustsec.org/".to_string())
          } else {
            Some("https://embarkstudios.github.io/cargo-deny/".to_string())
          }
        });

      let rule = sarif::ReportingDescriptor::builder()
        .id(rule_id.clone())
        .name(rule_id.clone())
        .short_description(&rule_description);

      let rule = if let Some(uri) = help_uri {
        rule.help_uri(uri).build()
      } else {
        rule.build()
      };

      rules.push(rule);
    }

    if let Some(value) = map.get(&rule_id) {
      let level: sarif::ResultLevel = diagnostic.severity
        .as_ref()
        .and_then(|s| DenySeverity::from_str(s).ok())
        .map(|s| s.into())
        .unwrap_or(ResultLevel::Warning);

      let mut message = diagnostic.message.clone();
      
      // Enhance message with package info if available
      if let Some(ref package) = diagnostic.package {
        message = format!("{} (package: {} {})", message, package.name, package.version);
      }

      results.push(
        sarif::Result::builder()
          .rule_id(rule_id)
          .rule_index(*value)
          .message(&message)
          .locations(vec![(&diagnostic).into()])
          .level(level)
          .build(),
      );
    }
  }

  let tool_component = sarif::ToolComponent::builder()
    .name("cargo-deny")
    .information_uri("https://embarkstudios.github.io/cargo-deny/")
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
/// * `reader` - A `BufRead` of cargo-deny output
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
/// * `reader` - A `BufRead` of cargo-deny output
pub fn parse_to_string<R: BufRead>(reader: R) -> Result<String> {
  let sarif = process(reader)?;
  let json = serde_json::to_string_pretty(&sarif)?;
  Ok(json)
}