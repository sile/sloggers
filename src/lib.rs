//! This crate provides frequently used slog loggers and convenient functions.
//!
//! # Examples
//!
//! Createa a logger via `TerminalLoggerBuilder`:
//!
//! ```
//! use sloggers::terminal::TerminalLoggerBuilder;
//! ```
//!
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

pub type Result<T> = ::std::result::Result<T, Error>;
