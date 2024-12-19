#![allow(clippy::derive_partial_eq_without_eq)]

use strum_macros::Display;
use strum_macros::EnumString;
use thiserror::Error;

include!(concat!(env!("OUT_DIR"), "/sarif.rs"));

#[doc = "The SARIF format version of this log file."]
#[derive(Display, Debug, Serialize, Deserialize, EnumString)]
pub enum Version {
  #[strum(serialize = "2.1.0")]
  #[serde(rename = "2.1.0")]
  V2_1_0,
}

// todo: should be generated / synced with schema.json
pub static SCHEMA_URL: &str =
  "https://schemastore.azurewebsites.net/schemas/json/sarif-2.1.0.json";

#[doc = "The role or roles played by the artifact in the analysis."]
#[derive(Display, Debug, Serialize, Deserialize, EnumString)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum ArtifactRoles {
  AnalysisTarget,
  Attachment,
  ResponseFile,
  ResultFile,
  StandardStream,
  TracedFile,
  Unmodified,
  Modified,
  Added,
  Deleted,
  Renamed,
  Uncontrolled,
  Driver,
  Extension,
  Translation,
  Taxonomy,
  Policy,
  ReferencedOnCommandLine,
  MemoryContents,
  Directory,
  UserSpecifiedConfiguration,
  ToolSpecifiedConfiguration,
  DebugOutputFile,
}

#[doc = "The SARIF format version of this external properties object."]
#[derive(Display, Debug, Serialize, Deserialize, EnumString)]
pub enum ExternalPropertiesVersion {
  #[strum(serialize = "2.1.0")]
  #[serde(rename = "2.1.0")]
  V2_1_0,
}

#[doc = "A value specifying the severity level of the result."]
#[derive(Display, Debug, Serialize, Deserialize, EnumString)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum NotificationLevel {
  None,
  Note,
  Warning,
  Error,
}

#[doc = "Specifies the failure level for the report."]
#[derive(Display, Debug, Serialize, Deserialize, EnumString)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum ReportingConfigurationLevel {
  None,
  Note,
  Warning,
  Error,
}

#[doc = "A value that categorizes results by evaluation state."]
#[derive(
  Clone, Display, Debug, Serialize, Deserialize, EnumString, PartialEq,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum ResultKind {
  NotApplicable,
  Pass,
  Fail,
  Review,
  Open,
  Informational,
}

#[doc = "A value specifying the severity level of the result."]
#[derive(
  Clone, Copy, Display, Debug, Serialize, Deserialize, EnumString, PartialEq,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum ResultLevel {
  None,
  Note,
  Warning,
  Error,
}

#[doc = "The state of a result relative to a baseline of a previous run."]
#[derive(Display, Debug, Serialize, Deserialize, EnumString)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum ResultBaselineState {
  New,
  Unchanged,
  Updated,
  Absent,
}

#[doc = "Specifies the unit in which the tool measures columns."]
#[derive(Display, Debug, Serialize, Deserialize, EnumString)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum ResultColumnKind {
  Utf16CodeUnits,
  UnicodeCodePoints,
}

#[doc = "A string that indicates where the suppression is persisted."]
#[derive(Display, Debug, Serialize, Deserialize, EnumString)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum SupressionKind {
  InSource,
  External,
}

#[doc = "A string that indicates the review status of the suppression."]
#[derive(Display, Debug, Serialize, Deserialize, EnumString)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum SupressionStatus {
  Accepted,
  UnderReview,
}

#[doc = "Specifies the importance of this location in understanding the code flow in which it occurs. The order from most to least important is \"essential\", \"important\", \"unimportant\". Default: \"important\"."]
#[derive(Display, Debug, Serialize, Deserialize, EnumString)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum ThreadFlowLocationImportance {
  Important,
  Essential,
}

#[doc = "The kinds of data contained in this object."]
#[derive(Display, Debug, Serialize, Deserialize, EnumString)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum ToolComponentContents {
  LocalizedData,
  NonLocalizedData,
}

#[derive(Error, Debug)]
pub enum BuilderError {
  #[error("uninitialized field: {0}")]
  UninitializedField(&'static str),
}

// Note that due to the blanket implementation in core, TryFrom<AsRef<String>>
// results in a compiler error.
// https://github.com/rust-lang/rust/issues/50133
impl From<&String> for MultiformatMessageString {
  fn from(message: &String) -> Self {
    MultiformatMessageString::builder()
      .text(message.clone())
      .build()
  }
}

impl From<&String> for Message {
  fn from(message: &String) -> Self {
    Message::builder().text(message.clone()).build()
  }
}

impl From<&str> for Message {
  fn from(message: &str) -> Self {
    Message::builder().text(message).build()
  }
}

impl From<ToolComponent> for Tool {
  fn from(tool_component: ToolComponent) -> Self {
    Tool::builder().driver(tool_component).build()
  }
}

#[cfg(test)]
mod tests {
  use std::str::FromStr;

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
    let property_bag = PropertyBag::builder().build();
    let json = serde_json::to_string_pretty(&property_bag).unwrap();
    let json_expected = r#"{}"#;
    assert_eq!(json, json_expected);
  }

  #[test]
  fn test_serialize_property_bag_additional_properties() {
    let property_bag = PropertyBag::builder()
      .additional_properties(map!["key1".to_string() => "value1"])
      .build();
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
    let property_bag_expected = PropertyBag::builder().build();
    assert_eq!(property_bag, property_bag_expected);
  }

  #[test]
  fn test_deserialize_property_bag_additional_properties() {
    let json = r#"{
      "key1": "value1"
    }"#;
    let property_bag: PropertyBag = serde_json::from_str(json).unwrap();
    let property_bag_expected = PropertyBag::builder()
      .additional_properties(map!["key1".to_string() => "value1"])
      .build();
    assert_eq!(property_bag, property_bag_expected);
  }

  #[test]
  fn test_serialize_resultkind() {
    assert_eq!(
      serde_json::to_string(&ResultKind::Fail).unwrap(),
      "\"fail\""
    );
  }

  #[test]
  fn test_parse_utf16codeunits() {
    let v = ResultColumnKind::from_str("utf16CodeUnits").unwrap();
    assert!(matches!(v, ResultColumnKind::Utf16CodeUnits));
  }
}
