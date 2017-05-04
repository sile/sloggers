//! This crate provides frequently used
//! [slog](https://github.com/slog-rs/slog) loggers and convenient functions.
//!
//! # Examples
//!
//! Creates a logger via `TerminalLoggerBuilder`:
//!
//! ```
//! #[macro_use]
//! extern crate slog;
//! extern crate sloggers;
//!
//! use sloggers::Build;
//! use sloggers::terminal::{TerminalLoggerBuilder, Destination};
//! use sloggers::types::Severity;
//!
//! # fn main() {
//! let mut builder = TerminalLoggerBuilder::new();
//! builder.level(Severity::Debug);
//! builder.destination(Destination::Stderr);
//!
//! let logger = builder.build().unwrap();
//! info!(logger, "Hello World!");
//! # }
//! ```
//!
//! Creates a logger from configuration text (TOML):
//!
//! ```
//! #[macro_use]
//! extern crate slog;
//! extern crate sloggers;
//!
//! use sloggers::{Build, Config, LoggerConfig};
//!
//! # fn main() {
//! let config = LoggerConfig::from_toml(r#"
//! type = "terminal"
//! level = "debug"
//! destination = "stderr"
//! "#).unwrap();
//!
//! let builder = config.try_into_builder().unwrap();
//! let logger = builder.build().unwrap();
//! info!(logger, "Hello World!");
//! # }
//! ```
#![warn(missing_docs)]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;
extern crate slog_stdlog;
extern crate slog_scope;
extern crate toml;
#[macro_use]
extern crate trackable;

pub use build::{Build, LoggerBuilder};
pub use config::{Config, LoggerConfig};
pub use error::{Error, ErrorKind};
pub use misc::set_stdlog_logger;

pub mod file;
pub mod null;
pub mod terminal;
pub mod types;

mod build;
mod config;
mod error;
mod misc;

/// A specialized `Result` type for this crate.
pub type Result<T> = ::std::result::Result<T, Error>;
