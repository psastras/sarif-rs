#![doc(html_root_url = "https://docs.rs/sarif-fmt/0.3.5")]
#![recursion_limit = "256"]
//! # WARNING: VERY UNSTABLE (EARLY IMPLEMENTATION)
//!
//! This crate provides a command line tool to pretty print SARIF files to
//! easy human readable output.
//!
//! The latest [documentation can be found here](https://docs.rs/sarif_fmt).
//!
//! SARIF or the Static Analysis Results Interchange Format is an industry
//! standard format for the output of static analysis tools. More information
//! can be found on the official website: [https://sarifweb.azurewebsites.net/](https://sarifweb.azurewebsites.net/).
//!
//! ## Installation
//!
//! `sarif-fmt` may be installed via `cargo`
//!
//! ```shell
//! cargo install sarif-fmt
//! ```
//!
//! or downloaded directly from Github Releases
//!
//!```shell
//! # make sure to adjust the target and version (you may also want to pin to a specific version)
//! curl -sSL https://github.com/psastras/sarif-rs/releases/download/sarif-fmt-latest/sarif-fmt-x86_64-unknown-linux-gnu -o sarif-fmt
//! ```
//!
//! ## Usage
//!
//! For most cases, simply pipe a SARIF file into `sarif-fmt` (`cat ./foo.sarif | sarif-fmt`)
//!
//! ## Example
//!
//!```shell
//! $ cargo clippy --message-format=json | clippy-sarif | sarif-fmt
//! $ warning: using `Option.and_then(|x| Some(y))`, which is more succinctly expressed as `map(|x| y)`
//!     ┌─ sarif-fmt/src/bin.rs:423:13
//!     │  
//! 423 │ ╭             the_rule
//! 424 │ │               .full_description
//! 425 │ │               .as_ref()
//! 426 │ │               .and_then(|mfms| Some(mfms.text.clone()))
//!     │ ╰───────────────────────────────────────────────────────^
//!     │  
//!     = `#[warn(clippy::bind_instead_of_map)]` on by default
//!       for further information visit https://rust-lang.github.io/rust-clippy/master#bind_instead_of_map
//! ```
//!
//! Often it is useful to record the SARIF file for machine processing but also
//! print the nicely formatted results to stdout at the same time. This can be done
//! using the `tee` command:
//!
//! ```shell
//! $ clang-tidy -checks=cert-* cpp.cpp -- | clang-tidy-sarif | tee clang-tidy.sarif | sarif-fmt
//! $ 2 warnings generated.
//! warning: 'atoi' used to convert a string to an integer value, but function will not report conversion errors; consider using 'strtol' instead [cert-err34-c]
//!   ┌─ /home/psastras/repos/sarif-rs/sarif-fmt/tests/data/cpp.cpp:4:10
//!   │
//! 4 │   return atoi(num);
//!   │          ^^^^^^^^^^
//!
//! warning: calling 'system' uses a command processor [cert-env33-c]
//!   ┌─ /home/psastras/repos/sarif-rs/sarif-fmt/tests/data/cpp.cpp:8:3
//!   │
//! 8 │   system("ls");
//!   │   ^^^^^^^^^^^^^
//!
//! $ cat clang-tidy.sarif
//! {
//!   "runs": [
//!     {
//!       "results": [
//!         {
//!           "level": "warning",
//!           "locations": [
//!             {
//!               "physicalLocation": {
//!                 "artifactLocation": {
//!                   "uri": "cpp.cpp"
//!                 },
//!                 "region": {
//!                   "startColumn": 10,
//!                   "startLine": 4
//!                 }
//!               }
//!             }
//!           ],
//!           "message": {
//!             "text": "'atoi' used to convert a string to an integer value, but function will not report conversion errors; consider using 'strtol' instead [cert-err34-c]"
//!           }
//!         },
//!         {
//!           "level": "warning",
//!           "locations": [
//!             {
//!               "physicalLocation": {
//!                 "artifactLocation": {
//!                   "uri": "cpp.cpp"
//!                 },
//!                 "region": {
//!                   "startColumn": 3,
//!                   "startLine": 8
//!                 }
//!               }
//!             }
//!           ],
//!           "message": {
//!             "text": "calling 'system' uses a command processor [cert-env33-c]"
//!           }
//!         }
//!       ],
//!       "tool": {
//!         "driver": {
//!           "name": "clang-tidy"
//!         }
//!       }
//!     }
//!   ],
//!   "version": "2.1.0"
//! }
//! ```
//!
use anyhow::Result;
use clap::{Parser, ValueEnum};
use codespan_reporting::diagnostic;
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
use serde_sarif::sarif;
use serde_sarif::sarif::ResultLevel;
use std::fs::File;
use std::io::Write;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
use std::usize;

fn process<R: BufRead>(mut reader: R) -> Result<sarif::Sarif> {
  let mut data = String::new();
  reader.read_to_string(&mut data)?;
  let s: sarif::Sarif = serde_json::from_str(&data)?;
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
    (region.start_line, region.start_column.or(Some(1)))
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
      region.end_column.map_or_else(
        // if no end column use the line's last column
        || {
          region
            .end_line
            .map_or_else(|| region.start_line, Some)
            .and_then(|start_line| {
              files
                .line_range(file_id, start_line as usize - 1)
                .map_or(None, Option::from)
                .and_then(|byte_range| {
                  byte_range.last().and_then(|last_byte| {
                    files
                      .column_number(
                        file_id,
                        start_line as usize - 1,
                        last_byte,
                      )
                      .map_or(None, |v| Option::from(v as i64))
                  })
                })
            })
        },
        Some,
      ),
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

fn resolve_full_description_from_result(
  rules: &[sarif::ReportingDescriptor],
  result: &sarif::Result,
) -> Option<String> {
  result
    .rule_index
    .and_then(|rule_index| {
      rules.get(rule_index as usize).and_then(|the_descriptor| {
        the_descriptor
          .full_description
          .as_ref()
          .map(|mfms| mfms.text.clone())
      })
    })
    .or_else(|| {
      result.rule.as_ref().and_then(|rule| {
        rule.index.and_then(|rule_index| {
          rules.get(rule_index as usize).and_then(|the_descriptor| {
            the_descriptor
              .full_description
              .as_ref()
              .map(|mfms| mfms.text.clone())
          })
        })
      })
    })
}

fn resolve_short_description_from_result(
  rules: &[sarif::ReportingDescriptor],
  result: &sarif::Result,
) -> Option<String> {
  result
    .rule_index
    .and_then(|rule_index| {
      rules.get(rule_index as usize).and_then(|the_descriptor| {
        the_descriptor
          .short_description
          .as_ref()
          .map(|mfms| mfms.text.clone())
      })
    })
    .or_else(|| {
      result.rule.as_ref().and_then(|rule| {
        rule.index.and_then(|rule_index| {
          rules.get(rule_index as usize).and_then(|the_descriptor| {
            the_descriptor
              .short_description
              .as_ref()
              .map(|mfms| mfms.text.clone())
          })
        })
      })
    })
}

fn to_writer_plain(sarif: &sarif::Sarif) -> Result<()> {
  let mut files = SimpleFiles::new();
  sarif.runs.iter().try_for_each(|run| -> Result<()> {
    let mut diagnostics = vec![];
    if let Some(results) = run.results.as_ref() {
      results.iter().try_for_each(|result| -> Result<()> {
        let rules = vec![];
        let rules = run.tool.driver.rules.as_ref().unwrap_or(&rules);
        let level = resolve_level(rules, run, result);

        if let (Some(text), Some(locations)) = (
          resolve_message_text_from_result(result, run),
          result.locations.as_ref(),
        ) {
          locations.iter().for_each(|location| {
            if let Some((file_id, range)) = location
              .physical_location
              .as_ref()
              .and_then(|physical_location| {
                physical_location.artifact_location.as_ref().and_then(
                  |artifact_location| {
                    artifact_location.uri.as_ref().and_then(|uri| {
                      physical_location.region.as_ref().and_then(|region| {
                        get_physical_location_contents(physical_location, run)
                          .ok()
                          .and_then(|contents| {
                            let file_id = files.add(uri, contents);
                            if let (Some(range_start), Some(range_end)) =
                              get_byte_range(file_id, &files, region)
                            {
                              Some((file_id, range_start..range_end))
                            } else {
                              None
                            }
                          })
                      })
                    })
                  },
                )
              })
            {
              if let (Ok(name), Ok(location)) =
                (files.name(file_id), files.location(file_id, range.start))
              {
                let diagnostic = (
                  name.clone(),
                  level,
                  location.line_number,
                  location.column_number,
                  text.clone(),
                );
                diagnostics.push(diagnostic);
              } else {
                // todo: no location found
              }
            }
          });
          // todo: no location found
        }

        Ok(())
      })?;
    };

    diagnostics.sort_by(|a, b| {
      a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal)
    });

    diagnostics.iter().for_each(|diagnostic| {
      println!(
        "{}:{}:{}: {}: {}",
        diagnostic.0, diagnostic.2, diagnostic.3, diagnostic.1, diagnostic.4
      )
    });
    Ok(())
  })?;

  Ok(())
}

fn to_writer_pretty(sarif: &sarif::Sarif) -> Result<()> {
  let mut writer = StandardStream::stdout(ColorChoice::Auto);
  let mut files = SimpleFiles::new();
  let config = codespan_reporting::term::Config::default();
  let mut message_counter = (0, 0, 0);
  sarif.runs.iter().try_for_each(|run| -> Result<()> {
    if let Some(results) = run.results.as_ref() {
      results.iter().try_for_each(|result| -> Result<()> {
        let rules = vec![];
        let rules = run.tool.driver.rules.as_ref().unwrap_or(&rules);
        let level = resolve_level(rules, run, result);
        let mut diagnostic: Diagnostic<usize> = Diagnostic::new(match level {
          ResultLevel::Note => diagnostic::Severity::Note,
          ResultLevel::Warning => diagnostic::Severity::Warning,
          ResultLevel::Error => diagnostic::Severity::Error,
          _ => diagnostic::Severity::Warning,
        });
        if let Some(message) = resolve_message_text_from_result(result, run) {
          diagnostic.message = message;
        }
        if let Some(text) = resolve_short_description_from_result(rules, result)
        {
          diagnostic.notes.push(text);
        }
        if let Some(text) = resolve_full_description_from_result(rules, result)
        {
          diagnostic.notes.push(text);
        }

        if let Some(locations) = result.locations.as_ref() {
          locations.iter().for_each(|location| {
            if let Some((file_id, range)) = location
              .physical_location
              .as_ref()
              .and_then(|physical_location| {
                physical_location.artifact_location.as_ref().and_then(
                  |artifact_location| {
                    artifact_location.uri.as_ref().and_then(|uri| {
                      physical_location.region.as_ref().and_then(|region| {
                        get_physical_location_contents(physical_location, run)
                          .ok()
                          .and_then(|contents| {
                            let file_id = files.add(uri, contents);
                            if let (Some(range_start), Some(range_end)) =
                              get_byte_range(file_id, &files, region)
                            {
                              Some((file_id, range_start..range_end))
                            } else {
                              None
                            }
                          })
                      })
                    })
                  },
                )
              })
            {
              diagnostic.labels.push(Label::primary(file_id, range));
            }
          });
        }

        if let Some(locations) = result.related_locations.as_ref() {
          locations.iter().for_each(|location| {
            if let Some((file_id, range, message)) = location
              .physical_location
              .as_ref()
              .and_then(|physical_location| {
                physical_location.artifact_location.as_ref().and_then(
                  |artifact_location| {
                    artifact_location.uri.as_ref().and_then(|uri| {
                      physical_location.region.as_ref().and_then(|region| {
                        get_physical_location_contents(physical_location, run)
                          .ok()
                          .and_then(|contents| {
                            let file_id = files.add(uri, contents);
                            if let (Some(range_start), Some(range_end)) =
                              get_byte_range(file_id, &files, region)
                            {
                              Some((
                                file_id,
                                range_start..range_end,
                                location
                                  .message
                                  .as_ref()
                                  .and_then(|x| x.text.clone()),
                              ))
                            } else {
                              None
                            }
                          })
                      })
                    })
                  },
                )
              })
            {
              diagnostic.labels.push(
                Label::secondary(file_id, range)
                  .with_message(message.unwrap_or("".to_string())),
              );
            }
          });
        }

        term::emit(&mut writer.lock(), &config, &files, &diagnostic)?;
        match diagnostic.severity {
          codespan_reporting::diagnostic::Severity::Note => {
            message_counter.0 += 1
          }
          codespan_reporting::diagnostic::Severity::Warning => {
            message_counter.1 += 1
          }
          codespan_reporting::diagnostic::Severity::Error => {
            message_counter.2 += 1
          }
          _ => {}
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
    writer.reset()?;
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
    writer.reset()?;
  }

  Ok(())
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum MessageFormat {
  Plain,
  Pretty,
}

#[derive(Parser, Debug)]
#[command(
  version,
  about = "Pretty print SARIF results",
  after_help = "The expected input is a SARIF file (ex. cat foo.sarif | sarif-fmt).",
  long_about = None
)]
struct Args {
  /// One of plain or pretty
  #[arg(short, long, value_enum, default_value = "pretty")]
  message_format: MessageFormat,
  /// input file; reads from stdin if none is given
  #[arg(short, long)]
  input: Option<std::path::PathBuf>,
}

fn main() -> Result<()> {
  let args = Args::parse();

  let read = match args.input {
    Some(path) => Box::new(File::open(path)?) as Box<dyn Read>,
    None => Box::new(std::io::stdin()) as Box<dyn Read>,
  };
  let reader = BufReader::new(read);
  let sarif = process(reader)?;
  match args.message_format {
    MessageFormat::Plain => to_writer_plain(&sarif),
    MessageFormat::Pretty => to_writer_pretty(&sarif),
  }
}
