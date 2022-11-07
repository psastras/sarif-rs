#[cfg(feature = "clippy-converters")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "clippy-converters")))]
pub mod clippy;

#[cfg(feature = "hadolint-converters")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "hadolint-converters")))]
pub mod hadolint;

#[cfg(feature = "shellcheck-converters")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "shellcheck-converters")))]
pub mod shellcheck;

#[cfg(feature = "clang-tidy-converters")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "clang-tidy-converters")))]
pub mod clang_tidy;

#[cfg(feature = "cppcheck-converters")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "cppcheck-converters")))]
pub mod cppcheck;
