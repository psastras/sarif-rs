use strum_macros::Display;
use strum_macros::EnumString;

include!(concat!(env!("OUT_DIR"), "/sarif.rs"));

#[doc = "The SARIF format version of this log file."]
#[derive(Display, Debug, Serialize, Deserialize, EnumString)]
#[serde(untagged)]
pub enum Version {
  #[strum(serialize = "2.1.0")]
  V2_1_0,
}

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
#[derive(Display, Debug, Serialize, Deserialize, EnumString)]
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
