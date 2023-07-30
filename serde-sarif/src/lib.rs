#![doc(html_root_url = "https://docs.rs/serde-sarif/0.4.0")]

//! # WARNING: VERY UNSTABLE (EARLY IMPLEMENTATION)
//!
//! This crate provides a type safe [serde](https://serde.rs/) compatible
//! [SARIF](https://sarifweb.azurewebsites.net/) structure. It is intended
//! for use in Rust code which may need to read or write SARIF files.
//!
//! The latest [documentation can be found here](https://docs.rs/serde_sarif).
//!
//! serde is a popular serialization framework for Rust. More information can be
//! found on the official repository: [https://github.com/serde-rs/serde](https://github.com/serde-rs/serde)
//!
//! SARIF or the Static Analysis Results Interchange Format is an industry
//! standard format for the output of static analysis tools. More information
//! can be found on the official website: [https://sarifweb.azurewebsites.net/](https://sarifweb.azurewebsites.net/).
//!
//! ## Usage
//!
//! For most cases, simply use the root [sarif::Sarif] struct with [serde] to read and
//! write to and from the struct.
//!
//! ## Example
//!
//!```rust
//! use serde_sarif::sarif::Sarif;
//!
//! let sarif: Sarif = serde_json::from_str(
//!   r#"{ "version": "2.1.0", "runs": [] }"#
//! ).unwrap();
//!
//! assert_eq!(
//!   sarif.version.to_string(),
//!   "\"2.1.0\"".to_string()
//! );
//! ```
//!
//! Because many of the [sarif::Sarif] structures contain a lot of optional fields, it is
//! often convenient to use the builder pattern to contstruct these structs. Each
//! structure has a builder with a default.
//!
//! ## Example
//!
//! ```rust
//! use serde_sarif::sarif::MessageBuilder;
//!
//! let message = MessageBuilder::default()
//!   .id("id")
//!   .build()
//!   .unwrap();
//! ```
//!
//! ## Internal Implementation Details
//!
//! The root [sarif::Sarif] struct is automatically generated from the latest Sarif
//! JSON schema, this is done at build time (via the buildscript).
//!
//! ## Crate Features
//!
//! This crate contains different features which may be enabled depndending on your
//! use case.
//!
//! ### Example
//!
//! ```toml
//! [dependencies]
//! serde-sarif = { version = "*", features = ["clippy-converters"] }
//! ```
//!
//! ### Converters
//! - **clang-tidy-converters** Provides conversions between clang tidy and SARIF
//!  types
//! - **clippy-converters** Provides conversions between Clippy
//!    and SARIF types
//! - **hadolint-converters** Provides conversions between hadolint
//!    and SARIF types
//! - **shellcheck-converters** Provides conversions between shellcheck
//!    and SARIF types
//!

pub mod converters;
pub mod sarif;
