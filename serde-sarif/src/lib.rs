#![doc(html_root_url = "https://docs.rs/serde-sarif/0.1.0")]

//! This crate provides a type safe [serde](https://serde.rs/) compatible
//! [SARIF](https://sarifweb.azurewebsites.net/) structure. It is intended
//! for use in Rust code which may need to read or write SARIF files.
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
//! For most cases, simply use the root [Sarif] struct with [serde] to read and
//! write to and from the struct.
//!
//! ## Example
//!
//!```rust
//! use serde_sarif::Sarif;
//!
//! let sarif: Sarif = serde_json::from_str(r#"{ "version": "2.1.0", "runs": [] }"#).unwrap();
//! assert_eq!(sarif.version.to_string(), "\"2.1.0\"".to_string());
//! ```
//!
//! ## Internal Implementation Details
//!
//! The root [Sarif] struct is automatically generated from the latest Sarif
//! JSON schema.
//!

include!(concat!(env!("OUT_DIR"), "/sarif.rs"));
