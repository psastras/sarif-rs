#[cfg(any(feature = "clippy-converters", feature = "miri-converters"))]
mod cargo;

#[cfg(feature = "clippy-converters")]
#[cfg_attr(doc, doc(cfg(feature = "clippy-converters")))]
pub mod clippy;

#[cfg(feature = "miri-converters")]
#[cfg_attr(doc, doc(cfg(feature = "miri-converters")))]
pub mod miri;

#[cfg(feature = "hadolint-converters")]
#[cfg_attr(doc, doc(cfg(feature = "hadolint-converters")))]
pub mod hadolint;

#[cfg(feature = "shellcheck-converters")]
#[cfg_attr(doc, doc(cfg(feature = "shellcheck-converters")))]
pub mod shellcheck;

#[cfg(feature = "clang-tidy-converters")]
#[cfg_attr(doc, doc(cfg(feature = "clang-tidy-converters")))]
pub mod clang_tidy;
