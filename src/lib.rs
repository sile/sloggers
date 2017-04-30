#[macro_use]
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

pub use build::Build;
pub use config::Config;
pub use error::{Error, ErrorKind};
pub use misc::set_stdlog_logger;

pub mod config;
pub mod build;
pub mod null;
pub mod loggers;

mod error;
mod misc;

pub type Result<T> = ::std::result::Result<T, Error>;
