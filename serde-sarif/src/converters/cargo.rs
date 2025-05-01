use std::{
  collections::HashMap,
  convert::From,
  io::{BufWriter, Write},
};

use crate::sarif::{self, Location};
use anyhow::Result;
use cargo_metadata::{
  self,
  diagnostic::{Diagnostic, DiagnosticLevel, DiagnosticSpan},
};

// TODO: refactor, add features, etc.

impl From<&Diagnostic> for sarif::Message {
  fn from(diagnostic: &Diagnostic) -> Self {
    sarif::Message::builder().text(&diagnostic.message).build()
  }
}

impl From<&DiagnosticSpan> for sarif::ArtifactLocation {
  fn from(span: &DiagnosticSpan) -> Self {
    sarif::ArtifactLocation::builder()
      .uri(&span.file_name)
      .build()
  }
}

impl From<&DiagnosticSpan> for sarif::Region {
  fn from(span: &DiagnosticSpan) -> Self {
    sarif::Region::builder()
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

impl From<&DiagnosticSpan> for sarif::Location {
  fn from(span: &DiagnosticSpan) -> Self {
    let artifact_location = sarif::ArtifactLocation::from(span);
    let region = sarif::Region::from(span);
    let location = sarif::Location::builder().physical_location(
      sarif::PhysicalLocation::builder()
        .artifact_location(artifact_location)
        .region(region)
        .build(),
    );

    if let Some(label) = span.label.as_ref() {
      location
        .message(sarif::Message::builder().text(label).build())
        .build()
    } else {
      location.build()
    }
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

/// Collects all the locations in the diagnostic's children spans
fn get_related_locations(
  diagnostic: &Diagnostic,
) -> Result<Vec<Location>, anyhow::Error> {
  let mut related_locations = vec![];
  for child in &diagnostic.children {
    let mut message = child.message.clone();
    for child_span in &child.spans {
      let mut child_loc: Location = child_span.into();
      if child_span.suggested_replacement.is_some() {
        let replacement = child_span.suggested_replacement.as_ref().unwrap();
        message.push_str(&format!(" \"{replacement}\""));
      }

      child_loc.message = Some(sarif::Message::from(&message));
      related_locations.push(child_loc);
    }
  }
  Ok(related_locations)
}

pub(crate) fn process<I: Iterator<Item = Diagnostic>>(
  mut diagnostic_iter: I,
  tool_name: &str,
  tool_info_uri: &str,
) -> Result<sarif::Sarif> {
  let mut results = vec![];
  let mut map = HashMap::new();
  let mut rules = vec![];

  let re =
    regex::Regex::new(r"^for further information visit (?P<url>\S+)").unwrap();

  diagnostic_iter.try_for_each(|diagnostic| -> Result<()> {
    diagnostic.spans.iter().try_for_each(|span| -> Result<()> {
      let diagnostic_code = match &diagnostic.code {
        Some(diagnostic_code) => diagnostic_code.code.clone(),
        _ => String::new(),
      };
      if !map.contains_key(&diagnostic_code) {
        let mut writer = BufWriter::new(Vec::new());
        build_global_message(&diagnostic, &mut writer)?;
        map.insert(diagnostic_code.clone(), map.len() as i64);

        let rule = sarif::ReportingDescriptor::builder()
          .id(&diagnostic_code)
          .full_description(&String::from_utf8(writer.into_inner()?)?);

        // help_uri is contained in a child diagnostic with a diagnostic level == help
        // search for the relevant child diagnostic, then extract the uri from the message
        let help_uri = diagnostic
          .children
          .iter()
          .find(|child| matches!(child.level, DiagnosticLevel::Help))
          .and_then(|help| {
            re.captures(&help.message)
              .and_then(|captures| captures.name("url"))
              .map(|re_match| re_match.as_str())
          });

        let rule = if let Some(help_uri) = help_uri {
          rule.help_uri(help_uri).build()
        } else {
          rule.build()
        };

        rules.push(rule);
      }

      if let Some(value) = map.get(&diagnostic_code) {
        let level: sarif::ResultLevel = (&diagnostic.level).into();
        results.push(
          sarif::Result::builder()
            .rule_id(diagnostic_code)
            .rule_index(*value)
            .message(&diagnostic)
            .locations(vec![span.into()])
            .level(level)
            .related_locations(get_related_locations(&diagnostic)?)
            .build(),
        );
      }
      Ok(())
    })?;

    Ok(())
  })?;

  let tool_component: sarif::ToolComponent = sarif::ToolComponent::builder()
    .name(tool_name)
    .information_uri(tool_info_uri)
    .rules(rules)
    .build();
  let run = sarif::Run::builder()
    .tool(tool_component)
    .results(results)
    .build();

  let sarif = sarif::Sarif::builder()
    .version(sarif::Version::V2_1_0.to_string())
    .schema(sarif::SCHEMA_URL)
    .runs(vec![run])
    .build();

  Ok(sarif)
}
