use std::{
  collections::HashMap,
  convert::From,
  convert::TryFrom,
  io::{BufRead, BufWriter, Write},
};

use crate::sarif::{self, BuilderError};
use anyhow::Result;
use cargo_metadata::{
  self,
  diagnostic::{Diagnostic, DiagnosticLevel, DiagnosticSpan},
  Message,
};
use std::convert::TryInto;

// TODO: refactor, add features, etc.

impl TryFrom<&Diagnostic> for sarif::Message {
  type Error = sarif::MessageBuilderError;

  fn try_from(diagnostic: &Diagnostic) -> Result<Self, Self::Error> {
    sarif::MessageBuilder::default()
      .text(diagnostic.message.clone())
      .build()
  }
}

impl TryFrom<&DiagnosticSpan> for sarif::ArtifactLocation {
  type Error = sarif::ArtifactLocationBuilderError;

  fn try_from(span: &DiagnosticSpan) -> Result<Self, Self::Error> {
    sarif::ArtifactLocationBuilder::default()
      .uri(&span.file_name)
      .build()
  }
}

impl TryFrom<&DiagnosticSpan> for sarif::Region {
  type Error = sarif::RegionBuilderError;

  fn try_from(span: &DiagnosticSpan) -> Result<Self, Self::Error> {
    sarif::RegionBuilder::default()
      .byte_offset(span.byte_start)
      .byte_length(span.byte_end - span.byte_start)
      .start_line(span.line_start as i64)
      .start_column(span.column_start as i64)
      .end_line(span.line_end as i64)
      .end_column(span.column_end as i64)
      .build()
  }
}

impl From<&DiagnosticLevel> for sarif::ResultLevel {
  fn from(level: &DiagnosticLevel) -> Self {
    match level {
      DiagnosticLevel::Help | DiagnosticLevel::Note => sarif::ResultLevel::Note,
      DiagnosticLevel::Warning => sarif::ResultLevel::Warning,
      DiagnosticLevel::Error => sarif::ResultLevel::Error,
      _ => sarif::ResultLevel::None,
    }
  }
}

impl TryFrom<&DiagnosticSpan> for sarif::Location {
  type Error = BuilderError;

  fn try_from(span: &DiagnosticSpan) -> Result<Self, Self::Error> {
    let artifact_location: sarif::ArtifactLocation = span.try_into()?;
    let region: sarif::Region = span.try_into()?;
    let mut location_builder = sarif::LocationBuilder::default();
    location_builder.physical_location(
      sarif::PhysicalLocationBuilder::default()
        .artifact_location(artifact_location)
        .region(region)
        .build()?,
    );
    if let Some(label) = span.label.as_ref() {
      location_builder
        .message(sarif::MessageBuilder::default().text(label).build()?);
    }
    Ok(location_builder.build()?)
  }
}

// recursively visits all diagnostic children which are non-local (have no span)
//  to build up diagnostic text.
fn build_global_message<W: Write>(
  diagnostic: &Diagnostic,
  writer: &mut BufWriter<W>,
) -> Result<()> {
  // if span exists, this message is local to a span, so skip it
  if diagnostic.spans.is_empty() {
    writeln!(writer, "{}", diagnostic.message)?;
  }

  diagnostic
    .children
    .iter()
    .try_for_each(|diagnostic| build_global_message(diagnostic, writer))
}

fn process<R: BufRead>(reader: R) -> Result<sarif::Sarif> {
  let mut results = vec![];
  let mut map = HashMap::new();
  let mut rules = vec![];
  Message::parse_stream(reader)
    .filter_map(|r| r.ok())
    .filter_map(|m| match m {
      Message::CompilerMessage(msg) => Some(msg.message),
      _ => None,
    })
    .try_for_each(|diagnostic| -> Result<()> {
      if diagnostic.spans.is_empty() {
        return Ok(());
      }

      let diagnostic_code = match &diagnostic.code {
        Some(diagnostic_code) => diagnostic_code.code.clone(),
        _ => "".into(),
      };
      if !map.contains_key(&diagnostic_code) {
        let mut writer = BufWriter::new(Vec::new());
        build_global_message(&diagnostic, &mut writer)?;
        map.insert(diagnostic_code.clone(), map.len() as i64);
        let mut rule = sarif::ReportingDescriptorBuilder::default();
        rule
          .id(&diagnostic_code)
          .full_description::<sarif::MultiformatMessageString>(
            (&String::from_utf8(writer.into_inner()?)?).try_into()?,
          );

        // help_uri is contained in a child diagnostic with a diagnostic level == help
        // search for the relevant child diagnostic, then extract the uri from the message
        if let Some(help_uri) = diagnostic
          .children
          .iter()
          .find(|child| matches!(child.level, DiagnosticLevel::Help))
          .and_then(|help| {
            let re =
              regex::Regex::new(r"^for further information visit (?P<url>\S+)")
                .unwrap();
            re.captures(&help.message)
              .and_then(|captures| captures.name("url"))
              .map(|re_match| re_match.as_str())
          })
        {
          rule.help_uri(help_uri);
        }
        rules.push(rule.build()?);
      }

      if let Some(value) = map.get(&diagnostic_code) {
        let level = sarif::ResultLevel::from(&diagnostic.level);
        results.push(
          sarif::ResultBuilder::default()
            .rule_id(diagnostic_code)
            .rule_index(*value)
            .message::<sarif::Message>((&diagnostic).try_into()?)
            .level(level.to_string())
            .build()?,
        );

        let result = results.last_mut().unwrap();
        diagnostic.spans.iter().try_for_each(|span| -> Result<()> {
          let location = sarif::Location::try_from(span)?;
          if span.is_primary {
            match result.locations.as_mut() {
              Some(locations) => locations.push(location),
              None => result.locations = Some(vec![location]),
            }
          } else {
            match result.related_locations.as_mut() {
              Some(related_locations) => related_locations.push(location),
              None => result.related_locations = Some(vec![location]),
            }
          }
          Ok(())
        })?;
      }

      Ok(())
    })?;

  let tool_component: sarif::ToolComponent =
    sarif::ToolComponentBuilder::default()
      .name("clippy")
      .information_uri("https://rust-lang.github.io/rust-clippy/")
      .rules(rules)
      .build()?;
  let run = sarif::RunBuilder::default()
    .tool::<sarif::Tool>(tool_component.try_into()?)
    .results(results)
    .build()?;

  let sarif = sarif::SarifBuilder::default()
    .version(sarif::Version::V2_1_0.to_string())
    .schema(sarif::SCHEMA_URL)
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
