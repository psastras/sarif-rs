#![allow(clippy::derive_partial_eq_without_eq)]

use std::convert::TryFrom;
use strum_macros::Display;
use strum_macros::EnumString;
use thiserror::Error;

include!(concat!(env!("OUT_DIR"), "/sarif.rs"));

#[doc = "The SARIF format version of this log file."]
#[derive(Display, Debug, Serialize, Deserialize, EnumString)]
#[serde(untagged)]
pub enum Version {
  #[strum(serialize = "2.1.0")]
  V2_1_0,
}

// todo: should be generated / synced with schema.json
pub static SCHEMA_URL: &str =
  "https://schemastore.azurewebsites.net/schemas/json/sarif-2.1.0-rtm.5.json";

#[doc = "The role or roles played by the artifact in the analysis."]
#[derive(Display, Debug, Serialize, Deserialize, EnumString)]
#[serde(untagged)]
pub enum ArtifactRoles {
  #[strum(serialize = "analysisTarget")]
  AnalysisTarget,
  #[strum(serialize = "attachment")]
  Attachment,
  #[strum(serialize = "responseFile")]
  ResponseFile,
  #[strum(serialize = "resultFile")]
  ResultFile,
  #[strum(serialize = "standardStream")]
  StandardStream,
  #[strum(serialize = "tracedFile")]
  TracedFile,
  #[strum(serialize = "unmodified")]
  Unmodified,
  #[strum(serialize = "modified")]
  Modified,
  #[strum(serialize = "added")]
  Added,
  #[strum(serialize = "deleted")]
  Deleted,
  #[strum(serialize = "renamed")]
  Renamed,
  #[strum(serialize = "uncontrolled")]
  Uncontrolled,
  #[strum(serialize = "driver")]
  Driver,
  #[strum(serialize = "extension")]
  Extension,
  #[strum(serialize = "translation")]
  Translation,
  #[strum(serialize = "taxonomy")]
  Taxonomy,
  #[strum(serialize = "policy")]
  Policy,
  #[strum(serialize = "referencedOnCommandLine")]
  ReferencedOnCommandLine,
  #[strum(serialize = "memoryContents")]
  MemoryContents,
  #[strum(serialize = "directory")]
  Directory,
  #[strum(serialize = "userSpecifiedConfiguration")]
  UserSpecifiedConfiguration,
  #[strum(serialize = "toolSpecifiedConfiguration")]
  ToolSpecifiedConfiguration,
  #[strum(serialize = "debugOutputFile")]
  DebugOutputFile,
}

#[doc = "The SARIF format version of this external properties object."]
#[derive(Display, Debug, Serialize, Deserialize, EnumString)]
#[serde(untagged)]
pub enum ExternalPropertiesVersion {
  #[strum(serialize = "2.1.0")]
  V2_1_0,
}

#[doc = "A value specifying the severity level of the result."]
#[derive(Display, Debug, Serialize, Deserialize, EnumString)]
#[serde(untagged)]
pub enum NotificationLevel {
  #[strum(serialize = "none")]
  None,
  #[strum(serialize = "note")]
  Note,
  #[strum(serialize = "warning")]
  Warning,
  #[strum(serialize = "error")]
  Error,
}

#[doc = "Specifies the failure level for the report."]
#[derive(Display, Debug, Serialize, Deserialize, EnumString)]
#[serde(untagged)]
pub enum ReportingConfigurationLevel {
  #[strum(serialize = "none")]
  None,
  #[strum(serialize = "note")]
  Note,
  #[strum(serialize = "warning")]
  Warning,
  #[strum(serialize = "error")]
  Error,
}

#[doc = "A value that categorizes results by evaluation state."]
#[derive(Display, Debug, Serialize, Deserialize, EnumString)]
#[serde(untagged)]
pub enum ResultKind {
  #[strum(serialize = "notApplicable")]
  NotApplicable,
  #[strum(serialize = "pass")]
  Pass,
  #[strum(serialize = "fail")]
  Fail,
  #[strum(serialize = "review")]
  Review,
  #[strum(serialize = "open")]
  Open,
  #[strum(serialize = "informational")]
  Informational,
}

#[doc = "A value specifying the severity level of the result."]
#[derive(Clone, Copy, Display, Debug, Serialize, Deserialize, EnumString)]
#[serde(untagged)]
pub enum ResultLevel {
  #[strum(serialize = "none")]
  None,
  #[strum(serialize = "note")]
  Note,
  #[strum(serialize = "warning")]
  Warning,
  #[strum(serialize = "error")]
  Error,
}

#[doc = "The state of a result relative to a baseline of a previous run."]
#[derive(Display, Debug, Serialize, Deserialize, EnumString)]
#[serde(untagged)]
pub enum ResultBaselineState {
  #[strum(serialize = "new")]
  New,
  #[strum(serialize = "unchanged")]
  Unchanged,
  #[strum(serialize = "updated")]
  Updated,
  #[strum(serialize = "absent")]
  Absent,
}

#[doc = "Specifies the unit in which the tool measures columns."]
#[derive(Display, Debug, Serialize, Deserialize, EnumString)]
#[serde(untagged)]
pub enum ResultColumnKind {
  #[strum(serialize = "utf16CodeUnits")]
  Utf16CodeUnits,
  #[strum(serialize = "unicodeCodePoints")]
  UnicodeCodePoints,
}

#[doc = "A string that indicates where the suppression is persisted."]
#[derive(Display, Debug, Serialize, Deserialize, EnumString)]
#[serde(untagged)]
pub enum SupressionKind {
  #[strum(serialize = "inSource")]
  InSource,
  #[strum(serialize = "external")]
  External,
}

#[doc = "A string that indicates the review status of the suppression."]
#[derive(Display, Debug, Serialize, Deserialize, EnumString)]
#[serde(untagged)]
pub enum SupressionStatus {
  #[strum(serialize = "accepted")]
  Accepted,
  #[strum(serialize = "underReview")]
  UnderReview,
}

#[doc = "Specifies the importance of this location in understanding the code flow in which it occurs. The order from most to least important is \"essential\", \"important\", \"unimportant\". Default: \"important\"."]
#[derive(Display, Debug, Serialize, Deserialize, EnumString)]
#[serde(untagged)]
pub enum ThreadFlowLocationImportance {
  #[strum(serialize = "important")]
  Important,
  #[strum(serialize = "essential")]
  Essential,
}

#[doc = "The kinds of data contained in this object."]
#[derive(Display, Debug, Serialize, Deserialize, EnumString)]
#[serde(untagged)]
pub enum ToolComponentContents {
  #[strum(serialize = "localizedData")]
  LocalizedData,
  #[strum(serialize = "nonLocalizedData")]
  NonLocalizedData,
}

// todo: implement for other error types, probably convert to procmacro
#[derive(Error, Debug)]
pub enum BuilderError {
  #[error(transparent)]
  LocationBuilderError {
    #[from]
    source: LocationBuilderError,
  },
  #[error(transparent)]
  PhysicalLocationBuilderError {
    #[from]
    source: PhysicalLocationBuilderError,
  },
  #[error(transparent)]
  RegionBuilderError {
    #[from]
    source: RegionBuilderError,
  },
  #[error(transparent)]
  ArtifactLocationBuilderError {
    #[from]
    source: ArtifactLocationBuilderError,
  },
  #[error(transparent)]
  ResultBuilderError {
    #[from]
    source: ResultBuilderError,
  },
  #[error(transparent)]
  MessageBuilderError {
    #[from]
    source: MessageBuilderError,
  },
}

// Note that due to the blanket implementation in core, TryFrom<AsRef<String>>
// results in a compiler error.
// https://github.com/rust-lang/rust/issues/50133
impl TryFrom<&String> for MultiformatMessageString {
  type Error = MultiformatMessageStringBuilderError;

  fn try_from(message: &String) -> anyhow::Result<Self, Self::Error> {
    MultiformatMessageStringBuilder::default()
      .text(message.clone())
      .build()
  }
}

impl TryFrom<&String> for Message {
  type Error = MessageBuilderError;

  fn try_from(message: &String) -> anyhow::Result<Self, Self::Error> {
    MessageBuilder::default().text(message.clone()).build()
  }
}

impl TryFrom<&str> for Message {
  type Error = MessageBuilderError;

  fn try_from(message: &str) -> anyhow::Result<Self, Self::Error> {
    MessageBuilder::default().text(message).build()
  }
}

impl TryFrom<ToolComponent> for Tool {
  type Error = ToolBuilderError;

  fn try_from(
    tool_component: ToolComponent,
  ) -> anyhow::Result<Self, Self::Error> {
    ToolBuilder::default().driver(tool_component).build()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  macro_rules! map {
    ($( $key: expr => $val: expr ),*) => {{
         let mut map = ::std::collections::BTreeMap::new();
         $( map.insert($key, serde_json::json!($val)); )*
         map
    }}
}

  #[test]
  fn test_serialize_property_bag_empty() {
    let property_bag = PropertyBagBuilder::default().build().unwrap();
    let json = serde_json::to_string_pretty(&property_bag).unwrap();
    let json_expected = r#"{}"#;
    assert_eq!(json, json_expected);
  }

  #[test]
  fn test_serialize_property_bag_additional_properties() {
    let property_bag = PropertyBagBuilder::default()
      .additional_properties(map!["key1".to_string() => "value1"])
      .build()
      .unwrap();
    let json = serde_json::to_string_pretty(&property_bag).unwrap();
    let json_expected = r#"{
  "key1": "value1"
}"#;
    assert_eq!(json, json_expected);
  }

  #[test]
  fn test_deserialize_property_bag_empty() {
    let json = r#"{}"#;
    let property_bag: PropertyBag = serde_json::from_str(json).unwrap();
    let property_bag_expected = PropertyBagBuilder::default().build().unwrap();
    assert_eq!(property_bag, property_bag_expected);
  }

  #[test]
  fn test_deserialize_property_bag_additional_properties() {
    let json = r#"{
      "key1": "value1"
    }"#;
    let property_bag: PropertyBag = serde_json::from_str(json).unwrap();
    let property_bag_expected = PropertyBagBuilder::default()
      .additional_properties(map!["key1".to_string() => "value1"])
      .build()
      .unwrap();
    assert_eq!(property_bag, property_bag_expected);
  }
}
