#![doc(html_root_url = "https://docs.rs/sarif-fmt/0.2.8")]
#![recursion_limit = "256"]
//! # DO NOT USE (EARLY IMPLEMENTATION)
//!
//! This crate provides a command line tool to format SARIF files to pretty
//! printed text.
//!
//! The latest [documentation can be found here](https://psastras.github.io/sarif-rs/sarif_fmt/index.html).
//!
//! SARIF or the Static Analysis Results Interchange Format is an industry
//! standard format for the output of static analysis tools. More information
//! can be found on the official website: [https://sarifweb.azurewebsites.net/](https://sarifweb.azurewebsites.net/).
//!
//! ## Installation
//!
//! `sarif-fmt` may be insalled via `cargo`
//!
//! ```shell
//! cargo install sarif-fmt
//! ```
//!
//! ## Usage
//!
//! For most cases, simply pipe a SARIF file into `sarif-fmt`
//!
//! ## Example
//!
//!```shell
//! cargo clippy --message-format=json | clippy-sarif | sarif-fmt
//! ```
//!

use anyhow::Result;
use clap::{App, Arg};
use codespan_reporting::diagnostic::Diagnostic;
use codespan_reporting::diagnostic::Label;
use codespan_reporting::files::Files;
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::Color;
use codespan_reporting::term::termcolor::ColorChoice;
use codespan_reporting::term::termcolor::ColorSpec;
use codespan_reporting::term::termcolor::StandardStream;
use codespan_reporting::term::termcolor::WriteColor;
use if_chain::if_chain;
use serde_sarif::sarif;
use serde_sarif::sarif::ResultLevel;
use std::fs::File;
use std::io::Write;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
use std::usize;

fn process<R: BufRead>(reader: R) -> Result<sarif::Sarif> {
  let s: sarif::Sarif = serde_json::from_reader(reader)?;
  Ok(s)
}

fn try_find_file(
  physical_location: &sarif::PhysicalLocation,
  run: &sarif::Run,
) -> Result<PathBuf> {
  let artifact_location = physical_location
    .artifact_location
    .as_ref()
    .map_or_else(|| Err(anyhow::anyhow!("No artifact location.")), Ok)?;
  let uri = artifact_location
    .uri
    .as_ref()
    .map_or_else(|| Err(anyhow::anyhow!("No uri.")), Ok)?;
  let path = Path::new(uri);

  // if the path is absolute and exists, return it, else
  if path.is_absolute() {
    if path.exists() {
      return Ok(path.to_path_buf());
    }
    return Err(anyhow::anyhow!("Path does not exist"));
  }

  // if it's a relative path, it's more complicated and dictated by the SARIF spec

  // 1. check if uri base id exists -- it SHOULD exist, but may not
  if let Some(uri_base_id) = artifact_location.uri_base_id.as_ref() {
    // check if this is defined in originalUriBaseIds
    if let Some(original_uri_base_ids) = run.original_uri_base_ids.as_ref() {
      let mut path = PathBuf::new();
      if let Some(uri_base) = original_uri_base_ids.get(uri_base_id) {
        // todo: this doesn't handle recursive uri_base_id...
        if let Some(uri) = uri_base.uri.as_ref() {
          path.push(uri);
        }
      }
      path.push(uri);
      if path.exists() {
        return Ok(path);
      }
    }

    // just check if the relative path exists by chance
    if path.exists() {
      return Ok(path.to_path_buf());
    }
  }
  if path.exists() {
    Ok(path.to_path_buf())
  } else {
    Err(anyhow::anyhow!("Path not found: {:#?}", path))
  }
}

fn get_physical_location_contents(
  physical_location: &sarif::PhysicalLocation,
  run: &sarif::Run,
) -> Result<String> {
  let path = try_find_file(physical_location, run)?;
  let mut file = File::open(path)?;
  let mut contents = String::new();
  file.read_to_string(&mut contents)?;
  Ok(contents)
}

fn try_get_byte_offset(
  file_id: usize,
  files: &SimpleFiles<&String, String>,
  row: i64,
  column: i64,
) -> Result<usize> {
  files
    .line_range(file_id, row as usize - 1)?
    .find(|byte| {
      if let Ok(location) = files.location(file_id, *byte) {
        location.column_number == column as usize
      } else {
        false
      }
    })
    .ok_or_else(|| anyhow::anyhow!("Byte offset not found"))
}

fn get_byte_range(
  file_id: usize,
  files: &SimpleFiles<&String, String>,
  region: &sarif::Region,
) -> (Option<usize>, Option<usize>) {
  // todo: support character regions
  let byte_offset = if let Some(byte_offset) = region.byte_offset {
    Some(byte_offset as usize)
  } else if let (Some(start_line), Some(start_column)) =
    (region.start_line, region.start_column)
  {
    if let Ok(byte_offset) =
      try_get_byte_offset(file_id, files, start_line, start_column)
    {
      Some(byte_offset)
    } else {
      None
    }
  } else {
    None
  };

  let byte_end = if let Some(byte_offset) = byte_offset {
    if let Some(byte_length) = region.byte_length {
      Some(byte_offset + byte_length as usize)
    } else if let (Some(end_line), Some(end_column)) = (
      region.end_line.map_or_else(|| region.start_line, Some), // if no end_line, default to start_line
      region.end_column.map_or_else(|| region.start_column, Some), // if no end_column, default to start_column
    ) {
      if let Ok(byte_offset) =
        try_get_byte_offset(file_id, files, end_line, end_column)
      {
        Some(byte_offset)
      } else {
        Some(byte_offset)
      }
    } else {
      Some(byte_offset)
    }
  } else {
    None
  };

  (byte_offset, byte_end)
}

// If kind (§3.27.9) has any value other than "fail", then if level is absent, it SHALL default to "none", and if it is present, it SHALL have the value "none".
// If kind has the value "fail" and level is absent, then level SHALL be determined by the following procedure:
// IF rule (§3.27.7) is present THEN
//     LET theDescriptor be the reportingDescriptor object (§3.49) that it specifies.
//     # Is there a configuration override for the level property?
//     IF result.provenance.invocationIndex (§3.27.29, §3.48.6) is >= 0 THEN
//         LET theInvocation be the invocation object (§3.20) that it specifies.
//         IF theInvocation.ruleConfigurationOverrides (§3.20.5) is present
//               AND it contains a configurationOverride object (§3.51) whose
//               descriptor property (§3.51.2) specifies theDescriptor THEN
//             LET theOverride be that configurationOverride object.
//             IF theOverride.configuration.level (§3.51.3, §3.50.3) is present THEN
//               Set level to theConfiguration.level.
//     ELSE
//         # There is no configuration override for level. Is there a default configuration for it?
//         IF theDescriptor.defaultConfiguration.level (§3.49.14, §, §3.50.3) is present THEN
//           SET level to theDescriptor.defaultConfiguration.level.
// IF level has not yet been set THEN
//     SET level to "warning".
fn resolve_level(
  rules: &[sarif::ReportingDescriptor],
  run: &sarif::Run,
  result: &sarif::Result,
) -> sarif::ResultLevel {
  result
    .kind
    .as_ref()
    .and_then(|value| {
      value.as_str().and_then(|kind| match kind {
        // If kind has the value "fail" and level is absent, then level SHALL be determined by the following procedure:
        "fail" => match result.level.as_ref() {
          Some(level) => level.as_str().and_then(|level| {
            sarif::ResultLevel::from_str(level).map_or(None, Option::from)
          }),
          None => result.rule.as_ref().and_then(|rule| {
            // IF rule (§3.27.7) is present THEN
            rule.index.and_then(|rule_index| {
              rules
                .get(rule_index as usize)
                //     LET theDescriptor be the reportingDescriptor object (§3.49) that it specifies.
                //     # Is there a configuration override for the level property?
                .and_then(|the_descriptor| {
                  //     IF result.provenance.invocationIndex (§3.27.29, §3.48.6) is >= 0 THEN
                  result
                    .provenance
                    .as_ref()
                    .and_then(|provenance| {
                      provenance.invocation_index.and_then(|invocation_index| {
                        run
                          .invocations
                          .iter()
                          .flatten()
                          .collect::<Vec<_>>()
                          .get(invocation_index as usize)
                          // LET theInvocation be the invocation object (§3.20) that it specifies.
                          // IF theInvocation.ruleConfigurationOverrides (§3.20.5) is present
                          //       AND it contains a configurationOverride object (§3.51) whose
                          //       descriptor property (§3.51.2) specifies theDescriptor THEN
                          .and_then(|the_invocation| {
                            the_invocation
                              .rule_configuration_overrides
                              .as_ref()
                              .and_then(|rule_configuration_overrides| {
                                rule_configuration_overrides
                                  .iter()
                                  .find(|v| {
                                    v.descriptor.id.as_ref()
                                      == Some(&the_descriptor.id)
                                  })
                                  .and_then(|the_override| {
                                    the_override
                                      .configuration
                                      .level
                                      .as_ref()
                                      .and_then(|value| {
                                        value.as_str().and_then(|level| {
                                          sarif::ResultLevel::from_str(level)
                                            .map_or(None, Option::from)
                                        })
                                      })
                                  })
                              })
                          })
                      })
                    })
                    .or_else(|| {
                      //         # There is no configuration override for level. Is there a default configuration for it?
                      //         IF theDescriptor.defaultConfiguration.level (§3.49.14, §, §3.50.3) is present THEN
                      //           SET level to theDescriptor.defaultConfiguration.level.
                      the_descriptor.default_configuration.as_ref().and_then(
                        |default_configuration| {
                          default_configuration.level.as_ref().and_then(
                            |value| {
                              value.as_str().and_then(|level| {
                                sarif::ResultLevel::from_str(level)
                                  .map_or(None, Option::from)
                              })
                            },
                          )
                        },
                      )
                    })
                })
            })
          }),
        },
        // If kind (§3.27.9) has any value other than "fail", then if level is absent, it SHALL default to "none", and if it is present, it SHALL have the value "none".
        _ => Some(sarif::ResultLevel::None),
      })
    })
    // IF level has not yet been set THEN
    //     SET level to "warning".
    .unwrap_or(sarif::ResultLevel::Warning)
}

// IF theMessage.text is present and the desired language is theRun.language THEN
//     Use the text or markdown property of theMessage as appropriate.
// IF the string has not yet been found THEN
//     IF theMessage occurs as the value of result.message (§3.27.11) THEN
//         LET theRule be the reportingDescriptor object (§3.49), an element of theComponent.rules (§3.19.23), which defines the rule that was violated by this result.
//         IF theRule exists AND theRule.messageStrings (§3.49.11) is present AND contains a property whose name equals theMessage.id THEN
//             LET theMFMS be the multiformatMessageString object (§3.12) that is the value of that property.
//             Use the text or markdown property of theMFMS as appropriate.
//     ELSE IF theMessage occurs as the value of notification.message (§3.58.5) THEN
//         LET theDescriptor be the reportingDescriptor object (§3.49), an element of theComponent.notifications (§3.19.23), which describes this notification.
//         IF theDescriptor exists AND theDescriptor.messageStrings is present AND contains a property whose name equals theMessage.id THEN
//             LET theMFMS be the multiformatMessageString object that is the value of that property.
//             Use the text or markdown property of theMFMS as appropriate.
// IF the string has not yet been found THEN
//     IF theComponent.globalMessageStrings (§3.19.22) is present AND contains a property whose name equals theMessage.id THEN
//             LET theMFMS be the multiformatMessageString object that is the value of that property.
//             Use the text or markdown property of theMFMS as appropriate.
// IF the string has not yet been found THEN
//     The lookup procedure fails (which means the SARIF log file is invalid).
fn resolve_message_text_from_result(
  result: &sarif::Result,
  run: &sarif::Run,
) -> Option<String> {
  result
    .message
    .text
    .as_ref()
    .cloned()
    // IF the string has not yet been found THEN
    .or_else(|| {
      //     IF theMessage occurs as the value of result.message (§3.27.11) THEN
      result.rule.as_ref().and_then(|the_rule| {
        the_rule.index.and_then(|rule_index| {
          run.tool.driver.rules.as_ref().and_then(|rules| {
            //         LET theRule be the reportingDescriptor object (§3.49), an element of theComponent.rules (§3.19.23), which defines the rule that was violated by this result.
            //         IF theRule exists AND theRule.messageStrings (§3.49.11) is present AND contains a property whose name equals theMessage.id THEN
            rules.get(rule_index as usize).and_then(|the_rule| {
              the_rule
                .message_strings
                .as_ref()
                .and_then(|message_strings| {
                  result.message.id.as_ref().and_then(|message_id| {
                    //             LET theMFMS be the multiformatMessageString object (§3.12) that is the value of that property.
                    //             Use the text or markdown property of theMFMS as appropriate.
                    message_strings
                      .get(message_id)
                      .map(|the_mfms| the_mfms.text.clone())
                  })
                })
            })
          })
        })
      })
    })
    // IF the string has not yet been found THEN
    //     IF theComponent.globalMessageStrings (§3.19.22) is present AND contains a property whose name equals theMessage.id THEN
    //             LET theMFMS be the multiformatMessageString object that is the value of that property.
    //             Use the text or markdown property of theMFMS as appropriate.
    .or_else(|| {
      run.tool.driver.global_message_strings.as_ref().and_then(
        |global_message_strings| {
          result.message.id.as_ref().and_then(|message_id| {
            global_message_strings
              .get(message_id)
              .map(|the_mfms| the_mfms.text.clone())
          })
        },
      )
    })
  // IF the string has not yet been found THEN
  //     The lookup procedure fails (which means the SARIF log file is invalid).
  // .or_else(|| None) # uncesscary but written for illustration
}

fn to_writer_pretty(sarif: &sarif::Sarif) -> Result<()> {
  let mut writer = StandardStream::stderr(ColorChoice::Always);
  let mut files = SimpleFiles::new();
  let config = codespan_reporting::term::Config::default();
  let mut message_counter = (0, 0, 0);
  sarif.runs.iter().try_for_each(|run| -> Result<()> {
    if let Some(results) = run.results.as_ref() {
      results.iter().try_for_each(|result| -> Result<()> {
        if_chain! {
          if let Some(message) = resolve_message_text_from_result(result, run);
          if let Some(locations) = result.locations.as_ref();
          then {
            locations.iter().try_for_each(|location| -> Result<()> {
              if_chain! {
                if let Some(physical_location) = location.physical_location.as_ref();
                if let Some(artifact_location) = physical_location.artifact_location.as_ref();
                if let Some(uri) = artifact_location.uri.as_ref();
                if let Some(region) = physical_location.region.as_ref();
                if let Ok(contents) = get_physical_location_contents(physical_location, run);
                let file_id = files.add(uri, contents);
                if let (Some(range_start), Some(range_end))  = get_byte_range(file_id, &files, region);
                then {
                  let rules = Vec::new();
                  let level = resolve_level(run.tool.driver.rules.as_ref().unwrap_or(&rules), run, result);
                  let mut diagnostic = match level {
                    ResultLevel::Warning => Diagnostic::<usize>::warning(),
                    ResultLevel::Error => Diagnostic::<usize>::error(),
                    ResultLevel::Note => Diagnostic::<usize>::note(),
                    _ => Diagnostic::warning(),
                  }.with_message(message.clone())
                  .with_labels(vec![Label::primary(
                    file_id,
                    range_start..range_end,
                  )]);
                  if_chain! {
                    if let Some(rule_index) = result.rule_index;
                    if let Some(rules) = run.tool.driver.rules.as_ref();
                    if let Some(rule) = rules.get(rule_index as usize);
                    then {
                      if let Some(short_description) = rule.short_description.as_ref() {
                        diagnostic = diagnostic.with_notes(vec![short_description.text.clone()])
                      }
                      if let Some(full_description) = rule.full_description.as_ref() {
                        diagnostic = diagnostic.with_notes(vec![full_description.text.clone()])
                      }
                      term::emit(&mut writer.lock(), &config, &files, &diagnostic,)?;
                      match diagnostic.severity {
                        codespan_reporting::diagnostic::Severity::Note => message_counter.0 += 1,
                        codespan_reporting::diagnostic::Severity::Warning => message_counter.1 += 1,
                        codespan_reporting::diagnostic::Severity::Error => message_counter.2 += 1,
                        _ =>  {}
                      }
                    }
                  }
                }
              }
              Ok(())
            })?;
          } else {
            println!("Oops! Not implemented yet.");
          }
        }

        Ok(())
      })?;
    };
    Ok(())
  })?;

  if message_counter.1 > 0 {
    writer
      .set_color(ColorSpec::new().set_fg(Some(Color::Yellow)).set_bold(true))?;
    writer.write_all("warning".as_bytes())?;
    writer.reset()?;
    writer.set_color(ColorSpec::new().set_bold(true))?;
    writer.write_all(
      format!(": {} warnings emitted\n", message_counter.1).as_bytes(),
    )?;
  }

  if message_counter.2 > 0 {
    writer
      .set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))?;
    writer.write_all("error".as_bytes())?;
    writer.reset()?;
    writer.set_color(ColorSpec::new().set_bold(true))?;
    writer.write_all(
      format!(": {} errors emitted\n", message_counter.1).as_bytes(),
    )?;
  }

  Ok(())
}

fn main() -> Result<()> {
  let matches = App::new("sarif-fmt")
        .about("Pretty print SARIF results")
        .after_help(
            "The expected input is generated by running 'cargo clippy --message-format=json'.",
        )
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::new("input")
                .about("input file; reads from stdin if none is given")
                .takes_value(true),
        )
        .get_matches();

  let read = match matches.value_of_os("input").map(Path::new) {
    Some(path) => Box::new(File::open(path)?) as Box<dyn Read>,
    None => Box::new(std::io::stdin()) as Box<dyn Read>,
  };
  let reader = BufReader::new(read);
  let sarif = process(reader)?;
  to_writer_pretty(&sarif)
}
