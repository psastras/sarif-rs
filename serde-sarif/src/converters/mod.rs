#[cfg(feature = "clippy-converters")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "clippy-converters")))]
pub mod clippy;

#[cfg(feature = "hadolint-converters")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "hadolint-converters")))]
pub mod hadolint;
